pub mod pricing;
pub mod structs;

use structs::*;
use tauri::State;

use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_orders_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_orders")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_orders_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrder>> {
    let work_orders: Vec<WorkOrder> = sqlx::query_as("SELECT id, car_id, odometer_in, customer_complaint, mechanic_notes, status, rate_cents, discount_type, discount_cents, discount_percent, created_at, started_at, completed_at, paid_at, payment_method FROM work_orders")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(work_orders)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_totals(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrderTotal>> {
    let work_orders: Vec<WorkOrder> = fetch_work_orders_list(state.clone()).await?;
    let labor_items = fetch_work_order_labor_items_list(state.clone()).await?;
    let part_items = fetch_work_order_part_items_list(state.clone()).await?;

    let totals = work_orders
        .iter()
        .map(|work_order| {
            let wo_labor: Vec<&WorkOrderLaborItem> = labor_items
                .iter()
                .filter(|item| item.work_order_id == work_order.id)
                .collect();
            let wo_parts: Vec<&WorkOrderPartItem> = part_items
                .iter()
                .filter(|item| item.work_order_id == work_order.id)
                .collect();
            let totals = pricing::calculate_totals(work_order, &wo_labor, &wo_parts);
            WorkOrderTotal {
                work_order_id: work_order.id.clone(),
                labor_total_cents: totals.labor_total_cents,
                parts_total_cents: totals.parts_total_cents,
                subtotal_cents: totals.subtotal_cents,
                grand_total_cents: totals.grand_total_cents,
            }
        })
        .collect();

    Ok(totals)
}

#[tauri::command]
#[specta::specta]
pub async fn create_work_order(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderPayload,
) -> AppResult<WorkOrder> {
    let work_order: WorkOrder =
        sqlx::query_as("INSERT INTO work_orders (id, car_id, odometer_in, customer_complaint, mechanic_notes, status, rate_cents, discount_type, discount_cents, discount_percent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *")
            .bind(payload.id)
            .bind(payload.car_id)
            .bind(payload.odometer_in)
            .bind(payload.customer_complaint)
            .bind(payload.mechanic_notes)
            .bind(payload.status)
            .bind(payload.rate_cents)
            .bind(payload.discount_type)
            .bind(payload.discount_cents)
            .bind(payload.discount_percent)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(work_order)
}

#[tauri::command]
#[specta::specta]
pub async fn update_work_order(
    state: State<'_, crate::AppState>,
    payload: UpdateWorkOrderPayload,
) -> AppResult<WorkOrder> {
    let work_order: WorkOrder = sqlx::query_as(
        "UPDATE work_orders SET car_id = $2, odometer_in = $3, customer_complaint = $4, mechanic_notes = $5, status = $6, rate_cents = $7, discount_type = $8, discount_cents = $9, discount_percent = $10, started_at = $11, completed_at = $12, paid_at = $13, payment_method = $14 WHERE id = $1 RETURNING *",
    )
    .bind(payload.id)
    .bind(payload.car_id)
    .bind(payload.odometer_in)
    .bind(payload.customer_complaint)
    .bind(payload.mechanic_notes)
    .bind(payload.status)
    .bind(payload.rate_cents)
    .bind(payload.discount_type)
    .bind(payload.discount_cents)
    .bind(payload.discount_percent)
    .bind(payload.started_at)
    .bind(payload.completed_at)
    .bind(payload.paid_at)
    .bind(payload.payment_method)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(work_order)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_work_order(state: State<'_, crate::AppState>, id: String) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM work_orders WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Замовлення з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_labor_items_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrderLaborItem>> {
    let work_order_labor_items: Vec<WorkOrderLaborItem> = sqlx::query_as(
        "SELECT id, work_order_id, service_id, code, name, labor_hours FROM work_order_labor_items",
    )
    .fetch_all(&state.db_pool)
    .await?;
    Ok(work_order_labor_items)
}

#[tauri::command]
#[specta::specta]
pub async fn create_work_order_labor_item(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderLaborItemPayload,
) -> AppResult<WorkOrderLaborItem> {
    let work_order_labor_item: WorkOrderLaborItem =
        sqlx::query_as("INSERT INTO work_order_labor_items (work_order_id, service_id, code, name, labor_hours) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(payload.work_order_id)
            .bind(payload.service_id)
            .bind(payload.code)
            .bind(payload.name)
            .bind(payload.labor_hours)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(work_order_labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_work_order_labor_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateWorkOrderLaborItemPayload,
) -> AppResult<WorkOrderLaborItem> {
    let work_order_labor_item: WorkOrderLaborItem = sqlx::query_as(
        "UPDATE work_order_labor_items SET service_id = $2, code = $3, name = $4, labor_hours = $5 WHERE id = $1 RETURNING *",
    )
        .bind(id)
        .bind(payload.service_id)
        .bind(payload.code)
        .bind(payload.name)
        .bind(payload.labor_hours)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(work_order_labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_work_order_labor_item(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM work_order_labor_items WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Сервісу з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_part_items_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrderPartItem>> {
    let work_order_part_items: Vec<WorkOrderPartItem> =
        sqlx::query_as("SELECT id, work_order_id, code, name, quantity, unit_type, cost_cents, markup, status FROM work_order_part_items")
            .fetch_all(&state.db_pool)
            .await?;
    Ok(work_order_part_items)
}

#[tauri::command]
#[specta::specta]
pub async fn create_work_order_part_item(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderPartItemPayload,
) -> AppResult<WorkOrderPartItem> {
    let work_order_part_item: WorkOrderPartItem =
        sqlx::query_as("INSERT INTO work_order_part_items (work_order_id, code, name, quantity, unit_type, cost_cents, markup, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *")
            .bind(payload.work_order_id)
            .bind(payload.code)
            .bind(payload.name)
            .bind(payload.quantity)
            .bind(payload.unit_type)
            .bind(payload.cost_cents)
            .bind(payload.markup)
            .bind(payload.status)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(work_order_part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_work_order_part_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateWorkOrderPartItemPayload,
) -> AppResult<WorkOrderPartItem> {
    let work_order_part_item: WorkOrderPartItem = sqlx::query_as(
        "UPDATE work_order_part_items SET code = $2, name = $3, quantity = $4, unit_type = $5, cost_cents = $6, markup = $7, status = $8 WHERE id = $1 RETURNING *",
    )
        .bind(id)
        .bind(payload.code)
        .bind(payload.name)
        .bind(payload.quantity)
        .bind(payload.unit_type)
        .bind(payload.cost_cents)
        .bind(payload.markup)
        .bind(payload.status)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(work_order_part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_work_order_part_item(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM work_order_part_items WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Запчастини з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}
