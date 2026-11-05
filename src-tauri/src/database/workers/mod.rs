pub mod structs;

use std::collections::HashMap;

use sqlx::{Executor, QueryBuilder, Sqlite};
use structs::*;
use tauri::State;

use crate::{
    database::utils::begin_immediate,
    error::{AppError, AppResult},
    logging::Event,
};

const WORKER_COLUMNS: &str = r#"
    id,
    first_name,
    surname,
    patronymic,
    position,
    is_active
"#;

fn trim_opt(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

struct WorkerFields {
    first_name: String,
    surname: String,
    patronymic: Option<String>,
    position: Option<String>,
}

fn normalize_fields(
    first_name: &str,
    surname: &str,
    patronymic: &Option<String>,
    position: &Option<String>,
) -> AppResult<WorkerFields> {
    let surname = surname.trim();
    if surname.is_empty() {
        return Err(AppError::validation("Вкажіть прізвище працівника"));
    }
    let first_name = first_name.trim();
    if first_name.is_empty() {
        return Err(AppError::validation("Вкажіть ім'я працівника"));
    }
    Ok(WorkerFields {
        first_name: first_name.to_string(),
        surname: surname.to_string(),
        patronymic: trim_opt(patronymic),
        position: trim_opt(position),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_workers_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workers")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_workers_list(state: State<'_, crate::AppState>) -> AppResult<Vec<Worker>> {
    let workers: Vec<Worker> = sqlx::query_as(&format!(
        "SELECT {WORKER_COLUMNS} FROM workers ORDER BY surname, first_name, patronymic"
    ))
    .fetch_all(&state.db_pool)
    .await?;
    Ok(workers)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_worker_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<Worker> {
    let worker: Worker = sqlx::query_as(&format!(
        "SELECT {WORKER_COLUMNS} FROM workers WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(AppError::worker_not_found)?;
    Ok(worker)
}

#[tauri::command]
#[specta::specta]
pub async fn create_worker(
    state: State<'_, crate::AppState>,
    payload: CreateWorkerPayload,
) -> AppResult<Worker> {
    let fields = normalize_fields(
        &payload.first_name,
        &payload.surname,
        &payload.patronymic,
        &payload.position,
    )?;

    let mut tx = begin_immediate(&state.db_pool).await?;

    let worker: Worker = sqlx::query_as(&format!(
        r#"
        INSERT INTO workers (first_name, surname, patronymic, position, is_active)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING {WORKER_COLUMNS}
        "#
    ))
    .bind(fields.first_name)
    .bind(fields.surname)
    .bind(fields.patronymic)
    .bind(fields.position)
    .bind(payload.is_active)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("worker.create").id(worker.id).ok();

    Ok(worker)
}

#[tauri::command]
#[specta::specta]
pub async fn update_worker(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateWorkerPayload,
) -> AppResult<Worker> {
    let fields = normalize_fields(
        &payload.first_name,
        &payload.surname,
        &payload.patronymic,
        &payload.position,
    )?;

    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_worker_exists(&mut *tx, id).await?;

    let worker: Worker = sqlx::query_as(&format!(
        r#"
        UPDATE workers SET
            first_name = $2,
            surname = $3,
            patronymic = $4,
            position = $5,
            is_active = $6
        WHERE id = $1
        RETURNING {WORKER_COLUMNS}
        "#
    ))
    .bind(id)
    .bind(fields.first_name)
    .bind(fields.surname)
    .bind(fields.patronymic)
    .bind(fields.position)
    .bind(payload.is_active)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("worker.update").id(id).ok();

    Ok(worker)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_worker(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM workers WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::worker_not_found());
    }

    Event::new("worker.delete").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_workers(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query("DELETE FROM workers WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::worker_not_found());
        }
    }
    tx.commit().await?;

    Event::new("worker.delete_many")
        .field("count", ids.len())
        .ok();

    Ok(())
}

pub(crate) async fn fetch_workers_by_ids(
    state: &State<'_, crate::AppState>,
    ids: &[i64],
) -> AppResult<HashMap<i64, Worker>> {
    let mut unique_ids: Vec<i64> = ids.to_vec();
    unique_ids.sort_unstable();
    unique_ids.dedup();
    if unique_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut qb = QueryBuilder::new(format!(
        "SELECT {WORKER_COLUMNS} FROM workers WHERE id IN ("
    ));
    let mut separated = qb.separated(", ");
    for id in &unique_ids {
        separated.push_bind(*id);
    }
    qb.push(")");

    let workers: Vec<Worker> = qb.build_query_as().fetch_all(&state.db_pool).await?;
    Ok(workers
        .into_iter()
        .map(|worker| (worker.id, worker))
        .collect())
}

pub(crate) async fn ensure_worker_exists<'e, E>(executor: E, id: i32) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let existing: Option<i32> = sqlx::query_scalar("SELECT id FROM workers WHERE id = $1")
        .bind(id)
        .fetch_optional(executor)
        .await?;
    existing.map(|_| ()).ok_or_else(AppError::worker_not_found)
}
