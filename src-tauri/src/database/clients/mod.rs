pub mod dtos;
pub mod structs;

use structs::*;
use tauri::State;

use crate::{
    database::{
        cars::{dtos::CarDto, fetch_cars_list_by_client_id},
        clients::dtos::ClientDetailsDto,
    },
    error::{AppError, AppResult},
};

#[tauri::command]
#[specta::specta]
pub async fn fetch_clients_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_clients_list(state: State<'_, crate::AppState>) -> AppResult<Vec<Client>> {
    let clients: Vec<Client> =
        sqlx::query_as("SELECT id, name, phone, notes, created_at FROM clients")
            .fetch_all(&state.db_pool)
            .await?;
    Ok(clients)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_client_details_dto(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<ClientDetailsDto> {
    let client: Client = fetch_client_by_id(state.clone(), id).await?;
    let cars: Vec<CarDto> = fetch_cars_list_by_client_id(state.clone(), client.id as i32).await?;
    Ok(ClientDetailsDto { client, cars })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_client_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<Client> {
    let client: Client =
        sqlx::query_as("SELECT id, name, phone, notes, created_at FROM clients WHERE id = $1")
            .bind(id)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn create_client(
    state: State<'_, crate::AppState>,
    payload: CreateClientPayload,
) -> AppResult<Client> {
    let client: Client =
        sqlx::query_as("INSERT INTO clients (name, phone, notes) VALUES ($1, $2, $3) RETURNING *")
            .bind(payload.name)
            .bind(payload.phone)
            .bind(payload.notes)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn update_client(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateClientPayload,
) -> AppResult<Client> {
    let client: Client = sqlx::query_as(
        "UPDATE clients SET name = $2, phone = $3, notes = $4 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.phone)
    .bind(payload.notes)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_client(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Клієнта з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}
