pub mod dtos;
pub mod pricing;
pub mod structs;

use std::collections::HashMap;

use rust_decimal::Decimal;
use structs::*;
use tauri::State;

use crate::{
    database::{
        cars::{dtos::CarDto, fetch_car_by_id},
        clients::{fetch_client_by_id, structs::Client},
        work_orders::dtos::{LaborItemDto, PartItemDto, WorkOrderDetailsDto},
    },
    error::{AppError, AppResult},
};

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
pub async fn fetch_work_order_by_id(
    state: &State<'_, crate::AppState>,
    id: &String,
) -> AppResult<WorkOrder> {
    let work_order: WorkOrder = sqlx::query_as("SELECT id, car_id, odometer_in, customer_complaint, mechanic_notes, status, rate, discount_type, discount_amount, discount_percent, created_at, started_at, completed_at, paid_at, payment_method FROM work_orders WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(work_order)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_orders_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrder>> {
    let work_orders: Vec<WorkOrder> = sqlx::query_as("SELECT id, car_id, odometer_in, customer_complaint, mechanic_notes, status, rate, discount_type, discount_amount, discount_percent, created_at, started_at, completed_at, paid_at, payment_method FROM work_orders")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(work_orders)
}

async fn build_work_order_details_dto(
    state: &State<'_, crate::AppState>,
    work_order: WorkOrder,
) -> AppResult<WorkOrderDetailsDto> {
    let car_dto = fetch_car_by_id(state.clone(), work_order.car_id as i32).await?;
    let client = fetch_client_by_id(state.clone(), car_dto.car.client_id as i32).await?;

    build_work_order_details_dto_with_context(state.clone(), work_order, car_dto, client).await
}

async fn build_work_order_details_dto_with_context(
    state: State<'_, crate::AppState>,
    work_order: WorkOrder,
    car_dto: CarDto,
    client: Client,
) -> AppResult<WorkOrderDetailsDto> {
    let labor_items: Vec<LaborItem> = sqlx::query_as(
        "SELECT id, service_id, work_order_id, code, name, labor_hours, discount_type, discount_amount, discount_percent FROM labor_items WHERE work_order_id = $1",
    )
    .bind(&work_order.id)
    .fetch_all(&state.db_pool)
    .await?;

    let part_items: Vec<PartItem> = sqlx::query_as(
        "SELECT id, work_order_id, status, code, name, quantity, unit_type, base_cost, markup, discount_type, discount_amount, discount_percent FROM part_items WHERE work_order_id = $1",
    )
    .bind(&work_order.id)
    .fetch_all(&state.db_pool)
    .await?;

    let labor_item_dtos: Vec<LaborItemDto> = labor_items
        .into_iter()
        .map(|item| {
            let item_totals = pricing::calculate_labor_item_totals(&item, work_order.rate);
            LaborItemDto {
                item,
                totals: item_totals,
            }
        })
        .collect();

    let part_item_dtos: Vec<PartItemDto> = part_items
        .into_iter()
        .map(|item| {
            let item_totals = pricing::calculate_part_item_totals(&item);
            PartItemDto {
                item,
                totals: item_totals,
            }
        })
        .collect();

    let labor_sum: Decimal = labor_item_dtos.iter().map(|dto| dto.totals.total).sum();
    let parts_sum: Decimal = part_item_dtos.iter().map(|dto| dto.totals.total).sum();

    let totals = pricing::calculate_work_order_totals(&work_order, labor_sum, parts_sum);

    Ok(WorkOrderDetailsDto {
        work_order,
        car_dto,
        client,
        labor_items: labor_item_dtos,
        part_items: part_item_dtos,
        totals,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto(
    state: State<'_, crate::AppState>,
    id: String,
) -> AppResult<WorkOrderDetailsDto> {
    let work_order: WorkOrder = fetch_work_order_by_id(&state, &id).await?;
    build_work_order_details_dto(&state, work_order).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrderDetailsDto>> {
    let work_orders: Vec<WorkOrder> = sqlx::query_as("SELECT id, car_id, odometer_in, customer_complaint, mechanic_notes, status, rate, discount_type, discount_amount, discount_percent, created_at, started_at, completed_at, paid_at, payment_method FROM work_orders")
        .fetch_all(&state.db_pool)
        .await?;

    let mut details_list: Vec<WorkOrderDetailsDto> = Vec::with_capacity(work_orders.len());
    for work_order in work_orders {
        let details = build_work_order_details_dto(&state, work_order).await?;
        details_list.push(details);
    }

    Ok(details_list)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto_by_car_id(
    state: State<'_, crate::AppState>,
    car_id: i32,
) -> AppResult<Vec<WorkOrderDetailsDto>> {
    let car_dto = fetch_car_by_id(state.clone(), car_id).await?;
    let client = fetch_client_by_id(state.clone(), car_dto.car.client_id as i32).await?;

    let work_orders: Vec<WorkOrder> = sqlx::query_as("SELECT id, car_id, odometer_in, customer_complaint, mechanic_notes, status, rate, discount_type, discount_amount, discount_percent, created_at, started_at, completed_at, paid_at, payment_method FROM work_orders WHERE car_id = $1")
        .bind(car_id)
        .fetch_all(&state.db_pool)
        .await?;

    let mut details_list: Vec<WorkOrderDetailsDto> = Vec::with_capacity(work_orders.len());
    for work_order in work_orders {
        let details = build_work_order_details_dto_with_context(
            state.clone(),
            work_order,
            car_dto.clone(),
            client.clone(),
        )
        .await?;
        details_list.push(details);
    }

    Ok(details_list)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto_by_client_id(
    state: State<'_, crate::AppState>,
    client_id: i32,
) -> AppResult<Vec<WorkOrderDetailsDto>> {
    let client = fetch_client_by_id(state.clone(), client_id).await?;

    let work_orders: Vec<WorkOrder> = sqlx::query_as(
        "SELECT wo.id, wo.car_id, wo.odometer_in, wo.customer_complaint, wo.mechanic_notes, wo.status, wo.rate, wo.discount_type, wo.discount_amount, wo.discount_percent, wo.created_at, wo.started_at, wo.completed_at, wo.paid_at, wo.payment_method \
         FROM work_orders wo \
         JOIN cars c ON c.id = wo.car_id \
         WHERE c.client_id = $1",
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Cache car_dto lookups so a client with many work orders on the same car
    // doesn't refetch that car for every single one.
    let mut car_cache: HashMap<i64, CarDto> = HashMap::new();
    let mut details_list: Vec<WorkOrderDetailsDto> = Vec::with_capacity(work_orders.len());

    for work_order in work_orders {
        let car_dto = if let Some(cached) = car_cache.get(&work_order.car_id) {
            cached.clone()
        } else {
            let fetched = fetch_car_by_id(state.clone(), work_order.car_id as i32).await?;
            car_cache.insert(work_order.car_id, fetched.clone());
            fetched
        };

        let details = build_work_order_details_dto_with_context(
            state.clone(),
            work_order,
            car_dto,
            client.clone(),
        )
        .await?;
        details_list.push(details);
    }

    Ok(details_list)
}

#[tauri::command]
#[specta::specta]
pub async fn create_work_order(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderPayload,
) -> AppResult<WorkOrder> {
    let work_order: WorkOrder =
        sqlx::query_as("INSERT INTO work_orders (id, car_id, odometer_in, customer_complaint, mechanic_notes, status, rate, discount_type, discount_amount, discount_percent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *")
            .bind(payload.id)
            .bind(payload.car_id)
            .bind(payload.odometer_in)
            .bind(payload.customer_complaint)
            .bind(payload.mechanic_notes)
            .bind(payload.status)
            .bind(payload.rate.to_string())
            .bind(payload.discount_type)
            .bind(payload.discount_amount.to_string())
            .bind(payload.discount_percent.to_string())
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
        "UPDATE work_orders SET car_id = $2, odometer_in = $3, customer_complaint = $4, mechanic_notes = $5, status = $6, rate = $7, discount_type = $8, discount_amount = $9, discount_percent = $10, started_at = $11, completed_at = $12, paid_at = $13, payment_method = $14 WHERE id = $1 RETURNING *",
    )
    .bind(payload.id)
    .bind(payload.car_id)
    .bind(payload.odometer_in)
    .bind(payload.customer_complaint)
    .bind(payload.mechanic_notes)
    .bind(payload.status)
    .bind(payload.rate.to_string())
    .bind(payload.discount_type)
    .bind(payload.discount_amount.to_string())
    .bind(payload.discount_percent.to_string())
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
pub async fn fetch_labor_items_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<LaborItem>> {
    let labor_items: Vec<LaborItem> = sqlx::query_as(
        "SELECT id, service_id, work_order_id, code, name, labor_hours, discount_type, discount_amount, discount_percent FROM labor_items",
    )
    .fetch_all(&state.db_pool)
    .await?;
    Ok(labor_items)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_labor_item_by_id(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<LaborItem> {
    let labor_item: LaborItem = sqlx::query_as(
        "SELECT id, service_id, work_order_id, code, name, labor_hours, discount_type, discount_amount, discount_percent FROM labor_items WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn create_labor_item(
    state: State<'_, crate::AppState>,
    payload: CreateLaborItemPayload,
) -> AppResult<LaborItem> {
    let labor_item: LaborItem =
        sqlx::query_as("INSERT INTO labor_items (service_id, work_order_id, code, name, labor_hours, discount_type, discount_amount, discount_percent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *")
            .bind(payload.service_id)
            .bind(payload.work_order_id)
            .bind(payload.code)
            .bind(payload.name)
            .bind(payload.labor_hours.to_string())
            .bind(payload.discount_type)
            .bind(payload.discount_amount.to_string())
            .bind(payload.discount_percent.to_string())
            .fetch_one(&state.db_pool)
            .await?;
    Ok(labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_labor_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateLaborItemPayload,
) -> AppResult<LaborItem> {
    let labor_item: LaborItem = sqlx::query_as(
        "UPDATE labor_items SET service_id = $2, code = $3, name = $4, labor_hours = $5, discount_type = $6, discount_amount = $7, discount_percent = $8 WHERE id = $1 RETURNING *",
    )
        .bind(id)
        .bind(payload.service_id)
        .bind(payload.code)
        .bind(payload.name)
        .bind(payload.labor_hours.to_string())
        .bind(payload.discount_type)
        .bind(payload.discount_amount.to_string())
        .bind(payload.discount_percent.to_string())
        .fetch_one(&state.db_pool)
        .await?;
    Ok(labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_labor_item(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM labor_items WHERE id = $1")
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
pub async fn fetch_part_items_list(state: State<'_, crate::AppState>) -> AppResult<Vec<PartItem>> {
    let part_items: Vec<PartItem> =
        sqlx::query_as("SELECT id, work_order_id, status, code, name, quantity, unit_type, base_cost, markup, discount_type, discount_amount, discount_percent FROM part_items")
            .fetch_all(&state.db_pool)
            .await?;
    Ok(part_items)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_part_item_by_id(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<PartItem> {
    let part_item: PartItem =
        sqlx::query_as("SELECT id, work_order_id, status, code, name, quantity, unit_type, base_cost, markup, discount_type, discount_amount, discount_percent FROM part_items WHERE id = $1")
            .bind(id)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn create_part_item(
    state: State<'_, crate::AppState>,
    payload: CreatePartItemPayload,
) -> AppResult<PartItem> {
    let part_item: PartItem =
        sqlx::query_as("INSERT INTO part_items (work_order_id, status, code, name, quantity, unit_type, base_cost, markup, discount_type, discount_amount, discount_percent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *")
            .bind(payload.work_order_id)
            .bind(payload.status)
            .bind(payload.code)
            .bind(payload.name)
            .bind(payload.quantity.to_string())
            .bind(payload.unit_type)
            .bind(payload.base_cost.to_string())
            .bind(payload.markup.to_string())
            .bind(payload.discount_type)
            .bind(payload.discount_amount.to_string())
            .bind(payload.discount_percent.to_string())
            .fetch_one(&state.db_pool)
            .await?;
    Ok(part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_part_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdatePartItemPayload,
) -> AppResult<PartItem> {
    let part_item: PartItem = sqlx::query_as(
        "UPDATE part_items SET status = $2, code = $3, name = $4, quantity = $5, unit_type = $6, base_cost = $7, markup = $8, discount_type = $9, discount_amount = $10, discount_percent = $11 WHERE id = $1 RETURNING *",
    )
        .bind(id)
        .bind(payload.status)
        .bind(payload.code)
        .bind(payload.name)
        .bind(payload.quantity.to_string())
        .bind(payload.unit_type)
        .bind(payload.base_cost.to_string())
        .bind(payload.markup.to_string())
        .bind(payload.discount_type)
        .bind(payload.discount_amount.to_string())
        .bind(payload.discount_percent.to_string())
        .fetch_one(&state.db_pool)
        .await?;
    Ok(part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_part_item(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM part_items WHERE id = $1")
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
