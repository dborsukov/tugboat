use std::str::FromStr;

use rust_decimal::Decimal;
use sqlx::{sqlite::SqliteRow, Row};

pub(crate) fn decimal_column(row: &SqliteRow, column: &str) -> sqlx::Result<Decimal> {
    let raw: String = row.try_get(column)?;
    Decimal::from_str(&raw).map_err(|err| sqlx::Error::ColumnDecode {
        index: column.to_string(),
        source: Box::new(err),
    })
}
