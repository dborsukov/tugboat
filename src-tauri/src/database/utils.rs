use std::str::FromStr;

use rust_decimal::Decimal;
use sqlx::{sqlite::SqliteRow, Pool, Row, Sqlite, Transaction};

use crate::error::{AppError, AppResult};

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

/// Trims `raw`, rejects an empty value and caps its length.
pub(crate) fn required_text(raw: &str, max: usize, label: &str) -> AppResult<String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err(AppError::validation(format!(
            "Потрібно заповнити поле «{label}»"
        )));
    }
    ensure_length(text, max, label)?;
    Ok(text.to_string())
}

/// Trims `raw`, maps whitespace-only to `None` and caps the length.
pub(crate) fn optional_text(
    raw: Option<String>,
    max: usize,
    label: &str,
) -> AppResult<Option<String>> {
    let Some(text) = raw else {
        return Ok(None);
    };
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    ensure_length(text, max, label)?;
    Ok(Some(text.to_string()))
}

fn ensure_length(text: &str, max: usize, label: &str) -> AppResult<()> {
    if text.chars().count() > max {
        return Err(AppError::validation(format!(
            "Поле «{label}» не може бути довшим за {max} символів"
        )));
    }
    Ok(())
}

const PHONE_MIN_DIGITS: usize = 7;
const PHONE_MAX_DIGITS: usize = 15;

/// Canonicalises a phone number to E.164: a leading `+` and digits only.
///
/// The client form already sends this shape, but the unique index on
/// `clients.phone` compares raw text, so the command has to guarantee it rather
/// than trust the form. Numbers without a country code are rejected instead of guessed.
pub(crate) fn phone(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(value) = raw else {
        return Ok(None);
    };

    let compact: String = value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '+')
        .collect();

    if compact.is_empty() {
        return Ok(None);
    }

    let digits = compact.strip_prefix('+').unwrap_or(&compact);
    let valid = compact.starts_with('+')
        && (PHONE_MIN_DIGITS..=PHONE_MAX_DIGITS).contains(&digits.chars().count());

    if !valid {
        return Err(AppError::validation(
            "Телефон має бути у міжнародному форматі, наприклад +380671234567",
        ));
    }

    Ok(Some(compact))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_text_trims_and_rejects_blank() {
        assert_eq!(
            required_text("  Заміна оливи ", 50, "Назва").expect("valid"),
            "Заміна оливи"
        );
        assert!(required_text("   ", 50, "Назва").is_err());
    }

    #[test]
    fn text_length_is_capped_in_characters() {
        let long = "я".repeat(11);
        assert!(required_text(&long, 10, "Назва").is_err());
        // The cap counts characters, not bytes: these are 10 two-byte letters.
        assert!(required_text(&"я".repeat(10), 10, "Назва").is_ok());
    }

    #[test]
    fn optional_text_blanks_whitespace() {
        assert_eq!(
            optional_text(Some("   ".into()), 10, "Нотатки").unwrap(),
            None
        );
        assert_eq!(optional_text(None, 10, "Нотатки").unwrap(), None);
        assert_eq!(
            optional_text(Some("  текст  ".into()), 10, "Нотатки").unwrap(),
            Some("текст".to_string())
        );
    }

    #[test]
    fn phone_is_canonicalised_to_e164() {
        assert_eq!(
            phone(Some("+380 (67) 555-12-34".into())).unwrap(),
            Some("+380675551234".to_string())
        );
        // Already canonical numbers pass through unchanged.
        assert_eq!(
            phone(Some("+380675551234".into())).unwrap(),
            Some("+380675551234".to_string())
        );
        assert_eq!(phone(Some("   ".into())).unwrap(), None);
        assert_eq!(phone(None).unwrap(), None);
    }

    #[test]
    fn phone_without_country_code_is_rejected() {
        assert!(phone(Some("0675551234".into())).is_err());
        assert!(phone(Some("+123".into())).is_err());
        assert!(phone(Some("+3806755512345678".into())).is_err());
    }
}
