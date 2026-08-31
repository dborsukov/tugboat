pub mod structs;

use sqlx::{Executor, Sqlite};
use structs::*;
use tauri::State;

use crate::{
    database::utils::begin_immediate,
    error::{AppError, AppResult},
};

#[tauri::command]
#[specta::specta]
pub async fn fetch_services_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE archived_at IS NULL")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_services_list(state: State<'_, crate::AppState>) -> AppResult<Vec<Service>> {
    let services: Vec<Service> = sqlx::query_as(
        r#"
        SELECT
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        FROM services
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;
    Ok(services)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_service_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<Service> {
    let service: Service = sqlx::query_as(
        r#"
        SELECT
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        FROM services
        WHERE id = $1
        "#,
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
        r#"
        INSERT INTO services (
            code,
            name,
            price_type,
            fixed_price,
            labor_hours
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        "#,
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
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_service_active(&mut *tx, id).await?;

    let service: Service = sqlx::query_as(
        r#"
        UPDATE services SET
            code = $2,
            name = $3,
            price_type = $4,
            fixed_price = $5,
            labor_hours = $6
        WHERE id = $1
        RETURNING
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        "#,
    )
    .bind(id)
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.price_type)
    .bind(payload.fixed_price.to_string())
    .bind(payload.labor_hours.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
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
        return Err(AppError::service_not_found());
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_services(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query("DELETE FROM services WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::service_not_found());
        }
    }
    tx.commit().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn archive_service(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE services SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = $1
          AND archived_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&state.db_pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::service_not_found());
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn archive_services(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query(
            r#"
            UPDATE services SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
            WHERE id = $1
              AND archived_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::service_not_found());
        }
    }
    tx.commit().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_service(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("UPDATE services SET archived_at = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::service_not_found());
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_services(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query("UPDATE services SET archived_at = NULL WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::service_not_found());
        }
    }
    tx.commit().await?;
    Ok(())
}

async fn ensure_service_active<'e, E>(executor: E, id: i32) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let archived_at: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM services WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await?;
    match archived_at {
        None => Err(AppError::service_not_found()),
        Some(Some(_)) => Err(AppError::service_archived()),
        Some(None) => Ok(()),
    }
}
