//! Figures for the dashboard.
//!
//! Every money figure here is net of VAT.
//! `WorkOrderTotals::vat_base`, not `total_with_vat`.
//!
//! Two balance tiles are the exception, and say so on their own section: what a customer
//! still owes, and what is tied up in work under way, are the invoice values, VAT included.

pub mod structs;

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use structs::*;
use tauri::State;

use crate::{
    database::{
        cars::structs::car_display_name,
        clients::structs::{client_display_name, ClientType},
        work_orders::{
            self,
            dtos::{LaborItemDto, PartItemDto, WorkOrderTotals},
            pricing,
            structs::{WorkOrder, WorkOrderStatus, WorkOrderType},
        },
    },
    error::AppResult,
};

/// A timestamp formatted for sqlite.
fn sqlite_bound_timestamp(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S%.f").to_string()
}

/// SQLite time modifier shifting a stored UTC timestamp into specified timezone.
fn offset_modifier(utc_offset_minutes: i32) -> String {
    format!("{utc_offset_minutes:+} minutes")
}

/// One order's lines, priced with the shared rules.
///
/// The totals and the lines they were rolled up from are kept together: a figure per
/// line needs both, and pricing them twice could let the two disagree.
struct PricedOrder {
    totals: WorkOrderTotals,
    labor_items: Vec<LaborItemDto>,
    part_items: Vec<PartItemDto>,
}

/// Prices a set of orders with the shared pricing rules, keyed by order id.
async fn priced_orders(
    state: &State<'_, crate::AppState>,
    orders: &[WorkOrder],
) -> AppResult<HashMap<String, PricedOrder>> {
    let work_order_ids: Vec<&str> = orders.iter().map(|order| order.id.as_str()).collect();
    let mut labor_items_by_order =
        work_orders::fetch_labor_items_by_work_order_ids(state, &work_order_ids).await?;
    let mut part_items_by_order =
        work_orders::fetch_part_items_by_work_order_ids(state, &work_order_ids).await?;

    let mut priced_by_order = HashMap::with_capacity(orders.len());
    for order in orders {
        let labor_items: Vec<LaborItemDto> = labor_items_by_order
            .remove(&order.id)
            .unwrap_or_default()
            .iter()
            .map(|item| LaborItemDto {
                item: item.clone(),
                totals: pricing::calculate_labor_item_totals(
                    item,
                    order.rate,
                    order.labor_cost_type,
                    order.labor_cost_value,
                    order.vat_rate,
                    order.vat_scheme,
                    order.prices_include_vat,
                ),
            })
            .collect();
        let part_items: Vec<PartItemDto> = part_items_by_order
            .remove(&order.id)
            .unwrap_or_default()
            .iter()
            .map(|item| PartItemDto {
                item: item.clone(),
                totals: pricing::calculate_part_item_totals(
                    item,
                    order.vat_rate,
                    order.vat_scheme,
                    order.prices_include_vat,
                ),
            })
            .collect();

        let totals = pricing::calculate_order_totals(order, &labor_items, &part_items);
        priced_by_order.insert(
            order.id.clone(),
            PricedOrder {
                totals,
                labor_items,
                part_items,
            },
        );
    }

    Ok(priced_by_order)
}

/// Just the totals of a set of orders, keyed by order id.
async fn financials_for_orders(
    state: &State<'_, crate::AppState>,
    orders: &[WorkOrder],
) -> AppResult<HashMap<String, WorkOrderTotals>> {
    Ok(priced_orders(state, orders)
        .await?
        .into_iter()
        .map(|(id, order)| (id, order.totals))
        .collect())
}

/// Orders closed as paid within `range`, oldest first.
async fn paid_orders_closed_in(
    state: &State<'_, crate::AppState>,
    range: DateRange,
) -> AppResult<Vec<WorkOrder>> {
    let sql = format!(
        "SELECT {} FROM work_orders
         WHERE archived_at IS NULL AND status = ? AND paid_at >= ? AND paid_at < ?
         ORDER BY paid_at",
        work_orders::WORK_ORDER_COLUMNS
    );
    let orders: Vec<WorkOrder> = sqlx::query_as(&sql)
        .bind(WorkOrderStatus::Paid)
        .bind(sqlite_bound_timestamp(range.from))
        .bind(sqlite_bound_timestamp(range.to))
        .fetch_all(&state.db_pool)
        .await?;
    Ok(orders)
}

// Months the shop was paid in, offset by timezone.
// Drives which months are available in the picker.
#[tauri::command]
#[specta::specta]
pub async fn fetch_order_months(
    state: State<'_, crate::AppState>,
    utc_offset_minutes: i32,
) -> AppResult<Vec<OrderMonth>> {
    let rows: Vec<(i64, i64, i64)> = sqlx::query_as(
        r#"
        WITH paid_orders AS (
            SELECT datetime(paid_at, ?) AS local_at
            FROM work_orders
            WHERE archived_at IS NULL
              AND status = ?
              AND paid_at IS NOT NULL
        )
        SELECT
            CAST(strftime('%Y', local_at) AS INTEGER) AS year,
            CAST(strftime('%m', local_at) AS INTEGER) AS month,
            COUNT(*) AS order_count
        FROM paid_orders
        GROUP BY year, month
        ORDER BY year, month
        "#,
    )
    .bind(offset_modifier(utc_offset_minutes))
    .bind(WorkOrderStatus::Paid)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(year, month, order_count)| OrderMonth {
            year: year as i32,
            month: month as i32,
            order_count: order_count as i32,
        })
        .collect())
}

// Outstanding balance
// Money for the work that is finished (or ready) but not paid for. The invoice the
// customer will settle, VAT included: money owed rather than money earned.
#[tauri::command]
#[specta::specta]
pub async fn fetch_outstanding_balance(
    state: State<'_, crate::AppState>,
) -> AppResult<OutstandingBalance> {
    let sql = format!(
        "SELECT {} FROM work_orders
         WHERE archived_at IS NULL AND status IN (?, ?)",
        work_orders::WORK_ORDER_COLUMNS
    );
    let orders: Vec<WorkOrder> = sqlx::query_as(&sql)
        .bind(WorkOrderStatus::ReadyForPickup)
        .bind(WorkOrderStatus::Completed)
        .fetch_all(&state.db_pool)
        .await?;

    let totals_by_order = financials_for_orders(&state, &orders).await?;

    let mut total = Decimal::ZERO;
    let mut order_count = 0;
    for order in &orders {
        if order.r#type == WorkOrderType::Warranty {
            continue;
        }
        let Some(totals) = totals_by_order.get(&order.id) else {
            continue;
        };
        if totals.total_with_vat.is_zero() {
            continue;
        }
        total += totals.total_with_vat;
        order_count += 1;
    }

    Ok(OutstandingBalance { total, order_count })
}

// In progress balance
// Money projected for the orders currently in progess, valued at the invoice they are
// expected to raise, VAT included, the same basis as the outstanding balance.
#[tauri::command]
#[specta::specta]
pub async fn fetch_in_progress_balance(
    state: State<'_, crate::AppState>,
) -> AppResult<InProgressBalance> {
    let sql = format!(
        "SELECT {} FROM work_orders
         WHERE archived_at IS NULL AND status NOT IN (?, ?, ?, ?)",
        work_orders::WORK_ORDER_COLUMNS
    );
    let orders: Vec<WorkOrder> = sqlx::query_as(&sql)
        .bind(WorkOrderStatus::Canceled)
        .bind(WorkOrderStatus::Paid)
        .bind(WorkOrderStatus::Completed)
        .bind(WorkOrderStatus::ReadyForPickup)
        .fetch_all(&state.db_pool)
        .await?;

    let totals_by_order = financials_for_orders(&state, &orders).await?;

    let mut total = Decimal::ZERO;
    let mut order_count = 0;
    for order in &orders {
        if order.r#type == WorkOrderType::Warranty {
            continue;
        }
        let Some(totals) = totals_by_order.get(&order.id) else {
            continue;
        };
        if totals.total_with_vat.is_zero() {
            continue;
        }
        total += totals.total_with_vat;
        order_count += 1;
    }

    Ok(InProgressBalance { total, order_count })
}

// Revenue
// What the orders closed in `range` earned, net of VAT.
async fn revenue_bucket(
    state: &State<'_, crate::AppState>,
    range: DateRange,
) -> AppResult<RevenueBucket> {
    let orders = paid_orders_closed_in(state, range).await?;
    let totals_by_order = financials_for_orders(state, &orders).await?;

    let mut total = Decimal::ZERO;
    let mut order_count = 0;
    for order in &orders {
        let Some(totals) = totals_by_order.get(&order.id) else {
            continue;
        };
        total += totals.vat_base;
        order_count += 1;
    }

    Ok(RevenueBucket { total, order_count })
}

// Output VAT recorded in the ledger for this month, by tax point.
#[tauri::command]
#[specta::specta]
pub async fn fetch_vat_due(
    state: State<'_, crate::AppState>,
    period: AnalyticsPeriod,
) -> AppResult<VatDue> {
    let settings = crate::database::settings::fetch_settings_from_pool(&state.db_pool).await?;
    let timezone: chrono_tz::Tz = settings
        .timezone
        .parse()
        .unwrap_or(chrono_tz::Tz::Europe__Kiev);

    let from = period
        .month
        .from
        .with_timezone(&timezone)
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let to = period
        .month
        .to
        .with_timezone(&timezone)
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();

    let sql = format!(
        "SELECT {} FROM vat_entries
         WHERE direction = 'output' AND tax_point >= $1 AND tax_point < $2
         ORDER BY tax_point, id",
        crate::database::vat::VAT_ENTRY_COLUMNS
    );
    let entries: Vec<crate::database::vat::structs::VatEntry> = sqlx::query_as(&sql)
        .bind(from)
        .bind(to)
        .fetch_all(&state.db_pool)
        .await?;

    let amount: Decimal = entries.iter().map(|entry| entry.amount).sum();
    let order_count = entries
        .iter()
        .filter(|entry| entry.reverses_id.is_none())
        .map(|entry| entry.source_id.as_str())
        .collect::<HashSet<_>>()
        .len() as i32;

    Ok(VatDue {
        amount,
        order_count,
    })
}

/// Which day of `from`'s window an instant falls on, first day being 1.
///
/// Counted in whole days from the start of the window rather than resolved as a
/// calendar date: a day is 23 or 25 hours long across a DST change, and an hour of
/// disagreement about where midnight sits is worth less than the guarantee that every
/// order inside the window lands in a day of it.
fn day_of_window(at: DateTime<Utc>, from: DateTime<Utc>) -> usize {
    (at - from).num_days().max(0) as usize + 1
}

/// Splits one order's revenue between what labor brought in and what parts did.
///
/// Line totals are measured before the order level discount, and each side carries a
/// discount of its own, so neither side's own total is what the order billed. Each side
/// is credited with its proportional share of what the order did earn
/// (`vat_base / subtotal`), which is what makes the two add up to the revenue the module
/// shows for the same order.
fn attribute_revenue(totals: &WorkOrderTotals) -> RevenueSplit {
    let subtotal = totals.labor_total + totals.parts_total;
    let billed = if subtotal.is_zero() {
        // Nothing was billed, so neither side earned anything.
        Decimal::ZERO
    } else {
        totals.vat_base / subtotal
    };

    RevenueSplit {
        labor: (totals.labor_total * billed).round_dp(2),
        parts: (totals.parts_total * billed).round_dp(2),
    }
}

/// A month of revenue in the shape the tile draws it: the bucket, the running total of
/// every day it has money on, and what that money was earned on.
struct RevenueMonth {
    bucket: RevenueBucket,
    by_day: Vec<Decimal>,
    split: RevenueSplit,
}

async fn revenue_month(
    state: &State<'_, crate::AppState>,
    range: DateRange,
) -> AppResult<RevenueMonth> {
    let orders = paid_orders_closed_in(state, range).await?;
    let totals_by_order = financials_for_orders(state, &orders).await?;

    let mut total = Decimal::ZERO;
    let mut order_count = 0;
    let mut per_day: Vec<Decimal> = Vec::new();
    let mut labor = Decimal::ZERO;
    let mut parts = Decimal::ZERO;

    for order in &orders {
        let Some(totals) = totals_by_order.get(&order.id) else {
            continue;
        };
        total += totals.vat_base;
        order_count += 1;

        if let Some(paid_at) = order.paid_at {
            let day = day_of_window(paid_at, range.from);
            per_day.resize(per_day.len().max(day), Decimal::ZERO);
            per_day[day - 1] += totals.vat_base;
        }

        let split = attribute_revenue(totals);
        labor += split.labor;
        parts += split.parts;
    }

    // The tile draws a running total, so fold the days once here rather than on every
    // redraw. The last entry is the bucket's own total.
    let mut running = Decimal::ZERO;
    for day in &mut per_day {
        running += *day;
        *day = running;
    }

    Ok(RevenueMonth {
        bucket: RevenueBucket { total, order_count },
        by_day: per_day,
        split: RevenueSplit { labor, parts },
    })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_revenue_summary(
    state: State<'_, crate::AppState>,
    period: AnalyticsPeriod,
) -> AppResult<RevenueSummary> {
    let month = revenue_month(&state, period.month).await?;
    let previous_month = revenue_month(&state, period.previous_month).await?;

    Ok(RevenueSummary {
        month: month.bucket,
        previous_month: previous_month.bucket,
        week: revenue_bucket(&state, period.week).await?,
        previous_week: revenue_bucket(&state, period.previous_week).await?,
        month_by_day: month.by_day,
        previous_month_by_day: previous_month.by_day,
        month_split: month.split,
    })
}

/// How many services the report may carry: more than a tile draws, so how many it shows
/// is the tile's business rather than the query's.
const TOP_SERVICES_MAX: usize = 9;

/// Times each service was billed in `range`, by the name on the line.
/// Work paid for in the window, which is the same set of orders the money tiles count.
async fn service_counts(
    state: &State<'_, crate::AppState>,
    range: DateRange,
) -> AppResult<HashMap<String, i32>> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT TRIM(li.name) AS name, COUNT(*) AS count
        FROM labor_items li
        JOIN work_orders wo ON wo.id = li.work_order_id
        WHERE wo.archived_at IS NULL
          AND wo.status = ?
          AND wo.paid_at >= ?
          AND wo.paid_at < ?
          AND li.is_free = 0
        GROUP BY TRIM(li.name)
        "#,
    )
    .bind(WorkOrderStatus::Paid)
    .bind(sqlite_bound_timestamp(range.from))
    .bind(sqlite_bound_timestamp(range.to))
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(name, count)| (name, count as i32))
        .collect())
}

/// What each service of `range` was billed and how often, biggest earner first.
#[tauri::command]
#[specta::specta]
pub async fn fetch_top_services(
    state: State<'_, crate::AppState>,
    period: AnalyticsPeriod,
) -> AppResult<TopServicesReport> {
    let orders = paid_orders_closed_in(&state, period.month).await?;
    let priced_by_order = priced_orders(&state, &orders).await?;
    let previous_counts = service_counts(&state, period.previous_month).await?;

    let mut by_service: HashMap<String, (i32, Decimal)> = HashMap::new();
    let mut total_revenue = Decimal::ZERO;

    for order in &orders {
        let Some(priced) = priced_by_order.get(&order.id) else {
            continue;
        };
        let totals = &priced.totals;
        total_revenue += totals.vat_base;

        // What the order kept of what its lines were billed: below one when the order
        // carried a discount.
        let kept = if totals.subtotal.is_zero() {
            Decimal::ZERO
        } else {
            totals.vat_base / totals.subtotal
        };

        for dto in priced
            .labor_items
            .iter()
            .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
        {
            let name = dto.item.name.trim().to_string();
            if name.is_empty() {
                continue;
            }

            let billed = (dto.totals.total * kept).round_dp(2);
            let figures = by_service.entry(name).or_insert((0, Decimal::ZERO));
            figures.0 += 1;
            figures.1 += billed;
        }
    }

    let mut services: Vec<TopServiceRow> = by_service
        .into_iter()
        .map(|(name, (count, revenue))| {
            let average_price = if count == 0 {
                Decimal::ZERO
            } else {
                (revenue / Decimal::from(count as u64)).round_dp(2)
            };
            let before = previous_counts.get(&name).copied().unwrap_or(0);

            TopServiceRow {
                name,
                count,
                revenue,
                average_price,
                // Nothing rather than zero when last month had none of it: a service that
                // was not there cannot have grown by nothing, it is new.
                change: (before > 0).then(|| count - before),
            }
        })
        .collect();

    // What every service of the month billed between them, including any the tile will
    // have no room to list: a share of the work is a share of all of it.
    let services_revenue = services.iter().map(|service| service.revenue).sum();

    // Most earned first; the name breaks a tie so an unchanged month lists in the same
    // order every time it is asked.
    services.sort_by(|left, right| {
        right
            .revenue
            .cmp(&left.revenue)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.name.cmp(&right.name))
    });
    services.truncate(TOP_SERVICES_MAX);

    Ok(TopServicesReport {
        total_revenue,
        services_revenue,
        services,
    })
}

// New versus returning clients
// Clients served this month, split into first-timers and regulars.
async fn client_mix_counts(
    state: &State<'_, crate::AppState>,
    range: DateRange,
) -> AppResult<(i32, i32)> {
    let row: (i64, i64) = sqlx::query_as(
        r#"
        WITH period_clients AS (
            SELECT DISTINCT COALESCE(wo.customer_client_id, car.client_id) AS client_id
            FROM work_orders wo
            JOIN cars car ON car.id = wo.car_id
            WHERE wo.archived_at IS NULL
              AND wo.status = ?
              AND wo.paid_at >= ?
              AND wo.paid_at < ?
        ),
        first_payments AS (
            SELECT
                COALESCE(wo.customer_client_id, car.client_id) AS client_id,
                MIN(wo.paid_at) AS first_at
            FROM work_orders wo
            JOIN cars car ON car.id = wo.car_id
            WHERE wo.archived_at IS NULL AND wo.status = ?
            GROUP BY COALESCE(wo.customer_client_id, car.client_id)
        )
        SELECT
            COUNT(*) AS total,
            COALESCE(SUM(CASE WHEN f.first_at >= ? THEN 1 ELSE 0 END), 0) AS new_clients
        FROM period_clients p
        JOIN first_payments f ON f.client_id = p.client_id
        "#,
    )
    .bind(WorkOrderStatus::Paid)
    .bind(sqlite_bound_timestamp(range.from))
    .bind(sqlite_bound_timestamp(range.to))
    .bind(WorkOrderStatus::Paid)
    .bind(sqlite_bound_timestamp(range.from))
    .fetch_one(&state.db_pool)
    .await?;

    Ok((row.0 as i32, row.1 as i32))
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_client_mix(
    state: State<'_, crate::AppState>,
    period: AnalyticsPeriod,
) -> AppResult<ClientMix> {
    let (total_clients, new_clients) = client_mix_counts(&state, period.month).await?;
    let (previous_total_clients, _) = client_mix_counts(&state, period.previous_month).await?;

    Ok(ClientMix {
        new_clients,
        returning_clients: total_clients - new_clients,
        total_clients,
        previous_total_clients,
    })
}

/// One order's client, and the car they came in.
#[derive(Debug, sqlx::FromRow)]
struct ClientMixOrderRow {
    order_id: String,
    client_id: i64,
    name: String,
    surname: Option<String>,
    first_name: Option<String>,
    patronymic: Option<String>,
    client_type: ClientType,
    /// The car itself, so the same model coming in twice is still one car.
    car_id: i64,
    brand: Option<String>,
    model: String,
    /// The client's earliest payment ever, which is what makes this month their first.
    first_at: Option<DateTime<Utc>>,
}

impl ClientMixOrderRow {
    fn display_name(&self) -> String {
        client_display_name(
            self.client_type,
            &self.name,
            self.surname.as_deref(),
            self.first_name.as_deref(),
            self.patronymic.as_deref(),
        )
    }

    fn display_car(&self) -> String {
        car_display_name(self.brand.as_deref(), &self.model)
    }
}

/// How many clients the list may carry: more than a tile draws, so how many it shows is
/// the tile's business rather than the query's.
const CLIENT_MIX_LIST_MAX: usize = 8;

/// The clients behind the period's mix, biggest spender first.
///
/// The orders are the ones paid for in the month — the set the revenue tile counts — so
/// the list, the counts above it and the money tiles all speak of the same visits.
///
/// An order is credited to whoever placed it — the client on the order, falling back to
/// the car's owner — the same rule the counts are worked out by, so a client can never
/// appear in the list without being counted above it.
///
/// Visits and what was spent are the period's own: the tile lists who paid *this month*,
/// so a row says what that client settled in it rather than what they have ever done.
#[tauri::command]
#[specta::specta]
pub async fn fetch_client_mix_list(
    state: State<'_, crate::AppState>,
    period: AnalyticsPeriod,
) -> AppResult<ClientMixList> {
    let orders = paid_orders_closed_in(&state, period.month).await?;
    let totals_by_order = financials_for_orders(&state, &orders).await?;

    let rows: Vec<ClientMixOrderRow> = sqlx::query_as(
        r#"
        SELECT
            wo.id AS order_id,
            c.id AS client_id,
            c.name AS name,
            c.surname AS surname,
            c.first_name AS first_name,
            c.patronymic AS patronymic,
            c.client_type AS client_type,
            car.id AS car_id,
            b.name AS brand,
            car.model AS model,
            (
                SELECT MIN(wo2.paid_at)
                FROM work_orders wo2
                JOIN cars car2 ON car2.id = wo2.car_id
                WHERE COALESCE(wo2.customer_client_id, car2.client_id) = c.id
                  AND wo2.archived_at IS NULL
                  AND wo2.status = ?
            ) AS first_at
        FROM work_orders wo
        JOIN cars car ON car.id = wo.car_id
        JOIN clients c ON c.id = COALESCE(wo.customer_client_id, car.client_id)
        LEFT JOIN car_brands b ON b.id = car.brand_id
        WHERE wo.archived_at IS NULL
          AND wo.status = ?
          AND wo.paid_at >= ?
          AND wo.paid_at < ?
        -- Earliest payment first, which is what makes a client's first order of the month
        -- the one their row is built from. The id breaks a tie: two orders settled in the
        -- same second have no order of their own, and which car a client is listed under
        -- must not change from one asking of this query to the next.
        ORDER BY wo.paid_at, wo.id
        "#,
    )
    .bind(WorkOrderStatus::Paid)
    .bind(WorkOrderStatus::Paid)
    .bind(sqlite_bound_timestamp(period.month.from))
    .bind(sqlite_bound_timestamp(period.month.to))
    .fetch_all(&state.db_pool)
    .await?;

    /// One client's month, as it is folded together from their orders.
    struct ClientTotals {
        name: String,
        car: String,
        /// Every car they came in this month, so the row can say how many others there
        /// were beside the one it names.
        cars: HashSet<i64>,
        visits: i32,
        spent: Decimal,
        first_at: Option<DateTime<Utc>>,
    }

    let mut by_client: HashMap<i64, ClientTotals> = HashMap::new();
    let mut month_spent = Decimal::ZERO;

    for row in rows {
        let spent = totals_by_order
            .get(&row.order_id)
            .map(|totals| totals.vat_base)
            .unwrap_or(Decimal::ZERO);
        month_spent += spent;

        by_client
            .entry(row.client_id)
            .and_modify(|client| {
                client.visits += 1;
                client.spent += spent;
                client.cars.insert(row.car_id);
            })
            .or_insert_with(|| ClientTotals {
                name: row.display_name(),
                car: row.display_car(),
                cars: HashSet::from([row.car_id]),
                visits: 1,
                spent,
                first_at: row.first_at,
            });
    }

    let clients: Vec<ClientMixEntry> = by_client
        .into_iter()
        .map(|(id, client)| ClientMixEntry {
            id,
            name: client.name,
            car: client.car,
            other_cars: client.cars.len().saturating_sub(1) as i32,
            visits: client.visits,
            spent: client.spent,
            is_new: client
                .first_at
                .is_some_and(|first_at| first_at >= period.month.from),
        })
        .collect();

    Ok(ClientMixList {
        average_check: average_of(month_spent, clients.len()),
        clients: rank_clients(clients),
    })
}

/// What one client was worth on average.
fn average_of(total: Decimal, clients: usize) -> Decimal {
    if clients == 0 {
        return Decimal::ZERO;
    }

    (total / Decimal::from(clients as u64)).round_dp(2)
}

/// The list as a tile draws it: most spent first, then most visits, then by name so an
/// unchanged month lists in the same order every time it is asked.
fn rank_clients(mut clients: Vec<ClientMixEntry>) -> Vec<ClientMixEntry> {
    clients.sort_by(|left, right| {
        right
            .spent
            .cmp(&left.spent)
            .then_with(|| right.visits.cmp(&left.visits))
            .then_with(|| left.name.cmp(&right.name))
    });
    clients.truncate(CLIENT_MIX_LIST_MAX);

    clients
}

struct MarginFigures {
    revenue: Decimal,
    cost: Decimal,
    /// Orders these figures were calculated from, and the orders the period had.
    orders: i32,
    period_orders: i32,
}

impl MarginFigures {
    fn income(&self) -> Decimal {
        self.revenue - self.cost
    }
    fn percent(&self) -> f64 {
        if self.revenue.is_zero() {
            return 0.0;
        }
        let margin = (self.revenue - self.cost) / self.revenue * Decimal::ONE_HUNDRED;
        margin.to_f64().unwrap_or(0.0)
    }
}

/// What one order costs, under the options the tile was set up with.
fn order_cost(totals: &WorkOrderTotals, options: MarginOptions) -> Decimal {
    if options.include_labor_cost {
        totals.labor_cost + totals.parts_cost
    } else {
        totals.parts_cost
    }
}

/// Folds a period's order totals into the figures the tiles show.
fn figures_from<'a>(
    totals: impl Iterator<Item = &'a WorkOrderTotals>,
    options: MarginOptions,
) -> MarginFigures {
    let mut revenue = Decimal::ZERO;
    let mut cost = Decimal::ZERO;
    let mut orders = 0;
    let mut period_orders = 0;
    for totals in totals {
        let cost_of_order = order_cost(totals, options);

        period_orders += 1;
        revenue += totals.vat_base;
        cost += cost_of_order;

        // An order that billed nothing and cost nothing has no figures to speak for:
        // counting it would only make the tile claim a coverage it does not have. One
        // that cost something without billing anything still moves the income, so it stays.
        if !totals.vat_base.is_zero() || !cost_of_order.is_zero() {
            orders += 1;
        }
    }

    MarginFigures {
        revenue,
        cost,
        orders,
        period_orders,
    }
}

async fn margin_figures(
    state: &State<'_, crate::AppState>,
    range: DateRange,
    options: MarginOptions,
) -> AppResult<MarginFigures> {
    let orders = paid_orders_closed_in(state, range).await?;
    let totals_by_order = financials_for_orders(state, &orders).await?;

    Ok(figures_from(
        orders
            .iter()
            .filter_map(|order| totals_by_order.get(&order.id)),
        options,
    ))
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_gross_margin(
    state: State<'_, crate::AppState>,
    period: AnalyticsPeriod,
    options: MarginOptions,
) -> AppResult<MarginSummary> {
    let current = margin_figures(&state, period.month, options).await?;
    let previous = margin_figures(&state, period.previous_month, options).await?;

    Ok(MarginSummary {
        revenue: current.revenue,
        cost: current.cost,
        income: current.income(),
        margin_percent: current.percent(),
        previous_revenue: previous.revenue,
        previous_income: previous.income(),
        previous_margin_percent: previous.percent(),
        order_count: current.orders,
        period_order_count: current.period_orders,
    })
}

/// How many lines the breakdown may list: more than the module draws, so how many it
/// shows is the module's business rather than the query's.
const MARGIN_SERVICES_MAX: usize = 6;

/// One billed line of an order, before the order's discount and VAT are shared out.
struct MarginLine {
    name: String,
    /// What the line was billed, before the order's own discount and tax.
    billed: Decimal,
    /// What the line cost the shop, under the tile's own rule for what counts as cost.
    cost: Decimal,
}

/// Folds one line into the name's running figures.
struct ServiceFigures {
    revenue: Decimal,
    cost: Decimal,
}

impl ServiceFigures {
    fn income(&self) -> Decimal {
        self.revenue - self.cost
    }
    fn percent(&self) -> f64 {
        if self.revenue.is_zero() {
            return 0.0;
        }
        let margin = (self.revenue - self.cost) / self.revenue * Decimal::ONE_HUNDRED;
        margin.to_f64().unwrap_or(0.0)
    }
}

/// What each billed line of `range` earned and what it cost, biggest contribution first.
#[tauri::command]
#[specta::specta]
pub async fn fetch_margin_services(
    state: State<'_, crate::AppState>,
    period: AnalyticsPeriod,
    options: MarginOptions,
) -> AppResult<Vec<MarginService>> {
    let orders = paid_orders_closed_in(&state, period.month).await?;
    let priced_by_order = priced_orders(&state, &orders).await?;

    let mut by_name: HashMap<String, ServiceFigures> = HashMap::new();
    for order in &orders {
        let Some(priced) = priced_by_order.get(&order.id) else {
            continue;
        };
        let totals = &priced.totals;

        // What the order kept of what its lines were billed: below one when the order
        // carried a discount, and the VAT share of every line besides.
        let net_subtotal =
            pricing::net_kept_amount(totals.subtotal, order.vat_rate, order.prices_include_vat);
        let kept = if net_subtotal.is_zero() {
            Decimal::ZERO
        } else {
            totals.vat_base / net_subtotal
        };

        let mut lines: Vec<MarginLine> = priced
            .labor_items
            .iter()
            .filter(|dto| pricing::labor_is_billed(dto.item.is_free))
            .map(|dto| MarginLine {
                name: dto.item.name.trim().to_string(),
                billed: dto.totals.total,
                // What the work cost is counted as far as the switch counts it at all.
                cost: if options.include_labor_cost {
                    dto.totals.cost
                } else {
                    Decimal::ZERO
                },
            })
            .chain(
                priced
                    .part_items
                    .iter()
                    .filter(|dto| pricing::part_is_billed(dto.item.status, dto.item.is_free))
                    .map(|dto| MarginLine {
                        name: dto.item.name.trim().to_string(),
                        billed: dto.totals.total,
                        // A part costs what was paid for it whether the work does or not.
                        cost: dto.totals.cost,
                    }),
            )
            .collect();

        for line in lines.drain(..) {
            if line.name.is_empty() {
                continue;
            }

            let revenue =
                (pricing::net_kept_amount(line.billed, order.vat_rate, order.prices_include_vat)
                    * kept)
                    .round_dp(2);

            let figures = by_name.entry(line.name).or_insert(ServiceFigures {
                revenue: Decimal::ZERO,
                cost: Decimal::ZERO,
            });
            figures.revenue += revenue;
            figures.cost += line.cost;
        }
    }

    let mut services: Vec<MarginService> = by_name
        .into_iter()
        .map(|(name, figures)| MarginService {
            name,
            revenue: figures.revenue,
            cost: figures.cost,
            income: figures.income(),
            margin_percent: figures.percent(),
        })
        .collect();

    // Biggest contribution first; the name breaks a tie so an unchanged month lists
    // in the same order every time it is asked.
    services.sort_by(|left, right| {
        right
            .income
            .cmp(&left.income)
            .then_with(|| right.revenue.cmp(&left.revenue))
            .then_with(|| left.name.cmp(&right.name))
    });
    services.truncate(MARGIN_SERVICES_MAX);

    Ok(services)
}
