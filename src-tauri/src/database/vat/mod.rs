//! The VAT ledger.
//!
//! VAT is not derived on every read, a supply is priced once, at its tax
//! point, and the figures are stored here. Reports read these rows, so editing
//! an order afterwards cannot rewrite a period that was already reported.
//!
//! A correction never deletes a row. The original gets `voided_at`, and a
//! separate reversal row carries the negated amounts.
//!
//! One supply may produce several rows, an order with two rates, or with a
//! standard and an exempt line, gets one row per rate and treatment.

pub mod structs;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{Executor, Pool, Sqlite, SqliteConnection};
use structs::*;
use tauri::State;

use crate::{
    database::{
        clients::structs::ClientType,
        expenses::structs::Expense,
        settings::fetch_settings_from_pool,
        utils::begin_immediate,
        work_orders::{
            self,
            dtos::{LaborItemDto, PartItemDto},
            pricing,
            structs::{LaborItem, PartItem, WorkOrder, WorkOrderStatus},
        },
    },
    error::{AppError, AppResult},
    AppState,
};

pub(crate) const VAT_ENTRY_COLUMNS: &str = r#"
    id,
    direction,
    tax_point,
    source_kind,
    source_id,
    document_number,
    document_date,
    exemption_code,
    counterparty_name,
    counterparty_code,
    counterparty_vat_payer,
    base,
    rate,
    amount,
    scheme,
    reverses_id,
    voided_at,
    created_at
"#;

fn timestamp_text() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn date_text(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn parse_day(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()
}

pub(crate) fn status_is_supplied(status: WorkOrderStatus) -> bool {
    matches!(
        status,
        WorkOrderStatus::ReadyForPickup | WorkOrderStatus::Completed | WorkOrderStatus::Paid
    )
}

/// The day VAT is due for an order, in the business time zone.
pub(crate) fn tax_point_of(
    completed_at: Option<DateTime<Utc>>,
    paid_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    timezone: chrono_tz::Tz,
) -> NaiveDate {
    let local = |value: DateTime<Utc>| value.with_timezone(&timezone).date_naive();
    match (completed_at, paid_at) {
        (Some(completed), Some(paid)) => local(completed).min(local(paid)),
        (Some(completed), None) => local(completed),
        (None, Some(paid)) => local(paid),
        (None, None) => local(created_at),
    }
}

fn tax_point_for(order: &WorkOrder, timezone: chrono_tz::Tz) -> NaiveDate {
    tax_point_of(
        order.completed_at,
        order.paid_at,
        order.created_at,
        timezone,
    )
}

pub(crate) fn validate_rate_and_scheme(rate: Decimal, scheme: VatScheme) -> AppResult<()> {
    if rate.is_sign_negative() || rate > Decimal::ONE_HUNDRED {
        return Err(AppError::validation("Ставка ПДВ має бути від 0 до 100"));
    }
    if !scheme.is_standard() && !rate.is_zero() {
        return Err(AppError::validation("Для пільгової ставки ПДВ вкажіть 0 %"));
    }
    Ok(())
}

pub(crate) async fn ensure_vat_period_open<'e, E>(
    executor: E,
    tax_point: NaiveDate,
) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    if vat_period_is_open(executor, tax_point).await? {
        return Ok(());
    }
    Err(AppError::validation(format!(
        "Період ПДВ {} закрито. Відкрийте його в журналі звітів, щоб записати податкову подію.",
        tax_point.format("%Y-%m")
    )))
}

async fn vat_period_is_open<'e, E>(executor: E, tax_point: NaiveDate) -> AppResult<bool>
where
    E: Executor<'e, Database = Sqlite>,
{
    let month = tax_point.format("%Y-%m").to_string();
    let filed: Option<String> =
        sqlx::query_scalar("SELECT filed_at FROM vat_periods WHERE month = $1")
            .bind(month)
            .fetch_optional(executor)
            .await?;
    Ok(filed.is_none())
}

pub(crate) async fn live_entries_for_order<'e, E>(
    executor: E,
    order_id: &str,
) -> AppResult<Vec<VatEntry>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let sql = format!(
        "SELECT {VAT_ENTRY_COLUMNS} FROM vat_entries
         WHERE direction = 'output'
           AND source_kind = 'work_order'
           AND source_id = $1
           AND reverses_id IS NULL
           AND voided_at IS NULL
         ORDER BY rate, scheme"
    );
    Ok(sqlx::query_as(&sql)
        .bind(order_id)
        .fetch_all(executor)
        .await?)
}

/// The first live entry of an order
pub(crate) async fn live_entry_for_order<'e, E>(
    executor: E,
    order_id: &str,
) -> AppResult<Option<VatEntry>>
where
    E: Executor<'e, Database = Sqlite>,
{
    Ok(live_entries_for_order(executor, order_id)
        .await?
        .into_iter()
        .next())
}

pub(crate) async fn ensure_vat_unlocked<'e, E>(executor: E, order_id: &str) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    if live_entry_for_order(executor, order_id).await?.is_some() {
        return Err(AppError::vat_locked());
    }
    Ok(())
}

pub(crate) async fn live_entry_for_expense<'e, E>(
    executor: E,
    expense_id: i64,
) -> AppResult<Option<VatEntry>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let sql = format!(
        "SELECT {VAT_ENTRY_COLUMNS} FROM vat_entries
         WHERE direction = 'input'
           AND source_kind = 'expense'
           AND source_id = $1
           AND reverses_id IS NULL
           AND voided_at IS NULL"
    );
    Ok(sqlx::query_as(&sql)
        .bind(expense_id.to_string())
        .fetch_optional(executor)
        .await?)
}

pub(crate) async fn ensure_expense_vat_unlocked<'e, E>(
    executor: E,
    expense_id: i64,
) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    if live_entry_for_expense(executor, expense_id)
        .await?
        .is_some()
    {
        return Err(AppError::expense_vat_locked());
    }
    Ok(())
}

pub(crate) async fn live_entry_for_part<'e, E>(
    executor: E,
    part_id: i64,
) -> AppResult<Option<VatEntry>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let sql = format!(
        "SELECT {VAT_ENTRY_COLUMNS} FROM vat_entries
         WHERE direction = 'input'
           AND source_kind = 'part_item'
           AND source_id = $1
           AND reverses_id IS NULL
           AND voided_at IS NULL"
    );
    Ok(sqlx::query_as(&sql)
        .bind(part_id.to_string())
        .fetch_optional(executor)
        .await?)
}

pub(crate) async fn ensure_part_vat_unlocked<'e, E>(executor: E, part_id: i64) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    if live_entry_for_part(executor, part_id).await?.is_some() {
        return Err(AppError::part_vat_locked());
    }
    Ok(())
}

struct VatCounterparty {
    name: String,
    code: Option<String>,
    vat_payer: bool,
}

#[derive(sqlx::FromRow)]
struct CounterpartyRow {
    name: String,
    surname: Option<String>,
    first_name: Option<String>,
    patronymic: Option<String>,
    client_type: ClientType,
    legal_name: Option<String>,
    edrpou: Option<String>,
    rnokpp: Option<String>,
    vat_payer: Option<bool>,
}

async fn counterparty_for_order<'e, E>(
    executor: E,
    customer_client_id: Option<i32>,
    car_id: i64,
) -> AppResult<VatCounterparty>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row: Option<CounterpartyRow> = sqlx::query_as(
        r#"
        SELECT
            c.name,
            c.surname,
            c.first_name,
            c.patronymic,
            c.client_type,
            cd.legal_name,
            cd.edrpou,
            cd.rnokpp,
            cd.vat_payer
        FROM cars car
        JOIN clients c ON c.id = COALESCE($1, car.client_id)
        LEFT JOIN client_details cd ON cd.client_id = c.id
        WHERE car.id = $2
        "#,
    )
    .bind(customer_client_id)
    .bind(car_id)
    .fetch_optional(executor)
    .await?;

    let row =
        row.ok_or_else(|| AppError::validation("Не вдалося визначити контрагента для запису ПДВ"))?;

    let trimmed = |value: Option<String>| {
        value
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    };

    let name = match row.client_type {
        ClientType::Individual => {
            let parts: Vec<String> = [
                row.surname.as_deref(),
                row.first_name.as_deref(),
                row.patronymic.as_deref(),
            ]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect();
            if parts.is_empty() {
                row.name.trim().to_string()
            } else {
                parts.join(" ")
            }
        }
        ClientType::Fop | ClientType::Company => {
            trimmed(row.legal_name).unwrap_or_else(|| row.name.trim().to_string())
        }
    };

    Ok(VatCounterparty {
        name,
        code: trimmed(row.edrpou).or_else(|| trimmed(row.rnokpp)),
        vat_payer: row.vat_payer.unwrap_or(false),
    })
}

async fn items_for_order(
    pool: &Pool<Sqlite>,
    order_id: &str,
) -> AppResult<(Vec<LaborItem>, Vec<PartItem>)> {
    let labor: Vec<LaborItem> = sqlx::query_as(
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
            is_free,
            position,
            vat_rate,
            vat_scheme
        FROM labor_items
        WHERE work_order_id = $1
        ORDER BY position, id
        "#,
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?;

    let parts: Vec<PartItem> = sqlx::query_as(
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
            is_free,
            position,
            vat_rate,
            vat_scheme,
            purchase_vat_rate,
            purchase_price_includes_vat,
            purchase_scheme,
            supplier_name,
            supplier_code,
            invoice_number,
            invoice_date,
            vat_credited
        FROM part_items
        WHERE work_order_id = $1
        ORDER BY position, id
        "#,
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?;

    Ok((labor, parts))
}

async fn build_supply_entries(state: &AppState, order: &WorkOrder) -> AppResult<Vec<NewVatEntry>> {
    if !status_is_supplied(order.status) {
        return Ok(Vec::new());
    }

    let settings = fetch_settings_from_pool(&state.db_pool).await?;

    if !settings
        .company_info
        .as_ref()
        .is_some_and(|company| company.vat_payer)
    {
        return Ok(Vec::new());
    }
    let timezone: chrono_tz::Tz = settings
        .timezone
        .parse()
        .unwrap_or(chrono_tz::Tz::Europe__Kiev);

    let (labor, parts) = items_for_order(&state.db_pool, &order.id).await?;

    let labor_items: Vec<LaborItemDto> = labor
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

    let part_items: Vec<PartItemDto> = parts
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

    let counterparty =
        counterparty_for_order(&state.db_pool, order.customer_client_id, order.car_id).await?;
    let tax_point = tax_point_for(order, timezone);
    let document_number = order
        .tax_invoice_number
        .clone()
        .map(|number| number.trim().to_string())
        .filter(|number| !number.is_empty())
        .or_else(|| Some(order.id.clone()));
    let document_date = order
        .tax_invoice_date
        .as_deref()
        .and_then(parse_day)
        .or(Some(tax_point));

    let mut entries = Vec::new();
    for bucket in &totals.vat_buckets {
        let reportable = !bucket.amount.is_zero() || !bucket.scheme.is_standard();
        if !reportable || bucket.base.is_zero() {
            continue;
        }
        entries.push(NewVatEntry {
            direction: VatDirection::Output,
            tax_point,
            source_kind: VatSourceKind::WorkOrder,
            source_id: order.id.clone(),
            document_number: document_number.clone(),
            document_date,
            exemption_code: order.exemption_code.clone(),
            counterparty_name: counterparty.name.clone(),
            counterparty_code: counterparty.code.clone(),
            counterparty_vat_payer: counterparty.vat_payer,
            base: bucket.base,
            rate: bucket.rate,
            amount: bucket.amount,
            scheme: bucket.scheme,
            reverses_id: None,
        });
    }

    Ok(entries)
}

pub(crate) fn build_expense_entry(expense: &Expense) -> Option<NewVatEntry> {
    if !expense.vat_credited || expense.vat_rate.is_zero() {
        return None;
    }

    let (base, amount, _) =
        pricing::calculate_vat(expense.amount, expense.vat_rate, expense.price_includes_vat);
    if amount.is_zero() {
        return None;
    }

    let invoice_day = expense.invoice_date.as_deref().and_then(parse_day);
    let tax_point = invoice_day.or_else(|| parse_day(&expense.date))?;

    Some(NewVatEntry {
        direction: VatDirection::Input,
        tax_point,
        source_kind: VatSourceKind::Expense,
        source_id: expense.id.to_string(),
        document_number: expense.invoice_number.clone(),
        document_date: invoice_day,
        exemption_code: None,
        counterparty_name: expense.supplier_name.clone().unwrap_or_default(),
        counterparty_code: expense.supplier_code.clone(),
        // A claimed credit is backed by the supplier's tax invoice, which only a
        // VAT payer issues.
        counterparty_vat_payer: true,
        base,
        rate: expense.vat_rate,
        amount,
        scheme: expense.vat_scheme,
        reverses_id: None,
    })
}

pub(crate) fn build_part_entry(part: &PartItem) -> Option<NewVatEntry> {
    if !part.vat_credited || part.purchase_vat_rate.is_zero() {
        return None;
    }

    let gross = (part.base_cost * part.quantity).round_dp(2);
    let (base, amount, _) = pricing::calculate_vat(
        gross,
        part.purchase_vat_rate,
        part.purchase_price_includes_vat,
    );
    if amount.is_zero() {
        return None;
    }

    let invoice_day = part.invoice_date.as_deref().and_then(parse_day)?;

    Some(NewVatEntry {
        direction: VatDirection::Input,
        tax_point: invoice_day,
        source_kind: VatSourceKind::PartItem,
        source_id: part.id.to_string(),
        document_number: part.invoice_number.clone(),
        document_date: Some(invoice_day),
        exemption_code: None,
        counterparty_name: part.supplier_name.clone().unwrap_or_default(),
        counterparty_code: part.supplier_code.clone(),
        counterparty_vat_payer: true,
        base,
        rate: part.purchase_vat_rate,
        amount,
        scheme: part.purchase_scheme,
        reverses_id: None,
    })
}

async fn insert_entry<'e, E>(executor: E, entry: NewVatEntry) -> AppResult<VatEntry>
where
    E: Executor<'e, Database = Sqlite>,
{
    let sql = format!(
        r#"
        INSERT INTO vat_entries (
            direction,
            tax_point,
            source_kind,
            source_id,
            document_number,
            document_date,
            exemption_code,
            counterparty_name,
            counterparty_code,
            counterparty_vat_payer,
            base,
            rate,
            amount,
            scheme,
            reverses_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING {VAT_ENTRY_COLUMNS}
        "#
    );

    Ok(sqlx::query_as(&sql)
        .bind(entry.direction)
        .bind(date_text(entry.tax_point))
        .bind(entry.source_kind)
        .bind(entry.source_id)
        .bind(entry.document_number)
        .bind(entry.document_date.map(date_text))
        .bind(entry.exemption_code)
        .bind(entry.counterparty_name)
        .bind(entry.counterparty_code)
        .bind(entry.counterparty_vat_payer)
        .bind(entry.base.to_string())
        .bind(entry.rate.to_string())
        .bind(entry.amount.to_string())
        .bind(entry.scheme)
        .bind(entry.reverses_id)
        .fetch_one(executor)
        .await?)
}

pub(crate) async fn record_supply(
    state: &AppState,
    executor: &mut SqliteConnection,
    order: &WorkOrder,
) -> AppResult<Vec<VatEntry>> {
    let entries = build_supply_entries(state, order).await?;
    let mut written = Vec::with_capacity(entries.len());
    for entry in entries {
        ensure_vat_period_open(&mut *executor, entry.tax_point).await?;
        written.push(insert_entry(&mut *executor, entry).await?);
    }
    Ok(written)
}

pub(crate) async fn record_expense_credit(
    executor: &mut SqliteConnection,
    expense: &Expense,
) -> AppResult<Option<VatEntry>> {
    let Some(entry) = build_expense_entry(expense) else {
        return Ok(None);
    };
    ensure_vat_period_open(&mut *executor, entry.tax_point).await?;
    Ok(Some(insert_entry(&mut *executor, entry).await?))
}

pub(crate) async fn record_part_credit(
    executor: &mut SqliteConnection,
    part: &PartItem,
) -> AppResult<Option<VatEntry>> {
    let Some(entry) = build_part_entry(part) else {
        return Ok(None);
    };
    ensure_vat_period_open(&mut *executor, entry.tax_point).await?;
    Ok(Some(insert_entry(&mut *executor, entry).await?))
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_vat(
    state: State<'_, AppState>,
    work_order_id: String,
) -> AppResult<Vec<VatEntry>> {
    let sql = format!(
        "SELECT {VAT_ENTRY_COLUMNS} FROM vat_entries
         WHERE direction = 'output' AND source_kind = 'work_order' AND source_id = $1
         ORDER BY id"
    );
    Ok(sqlx::query_as(&sql)
        .bind(work_order_id)
        .fetch_all(&state.db_pool)
        .await?)
}

fn credit_day(credit_note_date: Option<String>, timezone: chrono_tz::Tz) -> AppResult<NaiveDate> {
    match credit_note_date {
        Some(text) if !text.trim().is_empty() => {
            parse_day(&text).ok_or_else(|| AppError::validation("Некоректна дата коригування"))
        }
        _ => Ok(Utc::now().with_timezone(&timezone).date_naive()),
    }
}

fn credit_document_number(credit_note_number: Option<String>) -> Option<String> {
    credit_note_number.and_then(|number| {
        let number = number.trim().to_string();
        (!number.is_empty()).then_some(number)
    })
}

async fn local_timezone(state: &AppState) -> AppResult<chrono_tz::Tz> {
    let settings = fetch_settings_from_pool(&state.db_pool).await?;
    Ok(settings
        .timezone
        .parse()
        .unwrap_or(chrono_tz::Tz::Europe__Kiev))
}

/// Reverses every live VAT entry of an order, leaving the order editable again.
#[tauri::command]
#[specta::specta]
pub async fn void_work_order_vat(
    state: State<'_, AppState>,
    work_order_id: String,
    credit_note_number: Option<String>,
    credit_note_date: Option<String>,
) -> AppResult<Vec<VatEntry>> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let originals = live_entries_for_order(&mut *tx, &work_order_id).await?;
    if originals.is_empty() {
        return Err(AppError::vat_entry_not_found());
    }

    let timezone = local_timezone(&state).await?;
    let day = credit_day(credit_note_date, timezone)?;
    ensure_vat_period_open(&mut *tx, day).await?;
    let document_number =
        credit_document_number(credit_note_number).or_else(|| originals[0].document_number.clone());

    let mut reversals = Vec::with_capacity(originals.len());
    for original in &originals {
        let reversal = NewVatEntry {
            direction: VatDirection::Output,
            tax_point: day,
            source_kind: VatSourceKind::WorkOrder,
            source_id: work_order_id.clone(),
            document_number: document_number.clone(),
            document_date: Some(day),
            exemption_code: original.exemption_code.clone(),
            counterparty_name: original.counterparty_name.clone(),
            counterparty_code: original.counterparty_code.clone(),
            counterparty_vat_payer: original.counterparty_vat_payer,
            base: -original.base,
            rate: original.rate,
            amount: -original.amount,
            scheme: original.scheme,
            reverses_id: Some(original.id),
        };
        reversals.push(insert_entry(&mut *tx, reversal).await?);

        sqlx::query("UPDATE vat_entries SET voided_at = $1 WHERE id = $2")
            .bind(timestamp_text())
            .bind(original.id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    log::info!("Сторновано ПДВ за замовленням {work_order_id}");

    Ok(reversals)
}

/// Records the order's VAT again after a reversal, from its current figures.
#[tauri::command]
#[specta::specta]
pub async fn record_work_order_vat(
    state: State<'_, AppState>,
    work_order_id: String,
) -> AppResult<Vec<VatEntry>> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let sql = format!(
        "SELECT {} FROM work_orders WHERE id = $1",
        work_orders::WORK_ORDER_COLUMNS
    );
    let order: Option<WorkOrder> = sqlx::query_as(&sql)
        .bind(&work_order_id)
        .fetch_optional(&mut *tx)
        .await?;
    let order = order.ok_or_else(AppError::work_order_not_found)?;

    if order.archived_at.is_some() {
        return Err(AppError::work_order_archived());
    }

    let existing = live_entries_for_order(&mut *tx, &work_order_id).await?;
    if !existing.is_empty() {
        return Ok(existing);
    }

    if !status_is_supplied(order.status) {
        return Err(AppError::validation(
            "ПДВ фіксується, коли замовлення виконано або оплачено",
        ));
    }

    let entries = record_supply(state.inner(), &mut tx, &order).await?;
    if entries.is_empty() {
        return Err(AppError::validation(
            "У цьому замовленні немає ПДВ для фіксації",
        ));
    }

    tx.commit().await?;
    log::info!("Зафіксовано ПДВ за замовленням {work_order_id}");

    Ok(entries)
}

/// Every VAT row of an expense.
#[tauri::command]
#[specta::specta]
pub async fn fetch_expense_vat(
    state: State<'_, AppState>,
    expense_id: i32,
) -> AppResult<Vec<VatEntry>> {
    let sql = format!(
        "SELECT {VAT_ENTRY_COLUMNS} FROM vat_entries
         WHERE direction = 'input' AND source_kind = 'expense' AND source_id = $1
         ORDER BY id"
    );
    Ok(sqlx::query_as(&sql)
        .bind(expense_id.to_string())
        .fetch_all(&state.db_pool)
        .await?)
}

/// Reverses the live credit of an expense and stops claiming it.
#[tauri::command]
#[specta::specta]
pub async fn void_expense_vat(
    state: State<'_, AppState>,
    expense_id: i32,
    credit_note_number: Option<String>,
    credit_note_date: Option<String>,
) -> AppResult<VatEntry> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let Some(original) = live_entry_for_expense(&mut *tx, i64::from(expense_id)).await? else {
        return Err(AppError::vat_entry_not_found());
    };

    let timezone = local_timezone(&state).await?;
    let day = credit_day(credit_note_date, timezone)?;
    ensure_vat_period_open(&mut *tx, day).await?;
    let document_number =
        credit_document_number(credit_note_number).or_else(|| original.document_number.clone());

    let reversal = NewVatEntry {
        direction: VatDirection::Input,
        tax_point: day,
        source_kind: VatSourceKind::Expense,
        source_id: i64::from(expense_id).to_string(),
        document_number,
        document_date: Some(day),
        exemption_code: original.exemption_code.clone(),
        counterparty_name: original.counterparty_name.clone(),
        counterparty_code: original.counterparty_code.clone(),
        counterparty_vat_payer: original.counterparty_vat_payer,
        base: -original.base,
        rate: original.rate,
        amount: -original.amount,
        scheme: original.scheme,
        reverses_id: Some(original.id),
    };

    let entry = insert_entry(&mut *tx, reversal).await?;

    sqlx::query("UPDATE vat_entries SET voided_at = $1 WHERE id = $2")
        .bind(timestamp_text())
        .bind(original.id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE expenses SET vat_credited = 0 WHERE id = $1")
        .bind(expense_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    log::info!("Сторновано ПДВ за витратою {expense_id}");

    Ok(entry)
}

/// Every VAT row of a part purchase.
#[tauri::command]
#[specta::specta]
pub async fn fetch_part_item_vat(
    state: State<'_, AppState>,
    part_item_id: i32,
) -> AppResult<Vec<VatEntry>> {
    let sql = format!(
        "SELECT {VAT_ENTRY_COLUMNS} FROM vat_entries
         WHERE direction = 'input' AND source_kind = 'part_item' AND source_id = $1
         ORDER BY id"
    );
    Ok(sqlx::query_as(&sql)
        .bind(part_item_id.to_string())
        .fetch_all(&state.db_pool)
        .await?)
}

/// Reverses the live purchase credit of a part and stops claiming it.
#[tauri::command]
#[specta::specta]
pub async fn void_part_item_vat(
    state: State<'_, AppState>,
    part_item_id: i32,
    credit_note_number: Option<String>,
    credit_note_date: Option<String>,
) -> AppResult<VatEntry> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let Some(original) = live_entry_for_part(&mut *tx, i64::from(part_item_id)).await? else {
        return Err(AppError::vat_entry_not_found());
    };

    let timezone = local_timezone(&state).await?;
    let day = credit_day(credit_note_date, timezone)?;
    ensure_vat_period_open(&mut *tx, day).await?;
    let document_number =
        credit_document_number(credit_note_number).or_else(|| original.document_number.clone());

    let reversal = NewVatEntry {
        direction: VatDirection::Input,
        tax_point: day,
        source_kind: VatSourceKind::PartItem,
        source_id: i64::from(part_item_id).to_string(),
        document_number,
        document_date: Some(day),
        exemption_code: original.exemption_code.clone(),
        counterparty_name: original.counterparty_name.clone(),
        counterparty_code: original.counterparty_code.clone(),
        counterparty_vat_payer: original.counterparty_vat_payer,
        base: -original.base,
        rate: original.rate,
        amount: -original.amount,
        scheme: original.scheme,
        reverses_id: Some(original.id),
    };

    let entry = insert_entry(&mut *tx, reversal).await?;

    sqlx::query("UPDATE vat_entries SET voided_at = $1 WHERE id = $2")
        .bind(timestamp_text())
        .bind(original.id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE part_items SET vat_credited = 0 WHERE id = $1")
        .bind(part_item_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    log::info!("Сторновано ПДВ за запчастиною {part_item_id}");

    Ok(entry)
}

/// The closed months, newest last.
#[tauri::command]
#[specta::specta]
pub async fn fetch_vat_periods(state: State<'_, AppState>) -> AppResult<Vec<VatPeriod>> {
    Ok(
        sqlx::query_as("SELECT month, filed_at, note FROM vat_periods ORDER BY month")
            .fetch_all(&state.db_pool)
            .await?,
    )
}

fn validate_month(raw: &str) -> AppResult<String> {
    let month = raw.trim();
    if month.len() != 7 || NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d").is_err() {
        return Err(AppError::validation("Некоректний місяць періоду ПДВ"));
    }
    Ok(month.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn close_vat_period(
    state: State<'_, AppState>,
    month: String,
    note: Option<String>,
) -> AppResult<VatPeriod> {
    let month = validate_month(&month)?;
    let note = note.and_then(|note| {
        let note = note.trim().to_string();
        (!note.is_empty()).then_some(note)
    });

    let mut tx = begin_immediate(&state.db_pool).await?;

    sqlx::query(
        "INSERT INTO vat_periods (month, note) VALUES ($1, $2)
         ON CONFLICT (month) DO UPDATE SET note = excluded.note",
    )
    .bind(&month)
    .bind(&note)
    .execute(&mut *tx)
    .await?;

    let period: VatPeriod =
        sqlx::query_as("SELECT month, filed_at, note FROM vat_periods WHERE month = $1")
            .bind(&month)
            .fetch_one(&mut *tx)
            .await?;

    tx.commit().await?;
    log::info!("Закрито період ПДВ {month}");

    Ok(period)
}

#[tauri::command]
#[specta::specta]
pub async fn reopen_vat_period(state: State<'_, AppState>, month: String) -> AppResult<()> {
    let month = validate_month(&month)?;
    let result = sqlx::query("DELETE FROM vat_periods WHERE month = $1")
        .bind(&month)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Період ПДВ не знайдено".to_string()));
    }
    log::info!("Відкрито період ПДВ {month}");
    Ok(())
}
