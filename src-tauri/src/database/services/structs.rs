use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::database::{utils::decimal_column, work_orders::structs::LaborPriceType};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub price_type: LaborPriceType,
    pub fixed_price: Decimal,
    pub labor_hours: Decimal,
    pub archived_at: Option<DateTime<Utc>>,
}

impl<'r> FromRow<'r, SqliteRow> for Service {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            code: row.try_get("code")?,
            name: row.try_get("name")?,
            price_type: row.try_get("price_type")?,
            fixed_price: decimal_column(row, "fixed_price")?,
            labor_hours: decimal_column(row, "labor_hours")?,
            archived_at: row.try_get("archived_at")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateServicePayload {
    pub code: Option<String>,
    pub name: String,
    pub price_type: LaborPriceType,
    pub fixed_price: Decimal,
    pub labor_hours: Decimal,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServicePayload {
    pub code: Option<String>,
    pub name: String,
    pub price_type: LaborPriceType,
    pub fixed_price: Decimal,
    pub labor_hours: Decimal,
}
