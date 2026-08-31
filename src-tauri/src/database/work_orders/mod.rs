pub mod dtos;
pub mod pricing;
pub mod structs;

use std::collections::HashMap;

use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, QueryBuilder, Sqlite};
use structs::*;
use tauri::State;

use crate::{
    database::{
        cars, clients,
        utils::begin_immediate,
        work_orders::dtos::{LaborItemDto, PartItemDto, WorkOrderDetailsDto},
    },
    error::{AppError, AppResult},
};

/// Item table bulk operation targets, so table names stay
/// compile time checked instead of typed with strings.
enum ItemTable {
    Labor,
    Part,
}

impl ItemTable {
    fn table_name(self) -> &'static str {
        match self {
            ItemTable::Labor => "labor_items",
            ItemTable::Part => "part_items",
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_orders_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_orders WHERE archived_at IS NULL")
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
    let work_order: WorkOrder = sqlx::query_as(
        r#"
        SELECT
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            created_at,
            archived_at,
            started_at,
            completed_at,
            paid_at,
            payment_method
        FROM work_orders
        WHERE id = $1
        "#,
    )
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
    let work_orders: Vec<WorkOrder> = sqlx::query_as(
        r#"
        SELECT
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            created_at,
            archived_at,
            started_at,
            completed_at,
            paid_at,
            payment_method
        FROM work_orders
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;
    Ok(work_orders)
}

async fn fetch_labor_items_by_work_order_ids(
    state: &State<'_, crate::AppState>,
    work_order_ids: &[&str],
) -> AppResult<HashMap<String, Vec<LaborItem>>> {
    if work_order_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut qb = QueryBuilder::new(
        r#"
        SELECT
            id,
            service_id,
            work_order_id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            discount_type,
            discount_value
        FROM labor_items
        WHERE work_order_id IN ("#,
    );
    let mut separated = qb.separated(", ");
    for id in work_order_ids {
        separated.push_bind(*id);
    }
    qb.push(")");

    let items: Vec<LaborItem> = qb.build_query_as().fetch_all(&state.db_pool).await?;

    let mut by_order: HashMap<String, Vec<LaborItem>> = HashMap::new();
    for item in items {
        by_order
            .entry(item.work_order_id.clone())
            .or_default()
            .push(item);
    }
    Ok(by_order)
}

async fn fetch_part_items_by_work_order_ids(
    state: &State<'_, crate::AppState>,
    work_order_ids: &[&str],
) -> AppResult<HashMap<String, Vec<PartItem>>> {
    if work_order_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut qb = QueryBuilder::new(
        r#"
        SELECT
            id,
            work_order_id,
            status,
            code,
            name,
            quantity,
            unit_type,
            base_cost,
            markup,
            discount_type,
            discount_value
        FROM part_items
        WHERE work_order_id IN ("#,
    );
    let mut separated = qb.separated(", ");
    for id in work_order_ids {
        separated.push_bind(*id);
    }
    qb.push(")");

    let items: Vec<PartItem> = qb.build_query_as().fetch_all(&state.db_pool).await?;

    let mut by_order: HashMap<String, Vec<PartItem>> = HashMap::new();
    for item in items {
        by_order
            .entry(item.work_order_id.clone())
            .or_default()
            .push(item);
    }
    Ok(by_order)
}

async fn build_work_order_details_dtos(
    state: &State<'_, crate::AppState>,
    work_orders: Vec<WorkOrder>,
) -> AppResult<Vec<WorkOrderDetailsDto>> {
    if work_orders.is_empty() {
        return Ok(Vec::new());
    }

    let car_ids: Vec<i64> = work_orders.iter().map(|wo| wo.car_id).collect();
    let cars_by_id = cars::fetch_cars_by_ids(state, &car_ids).await?;

    let client_ids: Vec<i64> = cars_by_id.values().map(|dto| dto.car.client_id).collect();
    let clients_by_id = clients::fetch_clients_by_ids(state, &client_ids).await?;
    let client_details_by_id = clients::fetch_client_details_by_ids(state, &client_ids).await?;

    let work_order_ids: Vec<&str> = work_orders.iter().map(|wo| wo.id.as_str()).collect();
    let mut labor_items_by_order =
        fetch_labor_items_by_work_order_ids(state, &work_order_ids).await?;
    let mut part_items_by_order =
        fetch_part_items_by_work_order_ids(state, &work_order_ids).await?;

    let mut details_list: Vec<WorkOrderDetailsDto> = Vec::with_capacity(work_orders.len());

    for work_order in work_orders {
        let car_dto = cars_by_id.get(&work_order.car_id).cloned().ok_or_else(|| {
            AppError::NotFound(format!(
                "Автомобіль для замовлення {} не знайдено в базі даних",
                work_order.id
            ))
        })?;
        let client = clients_by_id
            .get(&car_dto.car.client_id)
            .cloned()
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Клієнта для замовлення {} не знайдено в базі даних",
                    work_order.id
                ))
            })?;
        let client_details = client_details_by_id.get(&car_dto.car.client_id).cloned();

        let labor_item_dtos: Vec<LaborItemDto> = labor_items_by_order
            .remove(&work_order.id)
            .unwrap_or_default()
            .into_iter()
            .map(|item| {
                let item_totals = pricing::calculate_labor_item_totals(
                    &item,
                    work_order.rate,
                    work_order.labor_cost_type,
                    work_order.labor_cost_value,
                );
                LaborItemDto {
                    item,
                    totals: item_totals,
                }
            })
            .collect();

        let part_item_dtos: Vec<PartItemDto> = part_items_by_order
            .remove(&work_order.id)
            .unwrap_or_default()
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
        let labor_cost_sum: Decimal = labor_item_dtos.iter().map(|dto| dto.totals.cost).sum();
        let parts_cost_sum: Decimal = part_item_dtos.iter().map(|dto| dto.totals.cost).sum();
        let labor_discount_sum: Decimal =
            labor_item_dtos.iter().map(|dto| dto.totals.discount).sum();
        let parts_discount_sum: Decimal =
            part_item_dtos.iter().map(|dto| dto.totals.discount).sum();

        let totals = pricing::calculate_work_order_totals(
            &work_order,
            labor_sum,
            labor_cost_sum,
            parts_sum,
            parts_cost_sum,
            labor_discount_sum,
            parts_discount_sum,
        );

        details_list.push(WorkOrderDetailsDto {
            work_order,
            car_dto,
            client,
            client_details,
            labor_items: labor_item_dtos,
            part_items: part_item_dtos,
            totals,
        });
    }

    Ok(details_list)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto(
    state: State<'_, crate::AppState>,
    id: String,
) -> AppResult<WorkOrderDetailsDto> {
    let work_order: WorkOrder = fetch_work_order_by_id(&state, &id).await?;
    let mut details = build_work_order_details_dtos(&state, vec![work_order]).await?;
    Ok(details.remove(0))
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrderDetailsDto>> {
    let work_orders: Vec<WorkOrder> = fetch_work_orders_list(state.clone()).await?;
    build_work_order_details_dtos(&state, work_orders).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto_by_car_id(
    state: State<'_, crate::AppState>,
    car_id: i32,
) -> AppResult<Vec<WorkOrderDetailsDto>> {
    let work_orders: Vec<WorkOrder> = sqlx::query_as(
        r#"
        SELECT
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            created_at,
            archived_at,
            started_at,
            completed_at,
            paid_at,
            payment_method
        FROM work_orders
        WHERE car_id = $1
        "#,
    )
    .bind(car_id)
    .fetch_all(&state.db_pool)
    .await?;

    build_work_order_details_dtos(&state, work_orders).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_details_dto_by_client_id(
    state: State<'_, crate::AppState>,
    client_id: i32,
) -> AppResult<Vec<WorkOrderDetailsDto>> {
    let work_orders: Vec<WorkOrder> = sqlx::query_as(
        r#"
        SELECT
            wo.id,
            wo.car_id,
            wo.odometer_in,
            wo.description,
            wo.notes,
            wo.status,
            wo.rate,
            wo.labor_cost_type,
            wo.labor_cost_value,
            wo.discount_type,
            wo.discount_value,
            wo.created_at,
            wo.archived_at,
            wo.started_at,
            wo.completed_at,
            wo.paid_at,
            wo.payment_method
        FROM work_orders wo
        JOIN cars c ON c.id = wo.car_id
        WHERE c.client_id = $1
        "#,
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await?;

    build_work_order_details_dtos(&state, work_orders).await
}

/// Generates a new work order id in the form `YYYYMM-NNNN`, where the
/// counter resets automatically at the start of each month.
/// Transaction failure will roll back counter, leaving no gap.
async fn generate_work_order_id(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> AppResult<String> {
    const PREFIX: &str = "ПП";
    let year_month = Utc::now().format("%Y%m").to_string();

    let counter: i64 = sqlx::query_scalar(
        "INSERT INTO work_order_sequences (year_month, counter) VALUES ($1, 1) \
         ON CONFLICT (year_month) DO UPDATE SET counter = counter + 1 \
         RETURNING counter",
    )
    .bind(&year_month)
    .fetch_one(&mut **tx)
    .await?;

    Ok(format!("{PREFIX}-{year_month}-{counter:03}"))
}

#[tauri::command]
#[specta::specta]
pub async fn create_work_order(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderPayload,
) -> AppResult<WorkOrder> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    cars::ensure_car_active(&mut *tx, payload.car_id as i32).await?;

    let id = generate_work_order_id(&mut tx).await?;

    let work_order: WorkOrder = sqlx::query_as(
        r#"
        INSERT INTO work_orders (
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            created_at,
            archived_at,
            started_at,
            completed_at,
            paid_at,
            payment_method
        "#,
    )
    .bind(id)
    .bind(payload.car_id)
    .bind(payload.odometer_in)
    .bind(payload.description)
    .bind(payload.notes)
    .bind(payload.status)
    .bind(payload.rate.to_string())
    .bind(payload.labor_cost_type)
    .bind(payload.labor_cost_value.to_string())
    .bind(payload.discount_type)
    .bind(payload.discount_value.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(work_order)
}

#[tauri::command]
#[specta::specta]
pub async fn update_work_order(
    state: State<'_, crate::AppState>,
    payload: UpdateWorkOrderPayload,
) -> AppResult<WorkOrder> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_work_order_active(&mut *tx, &payload.id).await?;
    cars::ensure_car_active(&mut *tx, payload.car_id as i32).await?;

    let work_order: WorkOrder = sqlx::query_as(
        r#"
        UPDATE work_orders SET
            car_id = $2,
            odometer_in = $3,
            description = $4,
            notes = $5,
            status = $6,
            rate = $7,
            labor_cost_type = $8,
            labor_cost_value = $9,
            discount_type = $10,
            discount_value = $11,
            started_at = $12,
            completed_at = $13,
            paid_at = $14,
            payment_method = $15
        WHERE id = $1
        RETURNING
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            created_at,
            archived_at,
            started_at,
            completed_at,
            paid_at,
            payment_method
        "#,
    )
    .bind(payload.id)
    .bind(payload.car_id)
    .bind(payload.odometer_in)
    .bind(payload.description)
    .bind(payload.notes)
    .bind(payload.status)
    .bind(payload.rate.to_string())
    .bind(payload.labor_cost_type)
    .bind(payload.labor_cost_value.to_string())
    .bind(payload.discount_type)
    .bind(payload.discount_value.to_string())
    .bind(payload.started_at)
    .bind(payload.completed_at)
    .bind(payload.paid_at)
    .bind(payload.payment_method)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
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
        return Err(AppError::work_order_not_found());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderDeleteImpact {
    #[specta(type = specta_typescript::Number)]
    pub labor_item_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub part_item_count: i64,
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_delete_impact(
    state: State<'_, crate::AppState>,
    id: String,
) -> AppResult<WorkOrderDeleteImpact> {
    let mut tx = state.db_pool.begin().await?;

    let labor_item_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM labor_items WHERE work_order_id = $1")
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;

    let part_item_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM part_items WHERE work_order_id = $1")
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;

    tx.commit().await?;

    Ok(WorkOrderDeleteImpact {
        labor_item_count,
        part_item_count,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn archive_work_order(state: State<'_, crate::AppState>, id: String) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE work_orders SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = $1
          AND archived_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&state.db_pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::work_order_not_found());
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_work_order(state: State<'_, crate::AppState>, id: String) -> AppResult<()> {
    let result = sqlx::query("UPDATE work_orders SET archived_at = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::work_order_not_found());
    }
    Ok(())
}

async fn ensure_work_order_active<'e, E>(executor: E, id: &str) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let archived_at: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM work_orders WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await?;
    match archived_at {
        None => Err(AppError::work_order_not_found()),
        Some(Some(_)) => Err(AppError::work_order_archived()),
        Some(None) => Ok(()),
    }
}

async fn ensure_item_work_order_active<'e, E>(
    executor: E,
    table: ItemTable,
    item_id: i32,
) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let archived_at: Option<Option<String>> = sqlx::query_scalar(&format!(
        "SELECT wo.archived_at FROM {table} li \
         JOIN work_orders wo ON wo.id = li.work_order_id \
         WHERE li.id = $1",
        table = table.table_name(),
    ))
    .bind(item_id)
    .fetch_optional(executor)
    .await?;
    match archived_at {
        None => Err(AppError::item_not_found()),
        Some(Some(_)) => Err(AppError::work_order_archived()),
        Some(None) => Ok(()),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_labor_items_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<LaborItem>> {
    let labor_items: Vec<LaborItem> = sqlx::query_as(
        r#"
        SELECT
            id,
            service_id,
            work_order_id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            discount_type,
            discount_value
        FROM labor_items
        "#,
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
        r#"
        SELECT
            id,
            service_id,
            work_order_id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            discount_type,
            discount_value
        FROM labor_items
        WHERE id = $1
        "#,
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
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_work_order_active(&mut *tx, &payload.work_order_id).await?;

    let labor_item: LaborItem = sqlx::query_as(
        r#"
        INSERT INTO labor_items (
            service_id,
            work_order_id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            discount_type,
            discount_value
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
            id,
            service_id,
            work_order_id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            discount_type,
            discount_value
        "#,
    )
    .bind(payload.service_id)
    .bind(payload.work_order_id)
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.price_type)
    .bind(payload.fixed_price.to_string())
    .bind(payload.labor_hours.to_string())
    .bind(payload.discount_type)
    .bind(payload.discount_value.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_labor_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateLaborItemPayload,
) -> AppResult<LaborItem> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_item_work_order_active(&mut *tx, ItemTable::Labor, id).await?;

    let labor_item: LaborItem = sqlx::query_as(
        r#"
        UPDATE labor_items SET
            service_id = $2,
            code = $3,
            name = $4,
            price_type = $5,
            fixed_price = $6,
            labor_hours = $7,
            discount_type = $8,
            discount_value = $9
        WHERE id = $1
        RETURNING
            id,
            service_id,
            work_order_id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            discount_type,
            discount_value
        "#,
    )
    .bind(id)
    .bind(payload.service_id)
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.price_type)
    .bind(payload.fixed_price.to_string())
    .bind(payload.labor_hours.to_string())
    .bind(payload.discount_type)
    .bind(payload.discount_value.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_labor_item(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_item_work_order_active(&mut *tx, ItemTable::Labor, id).await?;

    let result = sqlx::query("DELETE FROM labor_items WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::item_not_found());
    }

    tx.commit().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_labor_items(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        ensure_item_work_order_active(&mut *tx, ItemTable::Labor, *id).await?;
        let result = sqlx::query("DELETE FROM labor_items WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::item_not_found());
        }
    }
    tx.commit().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_part_items_list(state: State<'_, crate::AppState>) -> AppResult<Vec<PartItem>> {
    let part_items: Vec<PartItem> = sqlx::query_as(
        r#"
        SELECT
            id,
            work_order_id,
            status,
            code,
            name,
            quantity,
            unit_type,
            base_cost,
            markup,
            discount_type,
            discount_value
        FROM part_items
        "#,
    )
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
    let part_item: PartItem = sqlx::query_as(
        r#"
        SELECT
            id,
            work_order_id,
            status,
            code,
            name,
            quantity,
            unit_type,
            base_cost,
            markup,
            discount_type,
            discount_value
        FROM part_items
        WHERE id = $1
        "#,
    )
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
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_work_order_active(&mut *tx, &payload.work_order_id).await?;

    let part_item: PartItem = sqlx::query_as(
        r#"
        INSERT INTO part_items (
            work_order_id,
            status,
            code,
            name,
            quantity,
            unit_type,
            base_cost,
            markup,
            discount_type,
            discount_value
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            id,
            work_order_id,
            status,
            code,
            name,
            quantity,
            unit_type,
            base_cost,
            markup,
            discount_type,
            discount_value
        "#,
    )
    .bind(payload.work_order_id)
    .bind(payload.status)
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.quantity.to_string())
    .bind(payload.unit_type)
    .bind(payload.base_cost.to_string())
    .bind(payload.markup.to_string())
    .bind(payload.discount_type)
    .bind(payload.discount_value.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_part_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdatePartItemPayload,
) -> AppResult<PartItem> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_item_work_order_active(&mut *tx, ItemTable::Part, id).await?;

    let part_item: PartItem = sqlx::query_as(
        r#"
        UPDATE part_items SET
            status = $2,
            code = $3,
            name = $4,
            quantity = $5,
            unit_type = $6,
            base_cost = $7,
            markup = $8,
            discount_type = $9,
            discount_value = $10
        WHERE id = $1
        RETURNING
            id,
            work_order_id,
            status,
            code,
            name,
            quantity,
            unit_type,
            base_cost,
            markup,
            discount_type,
            discount_value
        "#,
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
    .bind(payload.discount_value.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_part_item(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_item_work_order_active(&mut *tx, ItemTable::Part, id).await?;

    let result = sqlx::query("DELETE FROM part_items WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::part_item_not_found());
    }

    tx.commit().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_part_items(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        ensure_item_work_order_active(&mut *tx, ItemTable::Part, *id).await?;
        let result = sqlx::query("DELETE FROM part_items WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::part_item_not_found());
        }
    }
    tx.commit().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_part_items_status(
    state: State<'_, crate::AppState>,
    ids: Vec<i32>,
    status: PartStatus,
) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        ensure_item_work_order_active(&mut *tx, ItemTable::Part, *id).await?;
        let result = sqlx::query("UPDATE part_items SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::part_item_not_found());
        }
    }
    tx.commit().await?;
    Ok(())
}
