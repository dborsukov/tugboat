use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::database::utils::decimal_column;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, specta::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VatDirection {
    /// VAT the shop charged the customer
    Output,
    /// VAT the shop paid on its purchases
    Input,
}

/// What the entry was written from.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, specta::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VatSourceKind {
    WorkOrder,
    Expense,
    PartItem,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    sqlx::Type,
    specta::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VatScheme {
    /// Taxed at the stated rate
    Standard,
    /// Zero-rated
    Zero,
    /// Exempt from VAT (ст. 197 ПКУ)
    Exempt,
    /// Outside VAT (ст. 196 ПКУ)
    NotObject,
    /// Zero-rated export
    Export,
}

impl VatScheme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Оподатковувана",
            Self::Zero => "Нульова",
            Self::Exempt => "Звільнена",
            Self::NotObject => "Не є об'єктом",
            Self::Export => "Експорт",
        }
    }

    pub fn is_standard(self) -> bool {
        matches!(self, Self::Standard)
    }
}

/// One fixed VAT amount.
///
/// The row is the record, a report reads it and never recomputes the tax from
/// the order's current lines. A correction is a second row that reverses this
/// one, so the history of a period stays readable.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VatEntry {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub direction: VatDirection,
    /// `YYYY-MM-DD` in the business time zone
    pub tax_point: String,
    pub source_kind: VatSourceKind,
    pub source_id: String,
    /// Reference the register prints, the work order number until a real tax
    /// invoice number is issued for the supply.
    pub document_number: Option<String>,
    pub document_date: Option<String>,
    /// The benefit code from the tax-invoice register, for an exempt supply.
    pub exemption_code: Option<String>,
    /// The counterparty as it was at the tax point.
    pub counterparty_name: String,
    pub counterparty_code: Option<String>,
    pub counterparty_vat_payer: bool,
    pub base: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    pub scheme: VatScheme,
    /// Set on a reversal row, pointing at the entry it cancels.
    #[specta(type = Option<specta_typescript::Number>)]
    pub reverses_id: Option<i64>,
    /// Set on the original entry once it has been reversed.
    pub voided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl<'r> FromRow<'r, SqliteRow> for VatEntry {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            direction: row.try_get("direction")?,
            tax_point: row.try_get("tax_point")?,
            source_kind: row.try_get("source_kind")?,
            source_id: row.try_get("source_id")?,
            document_number: row.try_get("document_number")?,
            document_date: row.try_get("document_date")?,
            exemption_code: row.try_get("exemption_code")?,
            counterparty_name: row.try_get("counterparty_name")?,
            counterparty_code: row.try_get("counterparty_code")?,
            counterparty_vat_payer: row.try_get("counterparty_vat_payer")?,
            base: decimal_column(row, "base")?,
            rate: decimal_column(row, "rate")?,
            amount: decimal_column(row, "amount")?,
            scheme: row.try_get("scheme")?,
            reverses_id: row.try_get("reverses_id")?,
            voided_at: row.try_get("voided_at")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

/// An entry waiting to be written. Kept apart from [`VatEntry`] because the
/// database, not the caller, assigns the id and timestamp.
pub(crate) struct NewVatEntry {
    pub direction: VatDirection,
    pub tax_point: NaiveDate,
    pub source_kind: VatSourceKind,
    pub source_id: String,
    pub document_number: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub exemption_code: Option<String>,
    pub counterparty_name: String,
    pub counterparty_code: Option<String>,
    pub counterparty_vat_payer: bool,
    pub base: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    pub scheme: VatScheme,
    pub reverses_id: Option<i64>,
}

/// A month whose VAT return has been filed and locked.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VatPeriod {
    /// `YYYY-MM`.
    pub month: String,
    pub filed_at: DateTime<Utc>,
    pub note: Option<String>,
}

impl<'r> FromRow<'r, SqliteRow> for VatPeriod {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            month: row.try_get("month")?,
            filed_at: row.try_get("filed_at")?,
            note: row.try_get("note")?,
        })
    }
}
