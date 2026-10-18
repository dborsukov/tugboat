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
        workers,
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

fn validate_document_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty()
        || name == "."
        || name == ".."
        || name
            .chars()
            .any(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    {
        return Err(AppError::validation("Невірна назва документа"));
    }
    Ok(())
}

fn work_order_documents_dir(documents_dir: &Path, work_order_id: &str) -> PathBuf {
    documents_dir.join(work_order_id)
}

/// Full path of one generated document, or an error when it is not there.
fn resolve_document_path(
    state: &AppState,
    work_order_id: &str,
    file_name: &str,
) -> AppResult<PathBuf> {
    validate_work_order_id(work_order_id)?;
    validate_document_name(file_name)?;

    let folder = work_order_documents_dir(&invoices_dir(state)?, work_order_id);
    let path = folder.join(file_name);
    if !path.is_file() {
        return Err(AppError::NotFound(format!(
            "Документ «{file_name}» не знайдено"
        )));
    }
    Ok(path)
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
    let documents_dir = invoices_dir(&state)?;
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
    let documents_dir = invoices_dir(&state)?;
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

    // The dialog sends the selected worker; the order's own worker is the
    // fallback so a document can always be generated for an assigned order.
    let performer = if options.skip_performer_signature {
        None
    } else {
        match options.performer_worker_id.or(details.work_order.worker_id) {
            Some(worker_id) => Some(workers::fetch_worker_by_id(state.clone(), worker_id).await?),
            None => None,
        }
    };

    let data = model::build_document_data(
        &details,
        &settings,
        kind,
        &options,
        customer,
        payer,
        performer.as_ref(),
    )?;

    let bytes = match format {
        DocumentFormat::Pdf => pdf::render(&data)?,
        DocumentFormat::Xlsx => xlsx::render(&data)?,
    };

    let documents_dir = invoices_dir(&state)?;
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

#[tauri::command]
#[specta::specta]
pub async fn download_work_order_document(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    work_order_id: String,
    file_name: String,
) -> AppResult<Option<String>> {
    let source = resolve_document_path(&state, &work_order_id, &file_name)?;

    let suggested = file_name.clone();
    let extension = Path::new(&file_name)
        .extension()
        .map(|e| e.to_string_lossy().into_owned());
    let chosen = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;

        let mut builder = app
            .dialog()
            .file()
            .set_title("Зберегти документ")
            .set_file_name(&suggested);
        if let Some(extension) = extension.as_deref() {
            let label = extension.to_uppercase();
            builder = builder.add_filter(&label, &[extension]);
        }

        builder
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
pub async fn delete_work_order_document(
    state: State<'_, AppState>,
    work_order_id: String,
    file_name: String,
) -> AppResult<()> {
    let path = resolve_document_path(&state, &work_order_id, &file_name)?;

    fs::remove_file(&path).map_err(|e| {
        AppError::Generic(format!("Не вдалося видалити файл {}: {e}", path.display()))
    })?;

    log::info!("Документ {} видалено", path.display());

    Ok(())
}
