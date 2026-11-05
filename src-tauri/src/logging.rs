//! Three files live in [`log_dir`]:
//!
//! - `app.log` — lifecycle, warnings, errors, slow operations.
//! - `audit.log` — business events, target [`AUDIT_TARGET`], written by [`Event`].
//! - `panic.log` — crashes, appended by [`install_panic_hook`] without goingnthrough the logger.

use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use tauri::{AppHandle, State};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

use crate::{
    error::{AppError, AppResult},
    AppState,
};

/// Must match `identifier` in `tauri.conf.json`.
const IDENTIFIER: &str = "com.popeye.crm";

/// Target used by [`Event`].
pub const AUDIT_TARGET: &str = "audit";

/// The folder the log files live in.
/// Deliberately computed from `dirs` in case app panics.
pub fn log_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(IDENTIFIER)
        .join("logs")
}

/// `POPEYE_LOG=trace|debug|info|warn|error|off` overrides the built-in default.
fn requested_level() -> log::LevelFilter {
    use log::LevelFilter;

    match std::env::var("POPEYE_LOG").ok().as_deref() {
        Some("trace") => LevelFilter::Trace,
        Some("debug") => LevelFilter::Debug,
        Some("info") => LevelFilter::Info,
        Some("warn") => LevelFilter::Warn,
        Some("error") => LevelFilter::Error,
        Some("off") => LevelFilter::Off,
        Some(other) => {
            // logger is not registered yet
            eprintln!("POPEYE_LOG={other:?} is not a log level, using the default");
            default_level()
        }
        None => default_level(),
    }
}

fn default_level() -> log::LevelFilter {
    if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    }
}

fn is_audit(metadata: &log::Metadata<'_>) -> bool {
    let target = metadata.target();
    target == AUDIT_TARGET || target.starts_with("audit::")
}

/// Reports panics to `panic.log` and to the log file, then defers to the
/// previous hook so the console output of a dev run is unchanged.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("<unnamed>");
        let location = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|message| (*message).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());

        let message = format!("[PANIC] thread={name} at={location} payload={payload}");

        // A no-op when no logger is registered yet, and a line in `app.log` once one is.
        log::error!("{message}");

        // Written directly as well since the logger may not exist yet.
        let dir = log_dir();
        if std::fs::create_dir_all(&dir).is_ok() {
            let path = dir.join("panic.log");
            cap_file(&path, PANIC_LOG_MAX_BYTES);
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
                let _ = writeln!(file, "{} {message}", chrono::Local::now().to_rfc3339());
            }
        }

        previous(info);
    }));
}

const PANIC_LOG_MAX_BYTES: u64 = 1_000_000;

fn cap_file(path: &Path, max_bytes: u64) {
    let too_large = std::fs::metadata(path)
        .map(|meta| meta.len() > max_bytes)
        .unwrap_or(false);
    if !too_large {
        return;
    }

    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let _ = std::fs::rename(path, path.with_file_name(format!("{name}.1")));
}

/// Registers the logger. Call once, from setup.
pub fn init(app: &AppHandle, dir: &Path) -> tauri::Result<()> {
    use log::LevelFilter;

    let level = requested_level();

    // Always log audit target, unless turned off.
    let audit_level = match level {
        LevelFilter::Off => LevelFilter::Off,
        _ => LevelFilter::Info,
    };

    let builder = tauri_plugin_log::Builder::new()
        .clear_targets()
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}][{}] {message}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level(),
            ))
        })
        .level(level)
        .level_for(AUDIT_TARGET, audit_level)
        .level_for("sqlx::query", LevelFilter::Warn)
        .level_for("sqlx::pool", LevelFilter::Warn)
        // Kept at debug deliberately to catch certain webview errors
        .level_for("tauri_runtime_wry", LevelFilter::Debug)
        .level_for("hyper", LevelFilter::Warn)
        // Everything the webview sends through `@tauri-apps/plugin-log`.
        .level_for("webview", LevelFilter::Info)
        .rotation_strategy(RotationStrategy::KeepSome(20))
        .max_file_size(2_000_000)
        .target(
            Target::new(TargetKind::Folder {
                path: dir.to_path_buf(),
                file_name: Some("audit".into()),
            })
            .filter(is_audit),
        )
        .target(
            Target::new(TargetKind::Folder {
                path: dir.to_path_buf(),
                file_name: Some("app".into()),
            })
            .filter(|metadata| !is_audit(metadata)),
        );

    #[cfg(debug_assertions)]
    let builder = builder.target(Target::new(TargetKind::Stdout));

    app.plugin(builder.build())
}

/// A greppable business event: `client.create id=42`.
pub struct Event {
    action: &'static str,
    fields: Vec<String>,
}

fn needs_quoting(value: &str) -> bool {
    value.is_empty()
        || value
            .chars()
            .any(|character| character.is_whitespace() || character == '"' || character == '=')
}

impl Event {
    pub fn new(action: &'static str) -> Self {
        Self {
            action,
            fields: Vec::new(),
        }
    }

    /// The record the event is about.
    pub fn id(self, id: impl std::fmt::Display) -> Self {
        self.field("id", id)
    }

    /// Adds a `key=value` pair.
    pub fn field(mut self, key: &str, value: impl std::fmt::Display) -> Self {
        let value = value.to_string();
        let value = if needs_quoting(&value) {
            format!("{value:?}")
        } else {
            value
        };
        self.fields.push(format!("{key}={value}"));
        self
    }

    /// A free-text value. Kept for intent, it documents that the value is not a
    /// number or an identifier, even though [`Event::field`] quotes it the same way.
    pub fn text(self, key: &str, value: &str) -> Self {
        self.field(key, value)
    }

    /// `action key=value key=value`, without any outcome suffix.
    fn line(&self) -> String {
        if self.fields.is_empty() {
            self.action.to_string()
        } else {
            format!("{} {}", self.action, self.fields.join(" "))
        }
    }

    /// The change was made and committed.
    pub fn ok(self) {
        log::info!(target: AUDIT_TARGET, "{}", self.line());
    }

    /// The action was refused on purpose (locked VAT, archived record, …).
    pub fn denied(self, reason: &str) {
        log::warn!(target: AUDIT_TARGET, "{} refused={reason}", self.line());
    }

    /// The action failed; `error` carries the reason for the user.
    pub fn failed(self, error: &AppError) {
        log::error!(target: AUDIT_TARGET, "{} failed={error:?}", self.line());
    }

    /// Records a failure and passes the result through
    pub fn on_error<T>(self, result: AppResult<T>) -> AppResult<T> {
        if let Err(error) = &result {
            self.failed(error);
        }
        result
    }
}

#[tauri::command]
#[specta::specta]
pub fn open_logs_dir(app: AppHandle) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;

    let dir = log_dir();
    std::fs::create_dir_all(&dir).map_err(|error| {
        AppError::Generic(format!(
            "Не вдалося створити теку журналів {}: {error}",
            dir.display()
        ))
    })?;

    app.opener()
        .reveal_item_in_dir(&dir)
        .map_err(|error| AppError::Generic(format!("Не вдалося відкрити теку журналів: {error}")))
}

#[tauri::command]
#[specta::specta]
pub fn log_diagnostics(state: State<'_, AppState>) -> AppResult<String> {
    const AUDIT_TAIL_LINES: usize = 100;
    const APP_TAIL_LINES: usize = 250;

    let dir = log_dir();
    let app = dir.join("app.log");
    let audit = dir.join("audit.log");
    let panic = dir.join("panic.log");

    let describe = |path: &Path| match std::fs::metadata(path) {
        Ok(meta) => format!("{}: {} B", path.display(), meta.len()),
        Err(_) => format!("{}: відсутній", path.display()),
    };

    let mut report = format!(
        "Папай CRM {version}\nbuild={build}\nos={os} arch={arch}\nдані={data}\nжурнали={logs}\n{app}\n{audit}\n{panic}\nротацій: {rotated}\n",
        version = state.version,
        build = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        data = state.data_dir.display(),
        logs = dir.display(),
        app = describe(&app),
        audit = describe(&audit),
        panic = describe(&panic),
        rotated = rotated_count(&dir),
    );

    push_tail(&mut report, "app.log", &app, APP_TAIL_LINES);
    push_tail(&mut report, "audit.log", &audit, AUDIT_TAIL_LINES);
    push_tail(&mut report, "panic.log", &panic, 20);

    Ok(report)
}

fn push_tail(report: &mut String, label: &str, path: &Path, max_lines: usize) {
    let tail = tail_of(path, 128 * 1024, max_lines);
    if tail.is_empty() {
        return;
    }
    report.push_str(&format!("\n--- {label} (кінець) ---\n{tail}\n"));
}

fn rotated_count(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };

    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            (name.starts_with("app_") || name.starts_with("audit_")) && name.ends_with(".log")
        })
        .count()
}

fn tail_of(path: &Path, max_bytes: u64, max_lines: usize) -> String {
    let Ok(meta) = std::fs::metadata(path) else {
        return String::new();
    };

    let start = meta.len().saturating_sub(max_bytes);
    let Ok(mut file) = std::fs::File::open(path) else {
        return String::new();
    };
    if file.seek(SeekFrom::Start(start)).is_err() {
        return String::new();
    }

    let mut buffer = String::new();
    if file.take(max_bytes).read_to_string(&mut buffer).is_err() {
        return String::new();
    }

    if start > 0 {
        if let Some(newline) = buffer.find('\n') {
            buffer.drain(..=newline);
        }
    }

    let lines: Vec<&str> = buffer.lines().collect();
    let from = lines.len().saturating_sub(max_lines);

    lines[from..].join("\n")
}
