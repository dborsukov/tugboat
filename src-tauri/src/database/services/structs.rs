use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::database::utils::decimal_column;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    pub labor_hours: Decimal,
}

impl<'r> FromRow<'r, SqliteRow> for Service {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            code: row.try_get("code")?,
            name: row.try_get("name")?,
            labor_hours: decimal_column(row, "labor_hours")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateServicePayload {
    pub code: Option<String>,
    pub name: String,
    pub labor_hours: Decimal,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServicePayload {
    pub code: Option<String>,
    pub name: String,
    pub labor_hours: Decimal,
}
