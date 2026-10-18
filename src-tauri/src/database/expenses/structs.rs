use chrono::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row, Type};

use crate::database::{
    utils::decimal_column, vat::structs::VatScheme, work_orders::structs::PaymentMethod,
};

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type, specta::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExpenseCategoryColor {
    #[default]
    Slate,
    Red,
    Orange,
    Amber,
    Yellow,
    Lime,
    Green,
    Emerald,
    Teal,
    Cyan,
    Sky,
    Blue,
    Indigo,
    Violet,
    Purple,
    Fuchsia,
    Pink,
    Rose,
    Brown,
    Gray,
}

impl ExpenseCategoryColor {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Slate => "slate",
            Self::Red => "red",
            Self::Orange => "orange",
            Self::Amber => "amber",
            Self::Yellow => "yellow",
            Self::Lime => "lime",
            Self::Green => "green",
            Self::Emerald => "emerald",
            Self::Teal => "teal",
            Self::Cyan => "cyan",
            Self::Sky => "sky",
            Self::Blue => "blue",
            Self::Indigo => "indigo",
            Self::Violet => "violet",
            Self::Purple => "purple",
            Self::Fuchsia => "fuchsia",
            Self::Pink => "pink",
            Self::Rose => "rose",
            Self::Brown => "brown",
            Self::Gray => "gray",
        }
    }
}

fn color_column(row: &SqliteRow, column: &str) -> sqlx::Result<ExpenseCategoryColor> {
    let raw: String = row.try_get(column)?;
    Ok(raw.parse().unwrap_or_else(|_| {
        log::warn!("Категорія витрат: невідомий колір «{raw}», використано типовий");
        ExpenseCategoryColor::default()
    }))
}

impl std::str::FromStr for ExpenseCategoryColor {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "slate" => Self::Slate,
            "red" => Self::Red,
            "orange" => Self::Orange,
            "amber" => Self::Amber,
            "yellow" => Self::Yellow,
            "lime" => Self::Lime,
            "green" => Self::Green,
            "emerald" => Self::Emerald,
            "teal" => Self::Teal,
            "cyan" => Self::Cyan,
            "sky" => Self::Sky,
            "blue" => Self::Blue,
            "indigo" => Self::Indigo,
            "violet" => Self::Violet,
            "purple" => Self::Purple,
            "fuchsia" => Self::Fuchsia,
            "pink" => Self::Pink,
            "rose" => Self::Rose,
            "brown" => Self::Brown,
            "gray" => Self::Gray,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseCategory {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub color: ExpenseCategoryColor,
    pub archived_at: Option<DateTime<chrono::Utc>>,
}

impl<'r> FromRow<'r, SqliteRow> for ExpenseCategory {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            color: color_column(row, "color")?,
            archived_at: row.try_get("archived_at")?,
        })
    }
}

/// The category an expense points at, flattened into the expense row itself.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseCategoryRef {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub color: ExpenseCategoryColor,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub date: String,
    pub amount: Decimal,
    pub description: String,
    /// `None` when the expense has no category, and also when its category row is gone
    pub category: Option<ExpenseCategoryRef>,
    pub payment_method: Option<PaymentMethod>,
    pub vat_rate: Decimal,
    pub price_includes_vat: bool,
    pub vat_scheme: VatScheme,
    pub supplier_name: Option<String>,
    pub supplier_code: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub vat_credited: bool,
    pub created_at: DateTime<chrono::Utc>,
    pub archived_at: Option<DateTime<chrono::Utc>>,
}

impl<'r> FromRow<'r, SqliteRow> for Expense {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        let category_id: Option<i64> = row.try_get("category_id")?;
        let category = match category_id {
            Some(id) => Some(ExpenseCategoryRef {
                id,
                name: row.try_get("category_name")?,
                color: color_column(row, "category_color")?,
            }),
            None => None,
        };

        Ok(Self {
            id: row.try_get("id")?,
            date: row.try_get("date")?,
            amount: decimal_column(row, "amount")?,
            description: row.try_get("description")?,
            category,
            payment_method: row.try_get("payment_method")?,
            vat_rate: decimal_column(row, "vat_rate")?,
            price_includes_vat: row.try_get("price_includes_vat")?,
            vat_scheme: row.try_get("vat_scheme")?,
            supplier_name: row.try_get("supplier_name")?,
            supplier_code: row.try_get("supplier_code")?,
            invoice_number: row.try_get("invoice_number")?,
            invoice_date: row.try_get("invoice_date")?,
            vat_credited: row.try_get("vat_credited")?,
            created_at: row.try_get("created_at")?,
            archived_at: row.try_get("archived_at")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateExpensePayload {
    pub date: String,
    pub amount: Decimal,
    pub description: String,
    pub category_id: Option<i32>,
    pub payment_method: Option<PaymentMethod>,
    pub vat_rate: Decimal,
    pub price_includes_vat: bool,
    pub vat_scheme: VatScheme,
    pub supplier_name: Option<String>,
    pub supplier_code: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub vat_credited: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExpensePayload {
    pub date: String,
    pub amount: Decimal,
    pub description: String,
    pub category_id: Option<i32>,
    pub payment_method: Option<PaymentMethod>,
    pub vat_rate: Decimal,
    pub price_includes_vat: bool,
    pub vat_scheme: VatScheme,
    pub supplier_name: Option<String>,
    pub supplier_code: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub vat_credited: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateExpenseCategoryPayload {
    pub name: String,
    pub color: ExpenseCategoryColor,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExpenseCategoryPayload {
    pub name: String,
    pub color: ExpenseCategoryColor,
}
