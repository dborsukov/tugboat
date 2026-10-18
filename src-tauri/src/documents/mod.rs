pub mod hash;
pub mod model;
pub mod pdf;
pub mod xlsx;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
pub use model::{
    DocumentFormat, DocumentKind, GenerateDocumentOptions, GenerateDocumentOutcome,
    WorkOrderDocument, WorkOrderDocumentFile, WorkOrderDocumentVersion, WorkOrderDocuments,
};
use sqlx::SqlitePool;
use tauri::State;

use crate::{
    database::{
        clients,
        clients::structs::{Client, ClientDetails},
        settings::{fetch_settings_from_pool, structs::AppSettings},
        work_orders::{dtos::WorkOrderDetailsDto, fetch_work_order_details_dto},
        workers,
        workers::structs::Worker,
    },
    error::{AppError, AppResult},
    AppState,
};

/// The folder generated documents live in, always `[app_dir]/invoices`.
pub(crate) fn invoices_dir(state: &AppState) -> AppResult<PathBuf> {
    let dir = state.data_dir.join("invoices");
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку рахунків {}: {e}",
            dir.display()
        ))
    })?;
    Ok(dir)
}

fn validate_work_order_id(work_order_id: &str) -> AppResult<()> {
    if work_order_id.trim().is_empty()
        || work_order_id == "."
        || work_order_id == ".."
        || work_order_id
            .chars()
            .any(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    {
        return Err(AppError::validation(
            "Невірний ідентифікатор наряд-замовлення",
        ));
    }
    Ok(())
}

fn work_order_documents_dir(documents_dir: &Path, work_order_id: &str) -> PathBuf {
    documents_dir.join(work_order_id)
}

/// Versions of one kind share a folder, so a single document's history is
/// together and a file name only has to be unique within it.
fn kind_documents_dir(documents_dir: &Path, work_order_id: &str, kind: DocumentKind) -> PathBuf {
    work_order_documents_dir(documents_dir, work_order_id).join(kind.as_str())
}

/// A row of `work_order_document_versions`. One row is one edition of a
/// document, holding both of its files.
#[derive(sqlx::FromRow)]
struct VersionRow {
    id: i64,
    document_kind: DocumentKind,
    pdf_path: String,
    xlsx_path: String,
    source_hash: String,
    options: String,
    generated_at: DateTime<Utc>,
}

const VERSION_COLUMNS: &str =
    "id, document_kind, pdf_path, xlsx_path, source_hash, options, generated_at";

/// The choices a version was generated with. Rows are written by this app, so a
/// failure here is a damaged row: fall back to empty choices rather than taking
/// the whole page down. A version rebuilt from them is then reported as stale.
fn decode_options(raw: &str) -> GenerateDocumentOptions {
    serde_json::from_str(raw).unwrap_or_else(|error| {
        log::warn!("Не вдалося прочитати параметри документа: {error}");
        GenerateDocumentOptions::default()
    })
}

/// Both files are written by every generation, so a version missing either of
/// them is not current and has to be written again.
fn version_files_present(row: &VersionRow) -> bool {
    Path::new(&row.pdf_path).is_file() && Path::new(&row.xlsx_path).is_file()
}

fn document_file(format: DocumentFormat, path: &str) -> WorkOrderDocumentFile {
    WorkOrderDocumentFile {
        format,
        file_name: Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        file_size: fs::metadata(path).ok().map(|meta| meta.len() as i64),
    }
}

fn version_files(row: &VersionRow) -> Vec<WorkOrderDocumentFile> {
    vec![
        document_file(DocumentFormat::Pdf, &row.pdf_path),
        document_file(DocumentFormat::Xlsx, &row.xlsx_path),
    ]
}

fn version_from_row(
    row: &VersionRow,
    options: GenerateDocumentOptions,
    is_stale: bool,
) -> WorkOrderDocumentVersion {
    WorkOrderDocumentVersion {
        id: row.id,
        kind: row.document_kind,
        generated_at: row.generated_at,
        options,
        files: version_files(row),
        is_stale,
    }
}

async fn version_by_id(state: &State<'_, AppState>, id: i64) -> AppResult<VersionRow> {
    let row: Option<VersionRow> = sqlx::query_as(&format!(
        "SELECT {VERSION_COLUMNS} FROM work_order_document_versions WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?;

    row.ok_or_else(|| AppError::NotFound("Документ не знайдено".to_string()))
}

async fn latest_version(
    pool: &SqlitePool,
    work_order_id: &str,
    kind: DocumentKind,
) -> AppResult<Option<VersionRow>> {
    let row: Option<VersionRow> = sqlx::query_as(&format!(
        "SELECT {VERSION_COLUMNS} FROM work_order_document_versions
         WHERE work_order_id = $1 AND document_kind = $2
         ORDER BY generated_at DESC, id DESC
         LIMIT 1"
    ))
    .bind(work_order_id)
    .bind(kind)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

struct DocumentContext {
    details: WorkOrderDetailsDto,
    settings: AppSettings,
    clients: HashMap<i64, Client>,
    client_details: HashMap<i64, ClientDetails>,
    workers: HashMap<i64, Worker>,
}

impl DocumentContext {
    async fn load(state: &State<'_, AppState>, work_order_id: &str) -> AppResult<Self> {
        let details =
            fetch_work_order_details_dto(state.clone(), work_order_id.to_string()).await?;
        let settings = fetch_settings_from_pool(&state.db_pool).await?;

        let extra_client_ids: Vec<i64> = [
            details.work_order.customer_client_id,
            details.work_order.payer_client_id,
        ]
        .into_iter()
        .flatten()
        .map(i64::from)
        .collect();
        let clients = clients::fetch_clients_by_ids(state, &extra_client_ids).await?;
        let client_details = clients::fetch_client_details_by_ids(state, &extra_client_ids).await?;
        let workers = workers::fetch_workers_list(state.clone())
            .await?
            .into_iter()
            .map(|worker| (worker.id, worker))
            .collect();

        Ok(Self {
            details,
            settings,
            clients,
            client_details,
            workers,
        })
    }

    fn extra_client(&self, id: Option<i32>) -> Option<model::DocClient> {
        let id = i64::from(id?);
        let client = self.clients.get(&id)?;
        Some(model::DocClient::from_parts(
            client,
            self.client_details.get(&id),
        ))
    }

    fn data(
        &self,
        kind: DocumentKind,
        options: &GenerateDocumentOptions,
    ) -> AppResult<model::DocumentData> {
        // The performer the document is signed by: the one chosen for this
        // generation, falling back to the order's own worker.
        let performer = if options.skip_performer_signature {
            None
        } else {
            options
                .performer_worker_id
                .or(self.details.work_order.worker_id)
                .and_then(|id| self.workers.get(&i64::from(id)))
        };

        model::build_document_data(
            &self.details,
            &self.settings,
            kind,
            options,
            self.extra_client(self.details.work_order.customer_client_id),
            self.extra_client(self.details.work_order.payer_client_id),
            performer,
        )
    }
}

/// A first free timestamp name, both extensions checked.
fn unique_base_name(folder: &Path, at: DateTime<Utc>) -> String {
    let base = at.format("%Y%m%dT%H%M%SZ").to_string();
    let taken = |name: &str| {
        folder.join(format!("{name}.pdf")).exists() || folder.join(format!("{name}.xlsx")).exists()
    };

    if !taken(&base) {
        return base;
    }

    let mut counter = 2u32;
    loop {
        let candidate = format!("{base}_{counter}");
        if !taken(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

fn remove_files(paths: &[&Path]) {
    for path in paths {
        if let Err(error) = fs::remove_file(path) {
            log::warn!("Не вдалося прибрати файл {}: {error}", path.display());
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// The current document of each kind for a work order, with its older versions.
#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_documents(
    state: State<'_, AppState>,
    work_order_id: String,
) -> AppResult<WorkOrderDocuments> {
    validate_work_order_id(&work_order_id)?;

    let rows: Vec<VersionRow> = sqlx::query_as(&format!(
        "SELECT {VERSION_COLUMNS} FROM work_order_document_versions
         WHERE work_order_id = $1
         ORDER BY generated_at DESC, id DESC"
    ))
    .bind(&work_order_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Rebuilding a version needs today's records, loaded once for the whole page.
    let context = if rows.is_empty() {
        None
    } else {
        match DocumentContext::load(&state, &work_order_id).await {
            Ok(context) => Some(context),
            Err(error) => {
                log::warn!(
                    "Не вдалося перевірити актуальність документів {work_order_id}: {error}"
                );
                None
            }
        }
    };

    let mut by_kind: HashMap<DocumentKind, Vec<WorkOrderDocumentVersion>> = HashMap::new();
    for row in rows {
        let options = decode_options(&row.options);
        let is_stale = match &context {
            Some(context) => context
                .data(row.document_kind, &options)
                .map(|data| hash::document_hash(&data) != row.source_hash)
                .unwrap_or(true),
            None => false,
        };
        by_kind
            .entry(row.document_kind)
            .or_default()
            .push(version_from_row(&row, options, is_stale));
    }

    let documents = DocumentKind::ALL
        .into_iter()
        .map(|kind| WorkOrderDocument {
            kind,
            title: kind.title().to_string(),
            versions: by_kind.remove(&kind).unwrap_or_default(),
        })
        .collect();

    Ok(WorkOrderDocuments { documents })
}

/// Writes a new version of `kind`, both formats, and records it.
#[tauri::command]
#[specta::specta]
pub async fn generate_document(
    state: State<'_, AppState>,
    work_order_id: String,
    kind: DocumentKind,
    options: GenerateDocumentOptions,
) -> AppResult<GenerateDocumentOutcome> {
    validate_work_order_id(&work_order_id)?;

    let context = DocumentContext::load(&state, &work_order_id).await?;
    if context.details.work_order.archived_at.is_some() {
        return Err(AppError::work_order_archived());
    }
    if context.settings.company_info.is_none() {
        return Err(AppError::validation(
            "Спочатку заповніть дані бізнесу в налаштуваннях",
        ));
    }

    if kind == DocumentKind::AktVykonanykhRobit {
        if let Some(odometer_out) = options.odometer_out {
            sqlx::query("UPDATE work_orders SET odometer_out = $1 WHERE id = $2")
                .bind(odometer_out)
                .bind(&work_order_id)
                .execute(&state.db_pool)
                .await?;
            // The odometer is not printed, so the loaded details stay correct.
        }
    }

    let data = context.data(kind, &options)?;
    let source_hash = hash::document_hash(&data);

    if let Some(latest) = latest_version(&state.db_pool, &work_order_id, kind).await? {
        if latest.source_hash == source_hash && version_files_present(&latest) {
            log::info!(
                "Документ {} {} не змінився, нову версію не створено",
                kind.title(),
                work_order_id
            );
            return Ok(GenerateDocumentOutcome {
                created: false,
                version: version_from_row(&latest, decode_options(&latest.options), false),
            });
        }
    }

    let pdf_bytes = pdf::render(&data)?;
    let xlsx_bytes = xlsx::render(&data)?;

    let folder = kind_documents_dir(&invoices_dir(&state)?, &work_order_id, kind);
    fs::create_dir_all(&folder).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку {}: {e}",
            folder.display()
        ))
    })?;

    let generated_at = Utc::now();
    let base_name = unique_base_name(&folder, generated_at);
    let pdf_path = folder.join(format!("{base_name}.pdf"));
    let xlsx_path = folder.join(format!("{base_name}.xlsx"));

    if let Err(error) =
        fs::write(&pdf_path, &pdf_bytes).and_then(|()| fs::write(&xlsx_path, &xlsx_bytes))
    {
        remove_files(&[&pdf_path, &xlsx_path]);
        return Err(AppError::Generic(format!(
            "Не вдалося зберегти документ {}: {error}",
            pdf_path.display()
        )));
    }

    let options_json = serde_json::to_string(&options)?;
    let inserted: Result<VersionRow, sqlx::Error> = sqlx::query_as(&format!(
        "INSERT INTO work_order_document_versions
            (work_order_id, document_kind, pdf_path, xlsx_path, source_hash, options, generated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING {VERSION_COLUMNS}"
    ))
    .bind(&work_order_id)
    .bind(kind)
    .bind(pdf_path.to_string_lossy().into_owned())
    .bind(xlsx_path.to_string_lossy().into_owned())
    .bind(&source_hash)
    .bind(&options_json)
    .bind(generated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    .fetch_one(&state.db_pool)
    .await;

    let row = match inserted {
        Ok(row) => row,
        Err(error) => {
            // No row, no version: the files must not outlive the record.
            remove_files(&[&pdf_path, &xlsx_path]);
            return Err(error.into());
        }
    };

    log::info!(
        "Документ {} {} збережено до {}",
        kind.title(),
        work_order_id,
        folder.display()
    );

    Ok(GenerateDocumentOutcome {
        created: true,
        version: version_from_row(&row, options, false),
    })
}

/// Copies one file of a version to a location the user picks.
#[tauri::command]
#[specta::specta]
pub async fn download_work_order_document(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: i32,
    format: DocumentFormat,
    file_name: String,
) -> AppResult<Option<String>> {
    let row = version_by_id(&state, i64::from(id)).await?;
    let path = match format {
        DocumentFormat::Pdf => row.pdf_path,
        DocumentFormat::Xlsx => row.xlsx_path,
    };

    let source = PathBuf::from(&path);
    let root = invoices_dir(&state)?;
    if !source.starts_with(&root) || !source.is_file() {
        return Err(AppError::NotFound(
            "Файл документа не знайдено — створіть документ ще раз".to_string(),
        ));
    }

    let fallback = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("document.{}", format.extension()));
    let suggested = crate::utils::suggested_file_name(&file_name, &fallback);
    let extension = format.extension().to_string();

    let chosen = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;

        app.dialog()
            .file()
            .set_title("Зберегти документ")
            .set_file_name(&suggested)
            .add_filter(extension.to_uppercase(), &[extension.as_str()])
            .blocking_save_file()
            .and_then(|file| file.into_path().ok())
    })
    .await
    .map_err(|e| AppError::Generic(format!("Не вдалося відкрити вікно збереження: {e}")))?;

    let Some(destination) = chosen else {
        return Ok(None);
    };

    fs::copy(&source, &destination).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося зберегти файл {}: {e}",
            destination.display()
        ))
    })?;

    log::info!(
        "Документ {} збережено до {}",
        source.display(),
        destination.display()
    );

    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
#[specta::specta]
pub async fn open_work_order_document(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: i32,
    format: DocumentFormat,
) -> AppResult<()> {
    let row = version_by_id(&state, i64::from(id)).await?;
    let path = match format {
        DocumentFormat::Pdf => row.pdf_path,
        DocumentFormat::Xlsx => row.xlsx_path,
    };
    let path = PathBuf::from(path);
    let root = invoices_dir(&state)?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err(AppError::NotFound(
            "Файл документа не знайдено — створіть документ ще раз".to_string(),
        ));
    }

    use tauri_plugin_opener::OpenerExt;

    app.opener()
        .open_path(path.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|e| AppError::Generic(format!("Не вдалося відкрити файл: {e}")))
}
