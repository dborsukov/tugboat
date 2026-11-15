//! Database backups and restore.
//!
//! A backup is a zip archive with:
//!
//! - `manifest.json` — format version, timestamp, kind, app/schema version and
//!   a blake3 hash of the database;
//! - `main.db` — database snapshot made using `VACUUM INTO`;
//! - `invoices/` — the generated documents.
//!
//! Restoring is staged while the app runs, then applied on the next start
//! before the database is opened. The data it replaces is moved aside into a
//! `pre-restore-*` folder.

use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteConnection},
    Connection, SqlitePool,
};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{
    error::{AppError, AppResult},
    AppState,
};

const FORMAT_VERSION: u32 = 1;
const BACKUP_PREFIX: &str = "tugboat-backup-";
const MANIFEST_FILE: &str = "manifest.json";
const DB_FILE: &str = "main.db";
const BACKUPS_DIR: &str = "backups";
const RESTORE_STAGING: &str = ".restore-staging";
const RESTORE_MARKER: &str = ".restore-pending.json";
const STATUS_FILE: &str = "backup-status.json";
const BACKUP_STATUS_EVENT: &str = "backup-status";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// Serialises backups and restores inside this process.
static BACKUP_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
/// Whether a backup is running right now (not persisted across restarts).
static BACKUP_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum BackupKind {
    Manual,
    Scheduled,
    PreMigration,
    PreUpdate,
    PreRestore,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettings {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default = "default_interval_hours")]
    pub interval_hours: u32,
    #[serde(default = "default_retention")]
    pub retention: u32,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            folder: None,
            interval_hours: default_interval_hours(),
            retention: default_retention(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_interval_hours() -> u32 {
    24
}

fn default_retention() -> u32 {
    14
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupManifest {
    format: u32,
    created_at: String,
    kind: BackupKind,
    app_version: String,
    schema_version: i64,
    db_bytes: u64,
    db_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub file_name: String,
    pub created_at: String,
    pub kind: BackupKind,
    pub size_bytes: f64,
    pub app_version: String,
    pub schema_version: f64,
    /// True when the backup comes from a newer app or schema
    pub is_newer: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    pub running: bool,
    pub last_attempt_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupStatusFile {
    last_attempt_at: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupOverview {
    pub settings: BackupSettings,
    pub effective_folder: String,
    pub backups: Vec<BackupInfo>,
    pub status: BackupStatus,
    pub folder_available: bool,
    pub current_app_version: String,
    pub current_schema_version: f64,
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn timestamp() -> String {
    Utc::now().format("%Y%m%d-%H%M%S").to_string()
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("backup.json")
}

pub fn load_settings(data_dir: &Path) -> BackupSettings {
    fs::read_to_string(settings_path(data_dir))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn validate_settings(settings: &BackupSettings) -> AppResult<()> {
    if !(1..=720).contains(&settings.interval_hours) {
        return Err(AppError::validation(
            "Інтервал резервного копіювання має бути від 1 до 720 годин",
        ));
    }
    if !(1..=100).contains(&settings.retention) {
        return Err(AppError::validation(
            "Кількість резервних копій має бути від 1 до 100",
        ));
    }
    if let Some(folder) = &settings.folder {
        if folder.trim().is_empty() {
            return Err(AppError::validation("Не вказано теку для резервних копій"));
        }
        if !Path::new(folder).is_absolute() {
            return Err(AppError::validation(
                "Шлях до теки резервних копій має бути абсолютним",
            ));
        }
    }
    Ok(())
}

pub fn save_settings(data_dir: &Path, settings: &BackupSettings) -> AppResult<()> {
    validate_settings(settings)?;
    let path = settings_path(data_dir);
    let staging = path.with_extension("json.tmp");
    fs::write(&staging, serde_json::to_vec_pretty(settings)?)?;
    fs::rename(&staging, &path)?;
    Ok(())
}

pub fn resolve_folder(data_dir: &Path, settings: &BackupSettings) -> PathBuf {
    match &settings.folder {
        Some(folder) => PathBuf::from(folder),
        None => data_dir.join(BACKUPS_DIR),
    }
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

fn status_path(data_dir: &Path) -> PathBuf {
    data_dir.join(STATUS_FILE)
}

fn load_status_file(data_dir: &Path) -> BackupStatusFile {
    fs::read_to_string(status_path(data_dir))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_status_file(data_dir: &Path, status: &BackupStatusFile) {
    let path = status_path(data_dir);
    let staging = path.with_extension("json.tmp");
    if let Ok(raw) = serde_json::to_vec_pretty(status) {
        if fs::write(&staging, raw).is_ok() {
            let _ = fs::rename(&staging, &path);
        }
    }
}

pub fn current_status(data_dir: &Path) -> BackupStatus {
    let file = load_status_file(data_dir);
    BackupStatus {
        running: BACKUP_RUNNING.load(Ordering::SeqCst),
        last_attempt_at: file.last_attempt_at,
        last_error: file.last_error,
    }
}

fn emit_status(app: Option<&AppHandle>, data_dir: &Path) {
    if let Some(app) = app {
        let _ = app.emit(BACKUP_STATUS_EVENT, current_status(data_dir));
    }
}

fn mark_attempt_started(data_dir: &Path, app: Option<&AppHandle>) {
    BACKUP_RUNNING.store(true, Ordering::SeqCst);
    let mut file = load_status_file(data_dir);
    file.last_attempt_at = Some(now());
    file.last_error = None;
    save_status_file(data_dir, &file);
    emit_status(app, data_dir);
}

fn mark_attempt_finished(data_dir: &Path, app: Option<&AppHandle>, error: Option<&str>) {
    BACKUP_RUNNING.store(false, Ordering::SeqCst);
    let mut file = load_status_file(data_dir);
    file.last_attempt_at = Some(now());
    file.last_error = error.map(str::to_string);
    save_status_file(data_dir, &file);
    emit_status(app, data_dir);
}

// ---------------------------------------------------------------------------
// Database helpers
// ---------------------------------------------------------------------------

async fn schema_version(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations WHERE success = 1",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0)
}

async fn snapshot_database(pool: &SqlitePool, destination: &Path) -> AppResult<()> {
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    let escaped = destination.to_string_lossy().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{escaped}'"))
        .execute(pool)
        .await
        .map_err(|error| {
            AppError::Generic(format!("Не вдалося створити знімок бази даних: {error}"))
        })?;
    Ok(())
}

async fn verify_database(path: &Path) -> AppResult<()> {
    let options = SqliteConnectOptions::new().filename(path).read_only(true);
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(|error| AppError::Generic(format!("Не вдалося відкрити знімок: {error}")))?;

    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&mut connection)
        .await
        .map_err(|error| AppError::Generic(format!("Не вдалося перевірити знімок: {error}")))?;

    connection.close().await.ok();

    if result != "ok" {
        return Err(AppError::Generic(format!(
            "Знімок бази даних пошкоджено: {result}"
        )));
    }
    Ok(())
}

fn hash_file(path: &Path) -> io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

// ---------------------------------------------------------------------------
// Zip archive
// ---------------------------------------------------------------------------

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
}

fn zip_error(context: &str, error: zip::result::ZipError) -> AppError {
    AppError::Generic(format!("{context}: {error}"))
}

fn write_archive(
    destination: &Path,
    snapshot: &Path,
    manifest: &BackupManifest,
    invoices: &Path,
) -> AppResult<()> {
    let file = File::create(destination)?;
    let mut archive = ZipWriter::new(file);

    archive
        .start_file(MANIFEST_FILE, zip_options())
        .map_err(|error| zip_error("Не вдалося створити архів", error))?;
    archive.write_all(serde_json::to_string_pretty(manifest)?.as_bytes())?;

    archive
        .start_file(DB_FILE, zip_options())
        .map_err(|error| zip_error("Не вдалося створити архів", error))?;
    let mut database = File::open(snapshot)?;
    io::copy(&mut database, &mut archive)?;

    if invoices.is_dir() {
        add_directory(&mut archive, invoices, "invoices")?;
    }

    archive
        .finish()
        .map_err(|error| zip_error("Не вдалося завершити архів", error))?;
    Ok(())
}

fn add_directory(archive: &mut ZipWriter<File>, directory: &Path, prefix: &str) -> AppResult<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty() {
            continue;
        }
        let archive_name = format!("{prefix}/{name}");

        if file_type.is_dir() {
            add_directory(archive, &entry.path(), &archive_name)?;
        } else if file_type.is_file() {
            archive
                .start_file(archive_name, zip_options())
                .map_err(|error| zip_error("Не вдалося додати файл до архіву", error))?;
            let mut file = File::open(entry.path())?;
            io::copy(&mut file, archive)?;
        }
    }
    Ok(())
}

fn read_manifest(path: &Path) -> AppResult<BackupManifest> {
    let file = File::open(path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| zip_error("Не вдалося прочитати архів", error))?;

    let entry = archive
        .by_name(MANIFEST_FILE)
        .map_err(|_| AppError::Generic("В архіві немає опису резервної копії".to_string()))?;

    let mut raw = String::new();
    entry
        .take(MAX_MANIFEST_BYTES)
        .read_to_string(&mut raw)
        .map_err(|error| AppError::Generic(format!("Не вдалося прочитати опис: {error}")))?;

    serde_json::from_str(&raw)
        .map_err(|error| AppError::Generic(format!("Не вдалося розібрати опис: {error}")))
}

fn extract_archive(archive_path: &Path, staging: &Path) -> AppResult<()> {
    let file = File::open(archive_path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| zip_error("Не вдалося прочитати архів", error))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| zip_error("Не вдалося прочитати архів", error))?;

        let Some(enclosed) = entry.enclosed_name() else {
            continue;
        };
        let name = enclosed.to_string_lossy().replace('\\', "/");

        if name == DB_FILE {
            let mut output = File::create(staging.join(DB_FILE))?;
            io::copy(&mut entry, &mut output)?;
        } else if let Some(relative) = name.strip_prefix("invoices/") {
            if relative.is_empty() {
                continue;
            }
            let target = staging.join("invoices").join(relative);

            if entry.is_dir() {
                fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut output = File::create(&target)?;
                io::copy(&mut entry, &mut output)?;
            }
        }
    }

    if !staging.join(DB_FILE).is_file() {
        return Err(AppError::Generic(
            "В архіві немає файлу бази даних".to_string(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Creating and listing backups
// ---------------------------------------------------------------------------

fn unique_backup_name(folder: &Path, stamp: &str) -> String {
    let mut name = format!("{BACKUP_PREFIX}{stamp}.zip");
    let mut index = 2;
    while folder.join(&name).exists() {
        name = format!("{BACKUP_PREFIX}{stamp}-{index}.zip");
        index += 1;
    }
    name
}

pub fn list_backups_in(folder: &Path) -> AppResult<Vec<BackupInfo>> {
    if !folder.is_dir() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("zip") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with(BACKUP_PREFIX) {
            continue;
        }

        match read_manifest(&path) {
            Ok(manifest) => backups.push(BackupInfo {
                file_name: file_name.to_string(),
                created_at: manifest.created_at,
                kind: manifest.kind,
                size_bytes: entry
                    .metadata()
                    .map(|meta| meta.len() as f64)
                    .unwrap_or(0.0),
                app_version: manifest.app_version,
                schema_version: manifest.schema_version as f64,
                is_newer: false,
            }),
            Err(error) => {
                log::warn!(
                    "Пропускаю пошкоджену резервну копію {}: {error}",
                    path.display()
                )
            }
        }
    }

    backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(backups)
}

fn prune_backups(folder: &Path, retention: u32) -> AppResult<()> {
    let keep = retention.max(1) as usize;
    if !folder.is_dir() {
        return Ok(());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("zip") {
            continue;
        }
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(BACKUP_PREFIX) && name.ends_with(".zip"))
        {
            continue;
        }

        match read_manifest(&path) {
            // Safety copies are never removed by retention.
            Ok(manifest) if manifest.kind == BackupKind::PreRestore => continue,
            Ok(_) => files.push(path),
            Err(_) => continue,
        }
    }

    files.sort_by(|left, right| right.file_name().cmp(&left.file_name()));
    for path in files.into_iter().skip(keep) {
        if let Err(error) = fs::remove_file(&path) {
            log::warn!(
                "Не вдалося видалити стару копію {}: {error}",
                path.display()
            );
        }
    }
    Ok(())
}

async fn make_backup(
    pool: &SqlitePool,
    data_dir: &Path,
    app_version: &str,
    kind: BackupKind,
    app: Option<&AppHandle>,
) -> AppResult<BackupInfo> {
    let _guard = BACKUP_LOCK.lock().await;
    make_backup_tracked(pool, data_dir, app_version, kind, app).await
}

/// Runs a backup and publishes its running/success/failure status.
async fn make_backup_tracked(
    pool: &SqlitePool,
    data_dir: &Path,
    app_version: &str,
    kind: BackupKind,
    app: Option<&AppHandle>,
) -> AppResult<BackupInfo> {
    mark_attempt_started(data_dir, app);
    let result = make_backup_locked(pool, data_dir, app_version, kind).await;

    match &result {
        Ok(_) => mark_attempt_finished(data_dir, app, None),
        Err(error) => mark_attempt_finished(data_dir, app, Some(&error.to_string())),
    }
    result
}

async fn make_backup_locked(
    pool: &SqlitePool,
    data_dir: &Path,
    app_version: &str,
    kind: BackupKind,
) -> AppResult<BackupInfo> {
    let settings = load_settings(data_dir);
    let folder = resolve_folder(data_dir, &settings);
    fs::create_dir_all(&folder).map_err(|error| {
        AppError::Generic(format!(
            "Не вдалося створити теку резервних копій {}: {error}",
            folder.display()
        ))
    })?;

    let created_at = now();
    let stamp = timestamp();
    let file_name = unique_backup_name(&folder, &stamp);
    let final_path = folder.join(&file_name);
    // Keep the raw snapshot out of the destination: it may be a cloud-synced
    // folder, and only the finished archive should be uploaded.
    let snapshot_path = std::env::temp_dir().join(format!(
        "tugboat-snapshot-{}-{stamp}.db",
        std::process::id()
    ));
    let temporary_path = folder.join(format!(".{file_name}.tmp"));

    let outcome = async {
        snapshot_database(pool, &snapshot_path).await?;
        verify_database(&snapshot_path).await?;

        let db_bytes = fs::metadata(&snapshot_path)?.len();
        let db_blake3 = hash_file(&snapshot_path)?;
        let schema = schema_version(pool).await;

        let manifest = BackupManifest {
            format: FORMAT_VERSION,
            created_at: created_at.clone(),
            kind,
            app_version: app_version.to_string(),
            schema_version: schema,
            db_bytes,
            db_blake3,
        };

        write_archive(
            &temporary_path,
            &snapshot_path,
            &manifest,
            &data_dir.join("invoices"),
        )?;
        fs::rename(&temporary_path, &final_path)?;

        let size_bytes = fs::metadata(&final_path)?.len();
        Ok::<_, AppError>(BackupInfo {
            file_name,
            created_at,
            kind,
            size_bytes: size_bytes as f64,
            app_version: app_version.to_string(),
            schema_version: schema as f64,
            is_newer: false,
        })
    }
    .await;

    let _ = fs::remove_file(&snapshot_path);
    let _ = fs::remove_file(&temporary_path);

    let info = outcome?;
    if let Err(error) = prune_backups(&folder, settings.retention) {
        log::warn!("Не вдалося прибрати старі резервні копії: {error}");
    }
    log::info!(
        "Створено резервну копію {} ({kind:?}, {} байт)",
        info.file_name,
        info.size_bytes
    );
    Ok(info)
}

pub async fn create_backup_snapshot(
    pool: &SqlitePool,
    data_dir: &Path,
    app_version: &str,
    app: Option<&AppHandle>,
) -> AppResult<BackupInfo> {
    make_backup(pool, data_dir, app_version, BackupKind::Manual, app).await
}

pub async fn run_due_backup(
    pool: &SqlitePool,
    data_dir: &Path,
    app_version: &str,
    app: Option<&AppHandle>,
) -> AppResult<Option<BackupInfo>> {
    let _guard = BACKUP_LOCK.lock().await;
    let settings = load_settings(data_dir);
    if !settings.enabled {
        return Ok(None);
    }

    let folder = resolve_folder(data_dir, &settings);
    let backups = list_backups_in(&folder)?;

    let due = match backups.first() {
        None => true,
        Some(latest) => match chrono::DateTime::parse_from_rfc3339(&latest.created_at) {
            Ok(created) => {
                let age = Utc::now().signed_duration_since(created.with_timezone(&Utc));
                age.num_hours() >= i64::from(settings.interval_hours)
            }
            Err(_) => true,
        },
    };

    if !due {
        return Ok(None);
    }
    make_backup_tracked(pool, data_dir, app_version, BackupKind::Scheduled, app)
        .await
        .map(Some)
}

pub async fn create_pre_migration(
    pool: &SqlitePool,
    data_dir: &Path,
    app_version: &str,
    app: Option<&AppHandle>,
) -> AppResult<Option<BackupInfo>> {
    let settings = load_settings(data_dir);
    if !settings.enabled {
        return Ok(None);
    }
    let _guard = BACKUP_LOCK.lock().await;
    make_backup_tracked(pool, data_dir, app_version, BackupKind::PreMigration, app)
        .await
        .map(Some)
}

pub async fn has_pending_migrations(pool: &SqlitePool, migrator: &sqlx::migrate::Migrator) -> bool {
    let tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if tables == 0 {
        return false;
    }

    let applied: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations WHERE success = 1",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    migrator
        .migrations
        .iter()
        .any(|migration| migration.version > applied)
}

// ---------------------------------------------------------------------------
// Restore
// ---------------------------------------------------------------------------

fn safe_backup_path(folder: &Path, file_name: &str) -> AppResult<PathBuf> {
    let invalid = file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
        || !file_name.starts_with(BACKUP_PREFIX)
        || !file_name.ends_with(".zip");
    if invalid {
        return Err(AppError::validation("Невірне ім'я резервної копії"));
    }

    let path = folder.join(file_name);
    if !path.is_file() {
        return Err(AppError::NotFound("Резервну копію не знайдено".to_string()));
    }
    Ok(path)
}

pub async fn stage_restore(
    pool: &SqlitePool,
    data_dir: &Path,
    file_name: &str,
    app_version: &str,
    app: Option<&AppHandle>,
) -> AppResult<()> {
    let _guard = BACKUP_LOCK.lock().await;

    let settings = load_settings(data_dir);
    let folder = resolve_folder(data_dir, &settings);
    let path = safe_backup_path(&folder, file_name)?;

    stage_restore_path_locked(pool, data_dir, &path, app_version, app).await
}

pub async fn stage_restore_from_path(
    pool: &SqlitePool,
    data_dir: &Path,
    path: &Path,
    app_version: &str,
    app: Option<&AppHandle>,
) -> AppResult<()> {
    let _guard = BACKUP_LOCK.lock().await;

    if !path.is_file() {
        return Err(AppError::Validation(
            "Це не файл резервної копії Tugboat".to_string(),
        ));
    }

    stage_restore_path_locked(pool, data_dir, path, app_version, app).await
}

async fn stage_restore_path_locked(
    pool: &SqlitePool,
    data_dir: &Path,
    path: &Path,
    app_version: &str,
    app: Option<&AppHandle>,
) -> AppResult<()> {
    let manifest = read_manifest(path)?;

    if manifest.format != FORMAT_VERSION {
        return Err(AppError::Generic(format!(
            "Непідтримуваний формат резервної копії: {}",
            manifest.format
        )));
    }

    let current_schema = schema_version(pool).await;
    if manifest.schema_version > current_schema || version_gt(&manifest.app_version, app_version) {
        return Err(AppError::Generic(
            "Створено новішою версією. Оновіть застосунок.".to_string(),
        ));
    }

    // The safety copy must exist before anything is staged. Its own retention
    // pass never deletes PreRestore archives.
    make_backup_tracked(pool, data_dir, app_version, BackupKind::PreRestore, app).await?;

    let staging = data_dir.join(RESTORE_STAGING);
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir_all(&staging)?;

    if let Err(error) = extract_archive(path, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let restored_database = staging.join(DB_FILE);
    let hash = hash_file(&restored_database)?;
    if hash != manifest.db_blake3 {
        let _ = fs::remove_dir_all(&staging);
        return Err(AppError::Generic(
            "Резервна копія пошкоджена: хеш бази даних не збігається".to_string(),
        ));
    }

    if let Err(error) = verify_database(&restored_database).await {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("backup.zip");
    let marker = data_dir.join(RESTORE_MARKER);
    let payload = serde_json::json!({
        "fileName": file_name,
        "stagedAt": now(),
    });
    fs::write(&marker, serde_json::to_vec_pretty(&payload)?)?;

    log::info!("Резервну копію {file_name} підготовлено до відновлення");
    Ok(())
}

pub async fn apply_pending_restore(data_dir: &Path) -> AppResult<bool> {
    let marker = data_dir.join(RESTORE_MARKER);
    if !marker.is_file() {
        return Ok(false);
    }

    let staging = data_dir.join(RESTORE_STAGING);
    let staged_database = staging.join(DB_FILE);
    if !staged_database.is_file() {
        let _ = fs::remove_file(&marker);
        return Err(AppError::Generic(
            "Не вдалося завершити відновлення: підготовлені дані відсутні".to_string(),
        ));
    }

    // Final check before touching the live data.
    verify_database(&staged_database).await?;

    let previous = data_dir.join(format!("pre-restore-{}", timestamp()));
    fs::create_dir_all(&previous)?;

    for name in ["main.db", "main.db-wal", "main.db-shm"] {
        let source = data_dir.join(name);
        if source.exists() {
            fs::rename(&source, previous.join(name))?;
        }
    }

    let invoices = data_dir.join("invoices");
    if invoices.exists() {
        fs::rename(&invoices, previous.join("invoices"))?;
    }

    if let Err(error) = fs::rename(&staged_database, data_dir.join(DB_FILE)) {
        rollback_restore(&previous, data_dir);
        return Err(AppError::Generic(format!(
            "Не вдалося встановити відновлену базу даних: {error}"
        )));
    }

    let staged_invoices = staging.join("invoices");
    let invoices_result = if staged_invoices.exists() {
        fs::rename(&staged_invoices, data_dir.join("invoices"))
    } else {
        fs::create_dir_all(data_dir.join("invoices"))
    };

    if let Err(error) = invoices_result {
        rollback_restore(&previous, data_dir);
        return Err(AppError::Generic(format!(
            "Не вдалося встановити відновлені документи: {error}"
        )));
    }

    let _ = fs::remove_dir_all(&staging);
    let _ = fs::remove_file(&marker);
    prune_pre_restore(data_dir);

    log::info!(
        "Дані відновлено з резервної копії; попередні дані збережено в {}",
        previous.display()
    );
    Ok(true)
}

fn rollback_restore(previous: &Path, data_dir: &Path) {
    for name in ["main.db", "main.db-wal", "main.db-shm", "invoices"] {
        let source = previous.join(name);
        if !source.exists() {
            continue;
        }

        let target = data_dir.join(name);
        if target.exists() {
            if target.is_dir() {
                let _ = fs::remove_dir_all(&target);
            } else {
                let _ = fs::remove_file(&target);
            }
        }
        let _ = fs::rename(&source, &target);
    }
}

fn prune_pre_restore(data_dir: &Path) {
    let Ok(entries) = fs::read_dir(data_dir) else {
        return;
    };

    let mut folders: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("pre-restore-"))
        })
        .collect();

    folders.sort_by(|left, right| right.file_name().cmp(&left.file_name()));
    for folder in folders.into_iter().skip(2) {
        let _ = fs::remove_dir_all(folder);
    }
}

fn folder_available(folder: &Path) -> bool {
    if fs::create_dir_all(folder).is_err() {
        return false;
    }

    let probe = folder.join(format!(".tugboat-probe-{}", std::process::id()));
    match File::create(&probe) {
        Ok(file) => {
            drop(file);
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn version_gt(candidate: &str, current: &str) -> bool {
    fn parts(value: &str) -> Vec<u64> {
        value
            .trim_start_matches('v')
            .split(['.', '-', '+'])
            .map(|part| {
                part.chars()
                    .take_while(|character| character.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u64>()
                    .unwrap_or(0)
            })
            .collect()
    }

    let left = parts(candidate);
    let right = parts(current);
    let length = left.len().max(right.len()).max(3);

    for index in 0..length {
        let left_part = left.get(index).copied().unwrap_or(0);
        let right_part = right.get(index).copied().unwrap_or(0);
        if left_part != right_part {
            return left_part > right_part;
        }
    }
    false
}

fn info_is_newer(info: &BackupInfo, current_version: &str, current_schema: i64) -> bool {
    info.schema_version > current_schema as f64 || version_gt(&info.app_version, current_version)
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
pub async fn get_backup_overview(state: State<'_, AppState>) -> AppResult<BackupOverview> {
    let settings = load_settings(&state.data_dir);
    let folder = resolve_folder(&state.data_dir, &settings);
    let available = folder_available(&folder);
    let mut backups = list_backups_in(&folder)?;
    let current_schema = schema_version(&state.db_pool).await;
    let current_version = state.version.clone();

    for backup in &mut backups {
        backup.is_newer = info_is_newer(backup, &current_version, current_schema);
    }

    Ok(BackupOverview {
        settings,
        effective_folder: folder.to_string_lossy().into_owned(),
        backups,
        status: current_status(&state.data_dir),
        folder_available: available,
        current_app_version: current_version,
        current_schema_version: current_schema as f64,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn set_backup_settings(
    state: State<'_, AppState>,
    mut settings: BackupSettings,
) -> AppResult<BackupSettings> {
    validate_settings(&settings)?;

    if let Some(folder) = &settings.folder {
        fs::create_dir_all(folder).map_err(|error| {
            AppError::Generic(format!(
                "Не вдалося створити теку резервних копій {folder}: {error}"
            ))
        })?;
    }
    settings.folder = settings
        .folder
        .as_ref()
        .map(|folder| folder.trim().to_string());

    save_settings(&state.data_dir, &settings)?;
    Ok(settings)
}

#[tauri::command]
#[specta::specta]
pub async fn create_backup(app: AppHandle, state: State<'_, AppState>) -> AppResult<BackupInfo> {
    create_backup_snapshot(&state.db_pool, &state.data_dir, &state.version, Some(&app)).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_backups(state: State<'_, AppState>) -> AppResult<Vec<BackupInfo>> {
    let settings = load_settings(&state.data_dir);
    let folder = resolve_folder(&state.data_dir, &settings);
    let mut backups = list_backups_in(&folder)?;
    let current_schema = schema_version(&state.db_pool).await;

    for backup in &mut backups {
        backup.is_newer = info_is_newer(backup, &state.version, current_schema);
    }
    Ok(backups)
}

#[tauri::command]
#[specta::specta]
pub async fn run_scheduled_backup(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<BackupInfo>> {
    run_due_backup(&state.db_pool, &state.data_dir, &state.version, Some(&app)).await
}

#[tauri::command]
#[specta::specta]
pub async fn stage_restore_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    file_name: String,
) -> AppResult<()> {
    stage_restore(
        &state.db_pool,
        &state.data_dir,
        &file_name,
        &state.version,
        Some(&app),
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn stage_restore_file(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> AppResult<()> {
    stage_restore_from_path(
        &state.db_pool,
        &state.data_dir,
        Path::new(&path),
        &state.version,
        Some(&app),
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn inspect_backup_file(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<BackupInfo> {
    let path = PathBuf::from(path);
    let manifest = if path.is_file() {
        read_manifest(&path).ok()
    } else {
        None
    };
    let manifest = manifest
        .ok_or_else(|| AppError::Validation("Це не файл резервної копії Tugboat".to_string()))?;

    if manifest.format != FORMAT_VERSION {
        return Err(AppError::Validation(
            "Це не файл резервної копії Tugboat".to_string(),
        ));
    }

    let current_schema = schema_version(&state.db_pool).await;
    let mut info = BackupInfo {
        file_name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("backup.zip")
            .to_string(),
        created_at: manifest.created_at,
        kind: manifest.kind,
        size_bytes: fs::metadata(&path)
            .map(|meta| meta.len() as f64)
            .unwrap_or(0.0),
        app_version: manifest.app_version,
        schema_version: manifest.schema_version as f64,
        is_newer: false,
    };
    info.is_newer = info_is_newer(&info, &state.version, current_schema);
    Ok(info)
}

#[tauri::command]
#[specta::specta]
pub fn open_backups_dir(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let settings = load_settings(&state.data_dir);
    let folder = resolve_folder(&state.data_dir, &settings);
    fs::create_dir_all(&folder).map_err(|error| {
        AppError::Generic(format!(
            "Не вдалося створити теку резервних копій {}: {error}",
            folder.display()
        ))
    })?;

    app.opener()
        .reveal_item_in_dir(&folder)
        .map_err(|error| AppError::Generic(format!("Не вдалося відкрити теку: {error}")))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "tugboat-backup-test-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn settings_round_trip() {
        let dir = temp_dir();
        let settings = BackupSettings {
            enabled: false,
            folder: None,
            interval_hours: 12,
            retention: 7,
        };
        save_settings(&dir, &settings).expect("save settings");

        let loaded = load_settings(&dir);
        assert!(!loaded.enabled);
        assert_eq!(loaded.interval_hours, 12);
        assert_eq!(loaded.retention, 7);

        let custom = dir.join("cloud");
        let settings = BackupSettings {
            folder: Some(custom.to_string_lossy().into_owned()),
            ..settings
        };
        save_settings(&dir, &settings).expect("save custom folder");
        assert_eq!(load_settings(&dir).folder, settings.folder);

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn backup_stage_and_apply_restore() {
        tauri::async_runtime::block_on(async {
            let dir = temp_dir();
            let db_path = dir.join("main.db");
            let options = SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("database");

            sqlx::query("CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT NOT NULL)")
                .execute(&pool)
                .await
                .expect("table");
            sqlx::query("INSERT INTO notes (body) VALUES ('before')")
                .execute(&pool)
                .await
                .expect("row");

            fs::create_dir_all(dir.join("invoices")).expect("invoices");
            fs::write(dir.join("invoices").join("invoice.txt"), b"invoice").expect("document");

            let backup = create_backup_snapshot(&pool, &dir, "0.1.0", None)
                .await
                .expect("backup");
            assert!(dir.join("backups").join(&backup.file_name).is_file());

            let backups = list_backups_in(&dir.join("backups")).expect("list");
            assert_eq!(backups.len(), 1);
            assert_eq!(backups[0].kind, BackupKind::Manual);
            assert!(!current_status(&dir).running);
            assert!(current_status(&dir).last_error.is_none());

            stage_restore(&pool, &dir, &backup.file_name, "0.1.0", None)
                .await
                .expect("stage");
            assert!(dir.join(RESTORE_MARKER).is_file());
            assert!(dir.join(RESTORE_STAGING).join(DB_FILE).is_file());

            let staged_backups = list_backups_in(&dir.join("backups")).expect("list");
            assert_eq!(staged_backups.len(), 2);
            assert!(staged_backups
                .iter()
                .any(|item| item.kind == BackupKind::PreRestore));

            pool.close().await;
            assert!(apply_pending_restore(&dir).await.expect("apply"));
            assert!(dir.join(DB_FILE).is_file());
            assert!(dir.join("invoices").join("invoice.txt").is_file());
            assert!(!dir.join(RESTORE_MARKER).exists());

            let options = SqliteConnectOptions::new().filename(&db_path);
            let restored = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("restored database");
            let body: String = sqlx::query_scalar("SELECT body FROM notes WHERE id = 1")
                .fetch_one(&restored)
                .await
                .expect("restored row");
            assert_eq!(body, "before");

            restored.close().await;
            fs::remove_dir_all(dir).ok();
        });
    }

    #[test]
    fn compares_versions() {
        assert!(version_gt("0.2.0", "0.1.0"));
        assert!(version_gt("0.1.1", "0.1.0"));
        assert!(version_gt("1.0.0", "0.9.9"));
        assert!(!version_gt("0.1.0", "0.1.0"));
        assert!(!version_gt("0.1.0-beta", "0.1.0"));
        assert!(!version_gt("0.0.9", "0.1.0"));
    }
}
