pub mod structs;

use rust_decimal::Decimal;
use sqlx::{Executor, Sqlite};
use structs::*;
use tauri::State;

use crate::{
    database::{utils::begin_immediate, vat, work_orders::structs::PaymentMethod},
    error::{AppError, AppResult},
    logging::Event,
};

/// Shared with the reports module, which reads expenses for a period
pub(crate) const EXPENSE_SELECT: &str = r#"
    SELECT
        e.id,
        e.date,
        e.amount,
        e.description,
        e.payment_method,
        e.vat_rate,
        e.price_includes_vat,
        e.vat_scheme,
        e.supplier_name,
        e.supplier_code,
        e.invoice_number,
        e.invoice_date,
        e.vat_credited,
        e.created_at,
        e.archived_at,
        c.id AS category_id,
        c.name AS category_name,
        c.color AS category_color
    FROM expenses e
    LEFT JOIN expense_categories c ON c.id = e.category_id
"#;

const EXPENSE_CATEGORY_COLUMNS: &str = "id, name, color, archived_at";

/// Newest day first; expenses written on the same day keep their creation order.
const EXPENSE_ORDER: &str = "ORDER BY e.date DESC, e.id DESC";

const MAX_DESCRIPTION_LENGTH: usize = 300;
const MAX_CATEGORY_NAME_LENGTH: usize = 60;
const MAX_SUPPLIER_NAME_LENGTH: usize = 200;
const MAX_SUPPLIER_CODE_LENGTH: usize = 20;
const MAX_INVOICE_NUMBER_LENGTH: usize = 60;

/// Date is a calendar day with no time zone attached
fn validate_date(raw: &str) -> AppResult<String> {
    let date = raw.trim();
    if date.len() != 10 {
        return Err(AppError::validation("Вкажіть дату витрати"));
    }

    let bytes = date.as_bytes();
    let shaped = bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());
    if !shaped {
        return Err(AppError::validation("Некоректна дата витрати"));
    }

    if chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
        return Err(AppError::validation("Некоректна дата витрати"));
    }

    Ok(date.to_string())
}

fn validate_amount(amount: Decimal) -> AppResult<Decimal> {
    if amount.is_sign_negative() {
        return Err(AppError::validation("Сума витрати не може бути від'ємною"));
    }
    Ok(amount)
}

fn validate_description(raw: &str) -> AppResult<String> {
    let description = raw.trim();
    if description.is_empty() {
        return Err(AppError::validation("Вкажіть опис витрати"));
    }
    if description.chars().count() > MAX_DESCRIPTION_LENGTH {
        return Err(AppError::validation(format!(
            "Опис не може бути довшим за {MAX_DESCRIPTION_LENGTH} символів"
        )));
    }
    Ok(description.to_string())
}

fn validate_vat_rate(vat_rate: Decimal) -> AppResult<Decimal> {
    if vat_rate.is_sign_negative() || vat_rate > Decimal::from(100) {
        return Err(AppError::validation("Ставка ПДВ має бути від 0 до 100"));
    }
    Ok(vat_rate)
}

fn validate_optional_text(
    raw: Option<String>,
    max: usize,
    message: &str,
) -> AppResult<Option<String>> {
    let Some(text) = raw else {
        return Ok(None);
    };
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if text.chars().count() > max {
        return Err(AppError::validation(message));
    }
    Ok(Some(text.to_string()))
}

fn validate_invoice_date(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    if raw.trim().is_empty() {
        return Ok(None);
    }
    validate_date(&raw).map(Some)
}

/// A credit has to be backed by a supplier's tax invoice, or it cannot be
/// claimed and would not survive a review.
fn validate_credit(
    vat_credited: bool,
    amount: Decimal,
    vat_rate: Decimal,
    supplier_name: &Option<String>,
    invoice_number: &Option<String>,
    invoice_date: &Option<String>,
) -> AppResult<()> {
    if !vat_credited {
        return Ok(());
    }
    if amount <= Decimal::ZERO {
        return Err(AppError::validation(
            "Для податкового кредиту вкажіть суму, більшу за нуль",
        ));
    }
    if vat_rate <= Decimal::ZERO {
        return Err(AppError::validation(
            "Для податкового кредиту вкажіть ставку ПДВ",
        ));
    }
    if supplier_name.is_none() {
        return Err(AppError::validation(
            "Вкажіть постачальника для податкового кредиту",
        ));
    }
    if invoice_number.is_none() {
        return Err(AppError::validation("Вкажіть номер податкової накладної"));
    }
    if invoice_date.is_none() {
        return Err(AppError::validation("Вкажіть дату податкової накладної"));
    }
    Ok(())
}

/// An id that survives a foreign key check, so a bad one is reported as a
/// missing category instead of surfacing as a raw constraint failure. `None`
/// means "no category", which is why it passes straight through.
async fn validate_category_id<'e, E>(
    executor: E,
    category_id: Option<i32>,
) -> AppResult<Option<i32>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let Some(id) = category_id else {
        return Ok(None);
    };

    let archived_at: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM expense_categories WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await?;

    match archived_at {
        None => Err(AppError::expense_category_not_found()),
        Some(Some(_)) => Err(AppError::validation(
            "Категорія в архіві — оберіть іншу або відновіть її в налаштуваннях",
        )),
        Some(None) => Ok(Some(id)),
    }
}

fn validate_category_name(raw: &str) -> AppResult<String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(AppError::validation(
            "Назва категорії не може бути порожньою",
        ));
    }
    if name.chars().count() > MAX_CATEGORY_NAME_LENGTH {
        return Err(AppError::validation(format!(
            "Назва категорії не може бути довшою за {MAX_CATEGORY_NAME_LENGTH} символів"
        )));
    }
    Ok(name.to_string())
}

fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_expenses_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM expenses WHERE archived_at IS NULL")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_expenses_list(state: State<'_, crate::AppState>) -> AppResult<Vec<Expense>> {
    let expenses: Vec<Expense> = sqlx::query_as(&format!("{EXPENSE_SELECT} {EXPENSE_ORDER}"))
        .fetch_all(&state.db_pool)
        .await?;
    Ok(expenses)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_expense_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<Expense> {
    fetch_expense(&state.db_pool, id).await
}

#[tauri::command]
#[specta::specta]
pub async fn create_expense(
    state: State<'_, crate::AppState>,
    payload: CreateExpensePayload,
) -> AppResult<Expense> {
    let fields = ExpenseFields::from(payload).validate()?;

    let mut tx = begin_immediate(&state.db_pool).await?;

    if fields.vat_credited
        && crate::database::work_orders::is_company_not_vat_payer(&mut tx).await?
    {
        return Err(AppError::validation(
            "Бізнес не є платником ПДВ — податковий кредит за витратою недоступний.",
        ));
    }

    let category_id = validate_category_id(&mut *tx, fields.category_id).await?;

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO expenses (
            date,
            amount,
            description,
            category_id,
            payment_method,
            vat_rate,
            price_includes_vat,
            vat_scheme,
            supplier_name,
            supplier_code,
            invoice_number,
            invoice_date,
            vat_credited
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id
        "#,
    )
    .bind(fields.date)
    .bind(fields.amount.to_string())
    .bind(fields.description)
    .bind(category_id)
    .bind(fields.payment_method)
    .bind(fields.vat_rate.to_string())
    .bind(fields.price_includes_vat)
    .bind(fields.vat_scheme)
    .bind(fields.supplier_name)
    .bind(fields.supplier_code)
    .bind(fields.invoice_number)
    .bind(fields.invoice_date)
    .bind(fields.vat_credited)
    .fetch_one(&mut *tx)
    .await?;

    let expense = fetch_expense(&mut *tx, id as i32).await?;

    Event::new("expense.create")
        .id(expense.id)
        .on_error(vat::record_expense_credit(&mut tx, &expense).await)?;

    tx.commit().await?;

    Event::new("expense.create")
        .id(expense.id)
        .field("amount", expense.amount)
        .ok();

    Ok(expense)
}

#[tauri::command]
#[specta::specta]
pub async fn update_expense(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateExpensePayload,
) -> AppResult<Expense> {
    let fields = ExpenseFields::from(payload).validate()?;

    let mut tx = begin_immediate(&state.db_pool).await?;

    let current = fetch_active_expense(&mut *tx, id).await?;

    let live_vat = vat::live_entry_for_expense(&mut *tx, i64::from(id)).await?;
    if live_vat.is_some() && expense_vat_fields_changed(&current, &fields) {
        return Err(AppError::expense_vat_locked());
    }
    if live_vat.is_none()
        && fields.vat_credited
        && crate::database::work_orders::is_company_not_vat_payer(&mut tx).await?
    {
        return Err(AppError::validation(
            "Бізнес не є платником ПДВ — податковий кредит за витратою недоступний.",
        ));
    }

    let category_id = validate_category_id(&mut *tx, fields.category_id).await?;

    sqlx::query(
        r#"
        UPDATE expenses SET
            date = $2,
            amount = $3,
            description = $4,
            category_id = $5,
            payment_method = $6,
            vat_rate = $7,
            price_includes_vat = $8,
            vat_scheme = $9,
            supplier_name = $10,
            supplier_code = $11,
            invoice_number = $12,
            invoice_date = $13,
            vat_credited = $14
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(fields.date)
    .bind(fields.amount.to_string())
    .bind(fields.description)
    .bind(category_id)
    .bind(fields.payment_method)
    .bind(fields.vat_rate.to_string())
    .bind(fields.price_includes_vat)
    .bind(fields.vat_scheme)
    .bind(fields.supplier_name)
    .bind(fields.supplier_code)
    .bind(fields.invoice_number)
    .bind(fields.invoice_date)
    .bind(fields.vat_credited)
    .execute(&mut *tx)
    .await?;

    let expense = fetch_expense(&mut *tx, id).await?;

    // A corrected expense that now claims VAT
    if live_vat.is_none() {
        Event::new("expense.update")
            .id(expense.id)
            .on_error(vat::record_expense_credit(&mut tx, &expense).await)?;
    }

    tx.commit().await?;

    Event::new("expense.update")
        .id(expense.id)
        .field("amount", expense.amount)
        .ok();

    Ok(expense)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_expense(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    vat::ensure_expense_vat_unlocked(&mut *tx, i64::from(id)).await?;

    let result = sqlx::query("DELETE FROM expenses WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::expense_not_found());
    }

    tx.commit().await?;

    Event::new("expense.delete").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_expenses(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        vat::ensure_expense_vat_unlocked(&mut *tx, i64::from(*id)).await?;
        let result = sqlx::query("DELETE FROM expenses WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::expense_not_found());
        }
    }
    tx.commit().await?;

    Event::new("expense.delete_many")
        .field("count", ids.len())
        .ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn archive_expense(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE expenses SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = $1
          AND archived_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&state.db_pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::expense_not_found());
    }

    Event::new("expense.archive").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn archive_expenses(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query(
            r#"
            UPDATE expenses SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
            WHERE id = $1
              AND archived_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::expense_not_found());
        }
    }
    tx.commit().await?;

    Event::new("expense.archive_many")
        .field("count", ids.len())
        .ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_expense(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("UPDATE expenses SET archived_at = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::expense_not_found());
    }

    Event::new("expense.restore").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_expenses(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query("UPDATE expenses SET archived_at = NULL WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::expense_not_found());
        }
    }
    tx.commit().await?;

    Event::new("expense.restore_many")
        .field("count", ids.len())
        .ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_expense_categories(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<ExpenseCategory>> {
    let categories: Vec<ExpenseCategory> = sqlx::query_as(&format!(
        "SELECT {EXPENSE_CATEGORY_COLUMNS} FROM expense_categories \
         ORDER BY name COLLATE NOCASE"
    ))
    .fetch_all(&state.db_pool)
    .await?;
    Ok(categories)
}

#[tauri::command]
#[specta::specta]
pub async fn create_expense_category(
    state: State<'_, crate::AppState>,
    payload: CreateExpenseCategoryPayload,
) -> AppResult<ExpenseCategory> {
    let name = validate_category_name(&payload.name)?;
    let key = name_key(&name);

    let mut tx = begin_immediate(&state.db_pool).await?;

    if let Some(existing) = fetch_category_by_name_key(&mut *tx, &key).await? {
        return Err(duplicate_category_error(&existing));
    }

    let category: ExpenseCategory = sqlx::query_as(&format!(
        r#"
        INSERT INTO expense_categories (name, name_key, color)
        VALUES ($1, $2, $3)
        RETURNING {EXPENSE_CATEGORY_COLUMNS}
        "#
    ))
    .bind(&name)
    .bind(&key)
    .bind(payload.color.as_str())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    log::info!("Створено категорію витрат «{name}»");

    Event::new("expense_category.create").id(category.id).ok();

    Ok(category)
}

#[tauri::command]
#[specta::specta]
pub async fn update_expense_category(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateExpenseCategoryPayload,
) -> AppResult<ExpenseCategory> {
    let name = validate_category_name(&payload.name)?;
    let key = name_key(&name);

    let mut tx = begin_immediate(&state.db_pool).await?;

    if let Some(existing) = fetch_category_by_name_key(&mut *tx, &key).await? {
        if existing.id != i64::from(id) {
            return Err(duplicate_category_error(&existing));
        }
    }

    let category: ExpenseCategory = sqlx::query_as(&format!(
        r#"
        UPDATE expense_categories SET name = $2, name_key = $3, color = $4
        WHERE id = $1
        RETURNING {EXPENSE_CATEGORY_COLUMNS}
        "#
    ))
    .bind(id)
    .bind(&name)
    .bind(&key)
    .bind(payload.color.as_str())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("expense_category.update").id(category.id).ok();

    Ok(category)
}

#[tauri::command]
#[specta::specta]
pub async fn archive_expense_category(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE expense_categories SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = $1
          AND archived_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&state.db_pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::expense_category_not_found());
    }

    Event::new("expense_category.archive").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_expense_category(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let category = fetch_category_by_id(&mut *tx, id).await?;

    let key = name_key(&category.name);
    if let Some(existing) = fetch_category_by_name_key(&mut *tx, &key).await? {
        if existing.id != category.id {
            return Err(AppError::validation(format!(
                "Категорія «{}» вже активна — перейменуйте її перед відновленням",
                existing.name
            )));
        }
    }

    sqlx::query("UPDATE expense_categories SET archived_at = NULL WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Event::new("expense_category.restore").id(id).ok();

    Ok(())
}
/// The payload after validation, nothing here can be blank
struct ExpenseFields {
    date: String,
    amount: Decimal,
    description: String,
    category_id: Option<i32>,
    payment_method: Option<PaymentMethod>,
    vat_rate: Decimal,
    price_includes_vat: bool,
    vat_scheme: crate::database::vat::structs::VatScheme,
    supplier_name: Option<String>,
    supplier_code: Option<String>,
    invoice_number: Option<String>,
    invoice_date: Option<String>,
    vat_credited: bool,
}

impl From<CreateExpensePayload> for ExpenseFields {
    fn from(payload: CreateExpensePayload) -> Self {
        Self {
            date: payload.date,
            amount: payload.amount,
            description: payload.description,
            category_id: payload.category_id,
            payment_method: payload.payment_method,
            vat_rate: payload.vat_rate,
            price_includes_vat: payload.price_includes_vat,
            vat_scheme: payload.vat_scheme,
            supplier_name: payload.supplier_name,
            supplier_code: payload.supplier_code,
            invoice_number: payload.invoice_number,
            invoice_date: payload.invoice_date,
            vat_credited: payload.vat_credited,
        }
    }
}

impl From<UpdateExpensePayload> for ExpenseFields {
    fn from(payload: UpdateExpensePayload) -> Self {
        Self {
            date: payload.date,
            amount: payload.amount,
            description: payload.description,
            category_id: payload.category_id,
            payment_method: payload.payment_method,
            vat_rate: payload.vat_rate,
            price_includes_vat: payload.price_includes_vat,
            vat_scheme: payload.vat_scheme,
            supplier_name: payload.supplier_name,
            supplier_code: payload.supplier_code,
            invoice_number: payload.invoice_number,
            invoice_date: payload.invoice_date,
            vat_credited: payload.vat_credited,
        }
    }
}

impl ExpenseFields {
    fn validate(self) -> AppResult<Self> {
        let vat_rate = validate_vat_rate(self.vat_rate)?;
        crate::database::vat::validate_rate_and_scheme(vat_rate, self.vat_scheme)?;
        let supplier_name = validate_optional_text(
            self.supplier_name,
            MAX_SUPPLIER_NAME_LENGTH,
            "Назва постачальника не може бути довшою за 200 символів",
        )?;
        let supplier_code = validate_optional_text(
            self.supplier_code,
            MAX_SUPPLIER_CODE_LENGTH,
            "Код постачальника не може бути довшим за 20 символів",
        )?;
        let invoice_number = validate_optional_text(
            self.invoice_number,
            MAX_INVOICE_NUMBER_LENGTH,
            "Номер податкової накладної не може бути довшим за 60 символів",
        )?;
        let invoice_date = validate_invoice_date(self.invoice_date)?;

        validate_credit(
            self.vat_credited,
            self.amount,
            vat_rate,
            &supplier_name,
            &invoice_number,
            &invoice_date,
        )?;

        Ok(Self {
            date: validate_date(&self.date)?,
            amount: validate_amount(self.amount)?,
            description: validate_description(&self.description)?,
            category_id: self.category_id,
            payment_method: self.payment_method,
            vat_rate,
            price_includes_vat: self.price_includes_vat,
            vat_scheme: self.vat_scheme,
            supplier_name,
            supplier_code,
            invoice_number,
            invoice_date,
            vat_credited: self.vat_credited,
        })
    }
}

fn duplicate_category_error(existing: &ExpenseCategory) -> AppError {
    if existing.archived_at.is_some() {
        AppError::validation(format!(
            "Категорія «{}» вже є, але в архіві — відновіть її в налаштуваннях",
            existing.name
        ))
    } else {
        AppError::validation(format!("Категорія «{}» вже існує", existing.name))
    }
}

/// One expense with its category resolved, or "not found" when there is no such
/// row. Every read goes through here so a single expense and a list of them can
/// never disagree about what a category is called.
async fn fetch_expense<'e, E>(executor: E, id: i32) -> AppResult<Expense>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as(&format!("{EXPENSE_SELECT} WHERE e.id = $1"))
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or_else(AppError::expense_not_found)
}

async fn fetch_active_expense<'e, E>(executor: E, id: i32) -> AppResult<Expense>
where
    E: Executor<'e, Database = Sqlite>,
{
    let expense = fetch_expense(executor, id).await?;
    if expense.archived_at.is_some() {
        return Err(AppError::expense_archived());
    }
    Ok(expense)
}

fn expense_vat_fields_changed(current: &Expense, fields: &ExpenseFields) -> bool {
    current.amount != fields.amount
        || current.vat_rate != fields.vat_rate
        || current.price_includes_vat != fields.price_includes_vat
        || current.vat_scheme != fields.vat_scheme
        || current.supplier_name != fields.supplier_name
        || current.supplier_code != fields.supplier_code
        || current.invoice_number != fields.invoice_number
        || current.invoice_date != fields.invoice_date
        || current.vat_credited != fields.vat_credited
}

async fn fetch_category_by_id<'e, E>(executor: E, id: i32) -> AppResult<ExpenseCategory>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as(&format!(
        "SELECT {EXPENSE_CATEGORY_COLUMNS} FROM expense_categories WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await?
    .ok_or_else(AppError::expense_category_not_found)
}

async fn fetch_category_by_name_key<'e, E>(
    executor: E,
    key: &str,
) -> AppResult<Option<ExpenseCategory>>
where
    E: Executor<'e, Database = Sqlite>,
{
    Ok(sqlx::query_as(&format!(
        "SELECT {EXPENSE_CATEGORY_COLUMNS} FROM expense_categories WHERE name_key = $1"
    ))
    .bind(key)
    .fetch_optional(executor)
    .await?)
}
