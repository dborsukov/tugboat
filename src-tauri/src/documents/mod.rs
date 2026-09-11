pub mod model;
pub mod pdf;
pub mod xlsx;

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
pub use model::{
    DocumentConflictAction, DocumentFormat, DocumentKind, GenerateDocumentOptions,
    WorkOrderDocument, WorkOrderDocuments,
};
use tauri::State;

use crate::{
    database::{
        clients, settings::fetch_settings_from_pool, work_orders::fetch_work_order_details_dto,
    },
    error::{AppError, AppResult},
    AppState,
};

/// The documents folder: the user-configured one, or `[app_dir]/documents`.
pub(crate) async fn resolve_documents_dir(state: &AppState) -> AppResult<PathBuf> {
    let saved: Option<String> =
        sqlx::query_scalar("SELECT documents_dir FROM app_settings WHERE id = 1")
            .fetch_one(&state.db_pool)
            .await?;
    let dir = match saved.as_deref().map(str::trim) {
        None | Some("") => state.data_dir.join("documents"),
        Some(path) => PathBuf::from(path),
    };
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку документів {}: {e}",
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

fn document_file_name(kind: DocumentKind, work_order_id: &str, format: DocumentFormat) -> String {
    format!("{}_{}.{}", kind.title(), work_order_id, format.extension())
}

fn resolve_document_target_path(
    documents_dir: &Path,
    work_order_id: &str,
    kind: DocumentKind,
    format: DocumentFormat,
) -> PathBuf {
    work_order_documents_dir(documents_dir, work_order_id).join(document_file_name(
        kind,
        work_order_id,
        format,
    ))
}

/// First free `name (n).ext` path starting from n = 2.
fn next_free_path(target: &Path) -> PathBuf {
    let parent = target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = target
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());
    let extension = target.extension().map(|e| e.to_string_lossy().into_owned());

    let mut counter = 2u32;
    loop {
        let name = match &extension {
            Some(ext) => format!("{stem} ({counter}).{ext}"),
            None => format!("{stem} ({counter})"),
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

fn document_from_path(path: &Path) -> AppResult<WorkOrderDocument> {
    let metadata = fs::metadata(path).map_err(|e| {
        AppError::Generic(format!("Не вдалося прочитати файл {}: {e}", path.display()))
    })?;
    Ok(WorkOrderDocument {
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: path.to_string_lossy().into_owned(),
        size: metadata.len() as i64,
        modified_at: metadata
            .modified()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now()),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_documents(
    state: State<'_, AppState>,
    work_order_id: String,
) -> AppResult<WorkOrderDocuments> {
    validate_work_order_id(&work_order_id)?;
    let documents_dir = resolve_documents_dir(&state).await?;
    let folder = work_order_documents_dir(&documents_dir, &work_order_id);

    // Make sure documents folder is created as soon as work order is opened
    fs::create_dir_all(&folder).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку документів {}: {e}",
            folder.display()
        ))
    })?;

    let mut files = Vec::new();
    for entry in fs::read_dir(&folder)
        .map_err(|e| AppError::Generic(format!("Не вдалося прочитати теку документів: {e}")))?
    {
        let entry = entry
            .map_err(|e| AppError::Generic(format!("Не вдалося прочитати теку документів: {e}")))?;
        let path = entry.path();
        if path.is_file() {
            files.push(document_from_path(&path)?);
        }
    }

    files.sort_by_key(|document| std::cmp::Reverse(document.modified_at));

    Ok(WorkOrderDocuments {
        folder_path: folder.to_string_lossy().into_owned(),
        files,
    })
}

/// Info about the effective documents folder, used by the settings page.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentsDirInfo {
    pub path: String,
    pub has_content: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_documents_dir_info(state: State<'_, AppState>) -> AppResult<DocumentsDirInfo> {
    let dir = resolve_documents_dir(&state).await?;
    let has_content = fs::read_dir(&dir)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false);
    Ok(DocumentsDirInfo {
        path: dir.to_string_lossy().into_owned(),
        has_content,
    })
}

/// Returns `Some(target path)` when a file with the default document name
/// already exists, `None` otherwise. Used to show the replace/rename dialog.
#[tauri::command]
#[specta::specta]
pub async fn document_target_path(
    state: State<'_, AppState>,
    work_order_id: String,
    kind: DocumentKind,
    format: DocumentFormat,
) -> AppResult<Option<String>> {
    validate_work_order_id(&work_order_id)?;
    let documents_dir = resolve_documents_dir(&state).await?;
    let target = resolve_document_target_path(&documents_dir, &work_order_id, kind, format);
    Ok(target
        .exists()
        .then(|| target.to_string_lossy().into_owned()))
}

#[tauri::command]
#[specta::specta]
pub async fn generate_document(
    state: State<'_, AppState>,
    work_order_id: String,
    kind: DocumentKind,
    format: DocumentFormat,
    options: GenerateDocumentOptions,
    conflict: DocumentConflictAction,
) -> AppResult<WorkOrderDocument> {
    validate_work_order_id(&work_order_id)?;

    let details = fetch_work_order_details_dto(state.clone(), work_order_id.clone()).await?;
    if details.work_order.archived_at.is_some() {
        return Err(AppError::work_order_archived());
    }

    let settings = fetch_settings_from_pool(&state.db_pool).await?;
    if settings.company_info.is_none() {
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
        }
    }

    let extra_client_ids: Vec<i64> = [
        details.work_order.customer_client_id,
        details.work_order.payer_client_id,
    ]
    .into_iter()
    .flatten()
    .map(i64::from)
    .collect();
    let extra_clients = clients::fetch_clients_by_ids(&state, &extra_client_ids).await?;
    let extra_client_details =
        clients::fetch_client_details_by_ids(&state, &extra_client_ids).await?;
    let doc_client = |id: Option<i32>| -> Option<model::DocClient> {
        let id = i64::from(id?);
        let client = extra_clients.get(&id)?;
        Some(model::DocClient::from_parts(
            client,
            extra_client_details.get(&id),
        ))
    };
    let customer = doc_client(details.work_order.customer_client_id);
    let payer = doc_client(details.work_order.payer_client_id);

    let data = model::build_document_data(&details, &settings, kind, &options, customer, payer)?;

    let bytes = match format {
        DocumentFormat::Pdf => pdf::render(&data)?,
        DocumentFormat::Xlsx => xlsx::render(&data)?,
    };

    let documents_dir = resolve_documents_dir(&state).await?;
    let folder = work_order_documents_dir(&documents_dir, &work_order_id);
    fs::create_dir_all(&folder).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку {}: {e}",
            folder.display()
        ))
    })?;

    let target = resolve_document_target_path(&documents_dir, &work_order_id, kind, format);
    let path = match conflict {
        DocumentConflictAction::Replace => target,
        DocumentConflictAction::Rename => {
            if target.exists() {
                next_free_path(&target)
            } else {
                target
            }
        }
    };

    fs::write(&path, bytes).map_err(|e| {
        AppError::Generic(format!("Не вдалося зберегти файл {}: {e}", path.display()))
    })?;

    document_from_path(&path)
}

/// Opens the OS folder picker. Runs on the main thread (sync command).
#[tauri::command]
#[specta::specta]
pub fn pick_documents_folder(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_folder()?;
    picked
        .into_path()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Saves the documents folder setting. When `move_documents` is true and the
/// old folder still has documents, its contents are moved into the new one.
#[tauri::command]
#[specta::specta]
pub async fn update_documents_dir(
    state: State<'_, AppState>,
    path: String,
    move_documents: bool,
) -> AppResult<String> {
    let new_dir = PathBuf::from(path.trim());
    if new_dir.as_os_str().is_empty() || !new_dir.is_absolute() {
        return Err(AppError::validation(
            "Вкажіть правильний абсолютний шлях до теки документів",
        ));
    }

    let old_dir = resolve_documents_dir(&state).await?;
    let new_dir = if new_dir == old_dir {
        old_dir.clone()
    } else {
        new_dir
    };

    fs::create_dir_all(&new_dir).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку {}: {e}",
            new_dir.display()
        ))
    })?;

    if move_documents && old_dir != new_dir && old_dir.exists() {
        for entry in fs::read_dir(&old_dir).map_err(|e| {
            AppError::Generic(format!(
                "Не вдалося прочитати теку {}: {e}",
                old_dir.display()
            ))
        })? {
            let entry = entry.map_err(|e| {
                AppError::Generic(format!(
                    "Не вдалося прочитати теку {}: {e}",
                    old_dir.display()
                ))
            })?;
            let source = entry.path();
            let mut destination = new_dir.join(entry.file_name());
            if destination.exists() {
                destination = next_free_path(&destination);
            }
            fs::rename(&source, &destination).map_err(|e| {
                AppError::Generic(format!(
                    "Не вдалося перенести {} у {}: {e}",
                    source.display(),
                    destination.display()
                ))
            })?;
        }
        // Remove old folder if empty
        let _ = fs::remove_dir(&old_dir);
    }

    let saved: String = sqlx::query_scalar(
        "UPDATE app_settings SET documents_dir = $1 WHERE id = 1 RETURNING documents_dir",
    )
    .bind(new_dir.to_string_lossy().into_owned())
    .fetch_one(&state.db_pool)
    .await?;

    Ok(saved)
}
