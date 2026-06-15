use tauri::State;

pub mod structs;

use structs::*;

#[tauri::command]
#[specta::specta]
pub async fn fetch_clients(state: State<'_, crate::AppState>) -> Result<Vec<Client>, ()> {
    let clients: Vec<Client> = sqlx::query_as("SELECT id, name, phone, created_at FROM clients")
        .fetch_all(&state.db_pool)
        .await
        .unwrap();
    Ok(clients)
}

#[tauri::command]
#[specta::specta]
pub async fn create_client(
    state: State<'_, crate::AppState>,
    payload: CreateClientPayload,
) -> Result<Client, String> {
    let client: Client =
        sqlx::query_as("INSERT INTO clients (name, phone) VALUES ($1, $2) RETURNING *")
            .bind(payload.name)
            .bind(payload.phone)
            .fetch_one(&state.db_pool)
            .await
            .map_err(|e| e.to_string())?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn update_client(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateClientPayload,
) -> Result<Client, String> {
    let client: Client = sqlx::query_as("UPDATE clients SET name = $1, phone = $2 WHERE id = $3 RETURNING id, name, phone, created_at")
        .bind(payload.name)
        .bind(payload.phone)
        .bind(id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_client(state: State<'_, crate::AppState>, id: i32) -> Result<(), String> {
    sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
