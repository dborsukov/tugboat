//! Financial reports.
//!
//! A report is a written file plus a row in `reports` recording which period it
//! covers and the hash of the records it was built from. The log is
//! append-only, regenerating a period writes a new file and leaves the
//! previous one in place, so what a period looked like at any point in time can
//! always be read back.

pub mod figures;
pub mod hash;
pub mod structs;
pub mod vat_figures;
pub mod vat_xlsx;
pub mod xlsx;

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, NaiveDate, Utc};
use hash::{OrderParty, ReportSources};
use sqlx::{QueryBuilder, Sqlite};
use structs::{DayRange, Report, ReportKind};
use tauri::State;

use crate::{
    database::{
        clients::structs::ClientType,
        expenses::structs::Expense,
        vat::structs::VatEntry,
        work_orders::{
            self,
            structs::{LaborItem, PartItem, WorkOrder, WorkOrderStatus},
        },
    },
    error::{AppError, AppResult},
    AppState,
};

/// Reports live in `[app_dir]/reports`.
pub(crate) fn reports_dir(state: &AppState) -> AppResult<PathBuf> {
    let dir = state.data_dir.join("reports");
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку звітів {}: {e}",
            dir.display()
        ))
    })?;
    Ok(dir)
}

/// Folder, one report kind and period owns: `reports/<kind>/<start>_<end>`.
fn report_folder(state: &AppState, kind: ReportKind, range: DayRange) -> AppResult<PathBuf> {
    let dir = reports_dir(state)?
        .join(kind.as_str())
        .join(range.folder_name());
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося створити теку звіту {}: {e}",
            dir.display()
        ))
    })?;
    Ok(dir)
}

/// Unique filename that inlcudes timestamp
fn timestamp_file_name(at: DateTime<Utc>) -> String {
    format!("{}.xlsx", at.format("%Y%m%dT%H%M%SZ"))
}

fn parse_day(text: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::validation("Некоректна дата періоду звіту"))
}

fn day_range(period_start: &str, period_end: &str) -> AppResult<DayRange> {
    Ok(DayRange::new(
        parse_day(period_start)?,
        parse_day(period_end)?,
    ))
}

/// The records a report of `range` is built from, and that its hash covers.
///
/// Orders are drawn from the ones paid inside the range, the same rule the
/// dashboard counts money by, and expenses from the days they are booked on.
pub(crate) async fn report_sources(
    state: &State<'_, AppState>,
    range: DayRange,
) -> AppResult<ReportSources> {
    let orders = paid_orders_in(state, range).await?;
    let order_ids: Vec<&str> = orders.iter().map(|order| order.id.as_str()).collect();

    let labor_items = labor_items_for_orders(state, &order_ids).await?;
    let part_items = part_items_for_orders(state, &order_ids).await?;
    let expenses = expenses_in(state, range).await?;
    let parties = order_parties_for_orders(state, &order_ids).await?;

    Ok(ReportSources {
        orders,
        labor_items,
        part_items,
        expenses,
        parties,
    })
}

/// The VAT ledger rows of a period, both sides, in the order the register prints them.
pub(crate) async fn vat_sources(
    state: &State<'_, AppState>,
    range: DayRange,
) -> AppResult<Vec<VatEntry>> {
    let sql = format!(
        "SELECT {} FROM vat_entries
         WHERE tax_point >= $1 AND tax_point <= $2
         ORDER BY tax_point, id",
        crate::database::vat::VAT_ENTRY_COLUMNS
    );
    Ok(sqlx::query_as(&sql)
        .bind(range.start_text())
        .bind(range.end_text())
        .fetch_all(&state.db_pool)
        .await?)
}

/// The records one report of a kind is built from.
enum ReportData {
    Financial(ReportSources),
    Vat(Vec<VatEntry>),
}

impl ReportData {
    fn hash(&self) -> String {
        match self {
            Self::Financial(sources) => sources.hash(),
            Self::Vat(entries) => hash::vat_entries_hash(entries),
        }
    }

    async fn render(
        &self,
        state: &State<'_, AppState>,
        kind: ReportKind,
        range: DayRange,
    ) -> AppResult<Vec<u8>> {
        match self {
            Self::Financial(sources) => xlsx::render(state, kind, range, sources).await,
            Self::Vat(entries) => vat_xlsx::render(state, range, entries).await,
        }
    }
}

async fn report_data(
    state: &State<'_, AppState>,
    kind: ReportKind,
    range: DayRange,
) -> AppResult<ReportData> {
    match kind {
        ReportKind::Financial => Ok(ReportData::Financial(report_sources(state, range).await?)),
        ReportKind::Vat => Ok(ReportData::Vat(vat_sources(state, range).await?)),
    }
}

async fn paid_orders_in(
    state: &State<'_, crate::AppState>,
    range: DayRange,
) -> AppResult<Vec<WorkOrder>> {
    let sql = format!(
        "SELECT {} FROM work_orders
         WHERE archived_at IS NULL AND status = ? AND paid_at >= ? AND paid_at < ?
         ORDER BY paid_at, id",
        work_orders::WORK_ORDER_COLUMNS
    );
    let orders: Vec<WorkOrder> = sqlx::query_as(&sql)
        .bind(WorkOrderStatus::Paid)
        .bind(
            range
                .start_timestamp()
                .format("%Y-%m-%d %H:%M:%S%.f")
                .to_string(),
        )
        .bind(
            range
                .end_timestamp()
                .format("%Y-%m-%d %H:%M:%S%.f")
                .to_string(),
        )
        .fetch_all(&state.db_pool)
        .await?;
    Ok(orders)
}

async fn expenses_in(
    state: &State<'_, crate::AppState>,
    range: DayRange,
) -> AppResult<Vec<Expense>> {
    let expenses: Vec<Expense> = sqlx::query_as(&format!(
        "{} WHERE e.date >= ? AND e.date <= ? ORDER BY e.date, e.id",
        crate::database::expenses::EXPENSE_SELECT
    ))
    .bind(range.start_text())
    .bind(range.end_text())
    .fetch_all(&state.db_pool)
    .await?;
    Ok(expenses)
}

/// Line items of the given orders, flattened.
///
/// Only the orders the report already counts: an item whose order is outside
/// the period is none of this report's business, and leaving it out keeps the
/// hash and the printed figures describing the same set of records.
async fn labor_items_for_orders(
    state: &State<'_, crate::AppState>,
    order_ids: &[&str],
) -> AppResult<Vec<LaborItem>> {
    fetch_items(
        &state.db_pool,
        order_ids,
        r#"SELECT
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
        FROM labor_items"#,
    )
    .await
}

async fn part_items_for_orders(
    state: &State<'_, crate::AppState>,
    order_ids: &[&str],
) -> AppResult<Vec<PartItem>> {
    fetch_items(
        &state.db_pool,
        order_ids,
        r#"SELECT
            id,
            work_order_id,
            status,
            code,
            name,
            quantity,
            unit_type,
            unit_key,
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
            provided_by,
            supplier_name,
            supplier_code,
            invoice_number,
            invoice_date,
            vat_credited,
            supplier,
            supplier_order_id
        FROM part_items"#,
    )
    .await
}

/// Both item tables are read with one query each rather than one per order.
async fn fetch_items<'r, T>(
    pool: &sqlx::Pool<Sqlite>,
    order_ids: &[&str],
    select: &str,
) -> AppResult<Vec<T>>
where
    T: for<'a> sqlx::FromRow<'a, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    if order_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::new(format!("{select} WHERE work_order_id IN ("));
    let mut separated = builder.separated(", ");
    for id in order_ids {
        separated.push_bind(*id);
    }
    builder.push(") ORDER BY work_order_id, position, id");

    Ok(builder.build_query_as::<T>().fetch_all(pool).await?)
}

#[derive(sqlx::FromRow)]
struct OrderPartyRow {
    order_id: String,
    customer_id: Option<i64>,
    customer_name: Option<String>,
    customer_surname: Option<String>,
    customer_first_name: Option<String>,
    customer_patronymic: Option<String>,
    customer_client_type: Option<ClientType>,
    owner_id: i64,
    owner_name: String,
    owner_surname: Option<String>,
    owner_first_name: Option<String>,
    owner_patronymic: Option<String>,
    owner_client_type: ClientType,
    car_id: i64,
    car_brand: Option<String>,
    car_model: String,
}

impl OrderPartyRow {
    fn into_party(self) -> OrderParty {
        let Self {
            order_id,
            customer_id,
            customer_name,
            customer_surname,
            customer_first_name,
            customer_patronymic,
            customer_client_type,
            owner_id,
            owner_name,
            owner_surname,
            owner_first_name,
            owner_patronymic,
            owner_client_type,
            car_id,
            car_brand,
            car_model,
        } = self;

        let (
            client_id,
            client_name,
            client_surname,
            client_first_name,
            client_patronymic,
            client_type,
        ) = match (customer_id, customer_name, customer_client_type) {
            (Some(id), Some(name), Some(client_type)) => (
                id,
                name,
                customer_surname,
                customer_first_name,
                customer_patronymic,
                client_type,
            ),
            _ => (
                owner_id,
                owner_name,
                owner_surname,
                owner_first_name,
                owner_patronymic,
                owner_client_type,
            ),
        };

        OrderParty {
            order_id,
            client_id,
            client_name,
            client_surname,
            client_first_name,
            client_patronymic,
            client_type,
            car_id,
            car_brand,
            car_model,
        }
    }
}

async fn order_parties_for_orders(
    state: &State<'_, AppState>,
    order_ids: &[&str],
) -> AppResult<Vec<OrderParty>> {
    if order_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::new(
        r#"SELECT
            wo.id AS order_id,
            customer.id AS customer_id,
            customer.name AS customer_name,
            customer.surname AS customer_surname,
            customer.first_name AS customer_first_name,
            customer.patronymic AS customer_patronymic,
            customer.client_type AS customer_client_type,
            owner.id AS owner_id,
            owner.name AS owner_name,
            owner.surname AS owner_surname,
            owner.first_name AS owner_first_name,
            owner.patronymic AS owner_patronymic,
            owner.client_type AS owner_client_type,
            car.id AS car_id,
            brand.name AS car_brand,
            car.model AS car_model
        FROM work_orders wo
        JOIN cars car ON car.id = wo.car_id
        JOIN clients owner ON owner.id = car.client_id
        LEFT JOIN clients customer ON customer.id = wo.customer_client_id
        LEFT JOIN car_brands brand ON brand.id = car.brand_id
        WHERE wo.id IN ("#,
    );

    let mut separated = builder.separated(", ");
    for id in order_ids {
        separated.push_bind(*id);
    }
    builder.push(") ORDER BY wo.paid_at, wo.id");

    let rows: Vec<OrderPartyRow> = builder.build_query_as().fetch_all(&state.db_pool).await?;
    Ok(rows.into_iter().map(OrderPartyRow::into_party).collect())
}

/// Source hashes for several kind-and-period pairs, each computed once.
pub(crate) async fn report_hashes_for(
    state: &State<'_, AppState>,
    keys: &[(ReportKind, DayRange)],
) -> AppResult<HashMap<(ReportKind, NaiveDate, NaiveDate), String>> {
    let mut unique: HashSet<(ReportKind, NaiveDate, NaiveDate)> = HashSet::new();
    let mut hashes = HashMap::new();

    for (kind, range) in keys {
        let key = (*kind, range.start, range.end);
        if !unique.insert(key) {
            continue;
        }
        let data = report_data(state, *kind, *range).await?;
        hashes.insert(key, data.hash());
    }

    Ok(hashes)
}

/// The newest version written for a type and period, if there is one.
async fn latest_report(
    state: &State<'_, AppState>,
    report_type: ReportKind,
    range: DayRange,
) -> AppResult<Option<structs::ReportRow>> {
    let row: Option<structs::ReportRow> = sqlx::query_as(
        r#"
        SELECT
            id,
            report_type,
            period_start,
            period_end,
            file_path,
            source_hash,
            generated_at
        FROM reports
        WHERE report_type = $1 AND period_start = $2 AND period_end = $3
        ORDER BY generated_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(report_type)
    .bind(range.start_text())
    .bind(range.end_text())
    .fetch_optional(&state.db_pool)
    .await?;

    Ok(row)
}

/// Creates a report of `kind` for `range` and records it.
/// The file is new every time, so a previous version of the same period is never touched.
#[tauri::command]
#[specta::specta]
pub async fn generate_report(
    state: State<'_, AppState>,
    report_type: ReportKind,
    period_start: String,
    period_end: String,
) -> AppResult<structs::GenerateReportOutcome> {
    let range = day_range(&period_start, &period_end)?;

    let data = report_data(&state, report_type, range).await?;
    let source_hash = data.hash();

    if let Some(previous) = latest_report(&state, report_type, range).await? {
        let path = PathBuf::from(&previous.file_path);
        // A file that has been deleted is not current regardless of hash
        if previous.source_hash == source_hash && path.is_file() {
            log::info!(
                "Звіт {} за {} не змінився, нову версію не створено",
                report_type.as_str(),
                range.folder_name()
            );
            return Ok(structs::GenerateReportOutcome {
                created: false,
                report: with_file_details(previous.report, &path, false),
            });
        }
    }

    let bytes = data.render(&state, report_type, range).await?;

    let folder = report_folder(&state, report_type, range)?;
    let generated_at = Utc::now();
    let path = folder.join(timestamp_file_name(generated_at));

    fs::write(&path, bytes).map_err(|e| {
        AppError::Generic(format!("Не вдалося зберегти звіт {}: {e}", path.display()))
    })?;

    let report: Report = sqlx::query_as(
        r#"
        INSERT INTO reports (report_type, period_start, period_end, file_path, source_hash, generated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            report_type,
            period_start,
            period_end,
            file_path,
            source_hash,
            generated_at
        "#,
    )
    .bind(report_type)
    .bind(range.start_text())
    .bind(range.end_text())
    .bind(path.to_string_lossy().into_owned())
    .bind(&source_hash)
    .bind(generated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    .fetch_one(&state.db_pool)
    .await?;

    log::info!(
        "Звіт {} за {} збережено до {}",
        report_type.as_str(),
        range.folder_name(),
        path.display()
    );

    Ok(structs::GenerateReportOutcome {
        created: true,
        report: with_file_details(report, &path, false),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_reports(state: State<'_, AppState>) -> AppResult<Vec<Report>> {
    let rows: Vec<structs::ReportRow> = sqlx::query_as(
        r#"
        SELECT
            id,
            report_type,
            period_start,
            period_end,
            file_path,
            source_hash,
            generated_at
        FROM reports
        ORDER BY generated_at DESC, id DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    // One hash per distinct kind and period, not one per row: a financial and a
    // VAT report of the same month are built from different records, so they
    // must not be compared against the same hash.
    let keys: Vec<(ReportKind, DayRange)> = rows
        .iter()
        .map(|row| {
            (
                row.report.report_type,
                DayRange::new(
                    parse_day(&row.report.period_start).unwrap_or_default(),
                    parse_day(&row.report.period_end).unwrap_or_default(),
                ),
            )
        })
        .collect();
    let hashes = report_hashes_for(&state, &keys).await?;

    let reports = rows
        .into_iter()
        .map(|row| {
            let range = DayRange::new(
                parse_day(&row.report.period_start).unwrap_or_default(),
                parse_day(&row.report.period_end).unwrap_or_default(),
            );
            let current = hashes.get(&(row.report.report_type, range.start, range.end));
            // A period whose sources cannot be read is reported as stale rather
            let is_stale = current != Some(&row.source_hash);
            let path = PathBuf::from(&row.file_path);

            with_file_details(row.report, &path, is_stale)
        })
        .collect();

    Ok(reports)
}

fn with_file_details(report: Report, path: &Path, is_stale: bool) -> Report {
    let file_size = fs::metadata(path).ok().map(|meta| meta.len() as i64);
    Report {
        file_size,
        is_stale,
        ..report
    }
}

/// Copies a report out of the reports folder to user selected location,
/// keeping the original generated file in its original folder.
#[tauri::command]
#[specta::specta]
pub async fn download_report(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: i32,
    file_name: String,
) -> AppResult<Option<String>> {
    let row: Option<structs::ReportRow> = sqlx::query_as(
        "SELECT id, report_type, period_start, period_end, file_path, source_hash, generated_at
         FROM reports WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?;
    let row = row.ok_or_else(|| AppError::NotFound("Звіт не знайдено".to_string()))?;

    let source = PathBuf::from(&row.file_path);
    let root = reports_dir(&state)?;
    if !source.starts_with(&root) || !source.is_file() {
        return Err(AppError::NotFound(
            "Файл звіту не знайдено — створіть його ще раз".to_string(),
        ));
    }

    let suggested = crate::utils::suggested_file_name(&file_name, &row.report.file_name);

    let chosen = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        app.dialog()
            .file()
            .set_title("Зберегти звіт")
            .set_file_name(&suggested)
            .add_filter("XLSX", &["xlsx"])
            .blocking_save_file()
            .and_then(|file| file.into_path().ok())
    })
    .await
    .map_err(|e| AppError::Generic(format!("Не вдалося відкрити вікно збереження: {e}")))?;

    let Some(destination) = chosen else {
        return Ok(None);
    };

    fs::copy(&source, &destination).map_err(|e| {
        AppError::Generic(format!(
            "Не вдалося зберегти файл {}: {e}",
            destination.display()
        ))
    })?;

    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
#[specta::specta]
pub async fn open_report_copy(app: tauri::AppHandle, path: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;

    let file = PathBuf::from(&path);
    let is_workbook = file
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"));

    if !is_workbook || !file.is_file() {
        return Err(AppError::NotFound(
            "Файл не знайдено — можливо, його переміщено або видалено".to_string(),
        ));
    }

    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| AppError::Generic(format!("Не вдалося відкрити файл: {e}")))
}
