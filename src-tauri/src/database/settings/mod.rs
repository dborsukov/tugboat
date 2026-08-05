pub mod structs;

use structs::*;
use tauri::State;

use crate::error::AppResult;

#[tauri::command]
#[specta::specta]
pub async fn fetch_settings(state: State<'_, crate::AppState>) -> AppResult<AppSettings> {
    let settings: AppSettings = sqlx::query_as("SELECT * FROM app_settings WHERE id = 1")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(settings)
}

#[tauri::command]
#[specta::specta]
pub async fn update_settings(
    settings: AppSettings,
    state: tauri::State<'_, crate::AppState>,
) -> AppResult<AppSettings> {
    let settings: AppSettings =
        sqlx::query_as("UPDATE app_settings SET theme = $1 WHERE id = 1 RETURNING *")
            .bind(settings.theme)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(settings)
}
