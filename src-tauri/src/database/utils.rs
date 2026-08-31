use std::str::FromStr;

use rust_decimal::Decimal;
use sqlx::{sqlite::SqliteRow, Pool, Row, Sqlite, Transaction};

use crate::error::AppResult;

/// Starts a `BEGIN IMMEDIATE` transaction: the write lock is acquired up
/// front, so a subsequent check-then-act (e.g. `ensure_*` + write) cannot
/// interleave with a concurrent writer. Use for every write transaction;
/// plain `begin()` stays fine for read-only snapshots.
pub(crate) async fn begin_immediate(
    pool: &Pool<Sqlite>,
) -> AppResult<Transaction<'static, Sqlite>> {
    Ok(pool.begin_with("BEGIN IMMEDIATE").await?)
}

pub(crate) fn decimal_column(row: &SqliteRow, column: &str) -> sqlx::Result<Decimal> {
    let raw: String = row.try_get(column)?;
    Decimal::from_str(&raw).map_err(|err| sqlx::Error::ColumnDecode {
        index: column.to_string(),
        source: Box::new(err),
    })
}
