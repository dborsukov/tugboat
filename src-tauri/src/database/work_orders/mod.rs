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
        settings::structs::decode_company_info,
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
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat,
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
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat,
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
            discount_value,
            is_free
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
            discount_value,
            is_free
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
                    work_order.vat_rate,
                    work_order.prices_include_vat,
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
                let item_totals = pricing::calculate_part_item_totals(
                    &item,
                    work_order.vat_rate,
                    work_order.prices_include_vat,
                );
                PartItemDto {
                    item,
                    totals: item_totals,
                }
            })
            .collect();

        // Only billed items count: labor unless marked free, parts only when
        // installed and not marked free. Parts staged for installation form
        // the pending balance, returned parts never count.
        let labor_sum: Decimal = labor_item_dtos
            .iter()
            .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
            .map(|dto| dto.totals.total)
            .sum();
        let labor_cost_sum: Decimal = labor_item_dtos
            .iter()
            .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
            .map(|dto| dto.totals.cost)
            .sum();
        let labor_discount_sum: Decimal = labor_item_dtos
            .iter()
            .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
            .map(|dto| dto.totals.discount)
            .sum();
        let parts_sum: Decimal = part_item_dtos
            .iter()
            .filter(|dto| pricing::part_is_billed(dto.item.status, dto.item.is_free))
            .map(|dto| dto.totals.total)
            .sum();
        let parts_cost_sum: Decimal = part_item_dtos
            .iter()
            .filter(|dto| pricing::part_is_billed(dto.item.status, dto.item.is_free))
            .map(|dto| dto.totals.cost)
            .sum();
        let parts_discount_sum: Decimal = part_item_dtos
            .iter()
            .filter(|dto| pricing::part_is_billed(dto.item.status, dto.item.is_free))
            .map(|dto| dto.totals.discount)
            .sum();
        let parts_pending_sum: Decimal = part_item_dtos
            .iter()
            .filter(|dto| pricing::part_is_pending(dto.item.status, dto.item.is_free))
            .map(|dto| dto.totals.total)
            .sum();

        let totals = pricing::calculate_work_order_totals(
            labor_sum,
            labor_cost_sum,
            parts_sum,
            parts_cost_sum,
            parts_pending_sum,
            labor_discount_sum,
            parts_discount_sum,
            work_order.discount_type,
            work_order.discount_value,
            work_order.vat_rate,
            work_order.prices_include_vat,
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
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat,
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
            wo.type,
            wo.warranty_of,
            wo.rate,
            wo.labor_cost_type,
            wo.labor_cost_value,
            wo.discount_type,
            wo.discount_value,
            wo.vat_rate,
            wo.prices_include_vat,
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

async fn is_company_not_vat_payer(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> AppResult<bool> {
    let company_info = decode_company_info(
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT company_info FROM app_settings WHERE id = 1",
        )
        .fetch_optional(&mut **tx)
        .await?
        .flatten(),
    );
    Ok(match company_info {
        Some(info) => !info.vat_payer,
        // Company info not configured yet default to a non-payer
        None => true,
    })
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

    let vat_rate = if is_company_not_vat_payer(&mut tx).await? {
        Decimal::ZERO
    } else {
        payload.vat_rate
    };

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
            discount_value,
            vat_rate,
            prices_include_vat
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat,
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
    .bind(vat_rate.to_string())
    .bind(payload.prices_include_vat)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(work_order)
}

#[tauri::command]
#[specta::specta]
pub async fn create_warranty_work_order(
    state: State<'_, crate::AppState>,
    source_work_order_id: String,
    odometer_in: i32,
    description: Option<String>,
    notes: Option<String>,
) -> AppResult<WorkOrder> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let source: WorkOrder = sqlx::query_as(
        r#"
        SELECT
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat,
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
    .bind(&source_work_order_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(AppError::work_order_not_found)?;

    if source.archived_at.is_some() {
        return Err(AppError::work_order_archived());
    }
    if source.r#type != WorkOrderType::Regular {
        return Err(AppError::validation(
            "Гарантійне замовлення можна створити лише від звичайного замовлення.",
        ));
    }

    cars::ensure_car_active(&mut *tx, source.car_id as i32).await?;

    let vat_rate = if is_company_not_vat_payer(&mut tx).await? {
        Decimal::ZERO
    } else {
        source.vat_rate
    };

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
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat,
            created_at,
            archived_at,
            started_at,
            completed_at,
            paid_at,
            payment_method
        "#,
    )
    .bind(id)
    .bind(source.car_id)
    .bind(odometer_in)
    .bind(description)
    .bind(notes)
    .bind(WorkOrderStatus::Received)
    .bind(WorkOrderType::Warranty)
    .bind(&source_work_order_id)
    .bind(source.rate.to_string())
    .bind(source.labor_cost_type)
    .bind(source.labor_cost_value.to_string())
    .bind(DiscountType::None)
    .bind(Decimal::ZERO.to_string())
    .bind(vat_rate.to_string())
    .bind(source.prices_include_vat)
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
            vat_rate = $12,
            prices_include_vat = $13,
            started_at = $14,
            completed_at = $15,
            paid_at = $16,
            payment_method = $17
        WHERE id = $1
        RETURNING
            id,
            car_id,
            odometer_in,
            description,
            notes,
            status,
            type,
            warranty_of,
            rate,
            labor_cost_type,
            labor_cost_value,
            discount_type,
            discount_value,
            vat_rate,
            prices_include_vat,
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
    .bind(payload.vat_rate.to_string())
    .bind(payload.prices_include_vat)
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
    let warranty_claim_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_orders WHERE warranty_of = $1")
            .bind(&id)
            .fetch_one(&state.db_pool)
            .await?;
    if warranty_claim_count > 0 {
        return Err(AppError::validation(format!(
            "Спочатку видаліть пов'язані гарантійні замовлення ({warranty_claim_count})."
        )));
    }
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
    #[specta(type = specta_typescript::Number)]
    pub warranty_claim_count: i64,
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

    let warranty_claim_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_orders WHERE warranty_of = $1")
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;

    tx.commit().await?;

    Ok(WorkOrderDeleteImpact {
        labor_item_count,
        part_item_count,
        warranty_claim_count,
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
            discount_value,
            is_free
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
            discount_value,
            is_free
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
            discount_value,
            is_free
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            discount_value,
            is_free
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
    .bind(payload.is_free)
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
            discount_value = $9,
            is_free = $10
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
            discount_value,
            is_free
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
    .bind(payload.is_free)
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
            discount_value,
            is_free
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
            discount_value,
            is_free
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
            discount_value,
            is_free
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
            discount_value,
            is_free
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
    .bind(payload.is_free)
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
            discount_value = $10,
            is_free = $11
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
            discount_value,
            is_free
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
    .bind(payload.is_free)
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

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLaborItemPayload {
    pub work_order_id: String,
    pub price_type: LaborPriceType,
    pub fixed_price: Decimal,
    pub labor_hours: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub is_free: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLaborItemQuote {
    pub cost: Decimal,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    /// Money kept after VAT
    pub kept: Decimal,
    pub cost_basis_known: bool,
    pub below_cost: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn preview_labor_item(
    state: State<'_, crate::AppState>,
    payload: PreviewLaborItemPayload,
) -> AppResult<PreviewLaborItemQuote> {
    let work_order = fetch_work_order_by_id(&state, &payload.work_order_id).await?;

    let item = LaborItem {
        id: 0,
        service_id: None,
        work_order_id: payload.work_order_id.clone(),
        code: None,
        name: String::new(),
        price_type: payload.price_type,
        fixed_price: payload.fixed_price,
        labor_hours: payload.labor_hours,
        discount_type: payload.discount_type,
        discount_value: payload.discount_value,
        is_free: payload.is_free,
    };

    let totals = pricing::calculate_labor_item_totals(
        &item,
        work_order.rate,
        work_order.labor_cost_type,
        work_order.labor_cost_value,
        work_order.vat_rate,
        work_order.prices_include_vat,
    );

    Ok(PreviewLaborItemQuote {
        cost: totals.cost,
        subtotal: totals.subtotal,
        discount: totals.discount,
        total: totals.total,
        kept: pricing::net_kept_amount(
            totals.total,
            work_order.vat_rate,
            work_order.prices_include_vat,
        ),
        cost_basis_known: pricing::labor_cost_basis_known(
            item.price_type,
            work_order.labor_cost_type,
        ),
        below_cost: totals.below_cost,
    })
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPartItemPayload {
    pub work_order_id: String,
    pub base_cost: Decimal,
    pub markup: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub is_free: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPartItemQuote {
    pub cost: Decimal,
    pub retail_price: Decimal,
    pub discounted_price: Decimal,
    /// Money kept after VAT
    pub kept: Decimal,
    pub below_cost: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn preview_part_item(
    state: State<'_, crate::AppState>,
    payload: PreviewPartItemPayload,
) -> AppResult<PreviewPartItemQuote> {
    let work_order = fetch_work_order_by_id(&state, &payload.work_order_id).await?;

    let item = PartItem {
        id: 0,
        work_order_id: payload.work_order_id.clone(),
        status: PartStatus::Quoted,
        code: None,
        name: String::new(),
        quantity: Decimal::ONE,
        unit_type: String::new(),
        base_cost: payload.base_cost,
        markup: payload.markup,
        discount_type: payload.discount_type,
        discount_value: payload.discount_value,
        is_free: payload.is_free,
    };

    let totals = pricing::calculate_part_item_totals(
        &item,
        work_order.vat_rate,
        work_order.prices_include_vat,
    );

    Ok(PreviewPartItemQuote {
        cost: totals.cost,
        retail_price: totals.retail_price,
        discounted_price: totals.discounted_price,
        kept: pricing::net_kept_amount(
            totals.discounted_price,
            work_order.vat_rate,
            work_order.prices_include_vat,
        ),
        below_cost: totals.below_cost,
    })
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreviewWorkOrderPayload {
    pub work_order_id: String,
    pub rate: Decimal,
    pub labor_cost_type: LaborCostType,
    pub labor_cost_value: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub vat_rate: Decimal,
    pub prices_include_vat: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreviewWorkOrderQuote {
    pub labor_total: Decimal,
    pub labor_cost: Decimal,
    pub parts_total: Decimal,
    pub parts_cost: Decimal,
    pub combined_cost: Decimal,
    pub subtotal: Decimal,
    pub labor_discount: Decimal,
    pub parts_discount: Decimal,
    pub order_discount: Decimal,
    pub grand_total: Decimal,
    pub vat_base: Decimal,
    pub vat_amount: Decimal,
    pub total_with_vat: Decimal,
    pub below_cost: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn preview_work_order(
    state: State<'_, crate::AppState>,
    payload: PreviewWorkOrderPayload,
) -> AppResult<PreviewWorkOrderQuote> {
    // Validate the order exists, the preview itself is payload driven.
    fetch_work_order_by_id(&state, &payload.work_order_id).await?;

    let work_order_ids = [payload.work_order_id.as_str()];
    let mut labor_items_by_order =
        fetch_labor_items_by_work_order_ids(&state, &work_order_ids).await?;
    let mut part_items_by_order =
        fetch_part_items_by_work_order_ids(&state, &work_order_ids).await?;

    let labor_items = labor_items_by_order
        .remove(&payload.work_order_id)
        .unwrap_or_default();
    let part_items = part_items_by_order
        .remove(&payload.work_order_id)
        .unwrap_or_default();

    let labor_item_dtos: Vec<LaborItemDto> = labor_items
        .iter()
        .map(|item| {
            let item_totals = pricing::calculate_labor_item_totals(
                item,
                payload.rate,
                payload.labor_cost_type,
                payload.labor_cost_value,
                payload.vat_rate,
                payload.prices_include_vat,
            );
            LaborItemDto {
                item: item.clone(),
                totals: item_totals,
            }
        })
        .collect();

    let part_item_dtos: Vec<PartItemDto> = part_items
        .iter()
        .map(|item| {
            let item_totals = pricing::calculate_part_item_totals(
                item,
                payload.vat_rate,
                payload.prices_include_vat,
            );
            PartItemDto {
                item: item.clone(),
                totals: item_totals,
            }
        })
        .collect();

    let labor_sum: Decimal = labor_item_dtos
        .iter()
        .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
        .map(|dto| dto.totals.total)
        .sum();
    let labor_cost_sum: Decimal = labor_item_dtos
        .iter()
        .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
        .map(|dto| dto.totals.cost)
        .sum();
    let labor_discount_sum: Decimal = labor_item_dtos
        .iter()
        .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
        .map(|dto| dto.totals.discount)
        .sum();

    // The preview is a projected-final view: it counts every part that will
    // be billed if the order completes as composed (pending + installed,
    // excluding free parts), so the below-cost guard keeps working while
    // parts are still staged. Returned parts never count.
    let parts_sum: Decimal = part_item_dtos
        .iter()
        .filter(|dto| pricing::part_is_projected_billed(dto.item.status, dto.item.is_free))
        .map(|dto| dto.totals.total)
        .sum();
    let parts_cost_sum: Decimal = part_item_dtos
        .iter()
        .filter(|dto| pricing::part_is_projected_billed(dto.item.status, dto.item.is_free))
        .map(|dto| dto.totals.cost)
        .sum();
    let parts_discount_sum: Decimal = part_item_dtos
        .iter()
        .filter(|dto| pricing::part_is_projected_billed(dto.item.status, dto.item.is_free))
        .map(|dto| dto.totals.discount)
        .sum();
    let parts_pending_sum: Decimal = part_item_dtos
        .iter()
        .filter(|dto| pricing::part_is_pending(dto.item.status, dto.item.is_free))
        .map(|dto| dto.totals.total)
        .sum();

    let totals = pricing::calculate_work_order_totals(
        labor_sum,
        labor_cost_sum,
        parts_sum,
        parts_cost_sum,
        parts_pending_sum,
        labor_discount_sum,
        parts_discount_sum,
        payload.discount_type,
        payload.discount_value,
        payload.vat_rate,
        payload.prices_include_vat,
    );

    Ok(PreviewWorkOrderQuote {
        labor_total: totals.labor_total,
        labor_cost: totals.labor_cost,
        parts_total: totals.parts_total,
        parts_cost: totals.parts_cost,
        combined_cost: labor_cost_sum + parts_cost_sum,
        subtotal: totals.subtotal,
        labor_discount: totals.labor_discount,
        parts_discount: totals.parts_discount,
        order_discount: totals.order_discount,
        grand_total: totals.grand_total,
        vat_base: totals.vat_base,
        vat_amount: totals.vat_amount,
        total_with_vat: totals.total_with_vat,
        below_cost: totals.below_cost,
    })
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

#[tauri::command]
#[specta::specta]
pub async fn update_labor_items_discount(
    state: State<'_, crate::AppState>,
    ids: Vec<i32>,
    discount_type: DiscountType,
    discount_value: Decimal,
) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        ensure_item_work_order_active(&mut *tx, ItemTable::Labor, *id).await?;
        let result = sqlx::query(
            "UPDATE labor_items SET discount_type = $2, discount_value = $3 WHERE id = $1",
        )
        .bind(id)
        .bind(discount_type)
        .bind(discount_value.to_string())
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
pub async fn update_part_items_discount(
    state: State<'_, crate::AppState>,
    ids: Vec<i32>,
    discount_type: DiscountType,
    discount_value: Decimal,
) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        ensure_item_work_order_active(&mut *tx, ItemTable::Part, *id).await?;
        let result = sqlx::query(
            "UPDATE part_items SET discount_type = $2, discount_value = $3 WHERE id = $1",
        )
        .bind(id)
        .bind(discount_type)
        .bind(discount_value.to_string())
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
