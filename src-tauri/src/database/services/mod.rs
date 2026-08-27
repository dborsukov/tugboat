pub mod structs;

use structs::*;
use tauri::State;

use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub async fn fetch_services_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_services_list(state: State<'_, crate::AppState>) -> AppResult<Vec<Service>> {
    let services: Vec<Service> =
        sqlx::query_as("SELECT id, code, name, price_type, fixed_price, labor_hours FROM services")
            .fetch_all(&state.db_pool)
            .await?;
    Ok(services)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_service_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<Service> {
    let service: Service = sqlx::query_as(
        "SELECT id, code, name, price_type, fixed_price, labor_hours FROM services WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(service)
}

#[tauri::command]
#[specta::specta]
pub async fn create_service(
    state: State<'_, crate::AppState>,
    payload: CreateServicePayload,
) -> AppResult<Service> {
    let service: Service = sqlx::query_as(
        "INSERT INTO services (code, name, price_type, fixed_price, labor_hours) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.price_type)
    .bind(payload.fixed_price.to_string())
    .bind(payload.labor_hours.to_string())
    .fetch_one(&state.db_pool)
    .await?;
    Ok(service)
}

#[tauri::command]
#[specta::specta]
pub async fn update_service(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateServicePayload,
) -> AppResult<Service> {
    let service: Service = sqlx::query_as(
        "UPDATE services SET code = $2, name = $3, price_type = $4, fixed_price =$5, labor_hours = $6 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.price_type)
    .bind(payload.fixed_price.to_string())
    .bind(payload.labor_hours.to_string())
    .fetch_one(&state.db_pool)
    .await?;
    Ok(service)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_service(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM services WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Сервіса з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_services(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = state.db_pool.begin().await?;
    for id in &ids {
        let result = sqlx::query("DELETE FROM services WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::NotFound(
                "Сервіса з таким ID не знайдено в базі даних".to_string(),
            ));
        }
    }
    tx.commit().await?;
    Ok(())
}
