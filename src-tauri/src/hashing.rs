//! Deterministic hashing of database records.
//!
//! A hash is built field by field, and every field is framed so that field
//! boundaries cannot blur: `("ab", "c")` and `("a", "bc")` must hash
//! differently. Features that compare a stored hash with the live records -
//! reports, generated documents share the framing implemented here.

use blake3::Hasher;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Feeds one field at a time into a hasher, each framed so field boundaries are unambiguous.
///
/// Every field is written as its byte length followed by its bytes.
/// This avoids collisions on permutations `("ab", "c")` and `("a", "bc")`.
pub(crate) trait FieldHash {
    fn field(&mut self, bytes: &[u8]) -> &mut Self;

    fn text(&mut self, value: &str) -> &mut Self {
        self.field(value.as_bytes())
    }

    /// An optional value keeps a marker so "absent" differs from "empty".
    fn optional_text(&mut self, value: Option<&str>) -> &mut Self {
        match value {
            Some(value) => {
                self.field(b"1");
                self.text(value)
            }
            None => self.field(b"0"),
        }
    }

    fn number(&mut self, value: i64) -> &mut Self {
        self.text(&value.to_string())
    }

    fn optional_number(&mut self, value: Option<i64>) -> &mut Self {
        match value {
            Some(value) => {
                self.field(b"1");
                self.number(value)
            }
            None => self.field(b"0"),
        }
    }

    fn flag(&mut self, value: bool) -> &mut Self {
        self.field(if value { b"1" } else { b"0" })
    }

    /// Decimals hash through their plain string form: `1.50` and `1.5` are the
    /// same amount written two ways, and must not read as a change.
    fn decimal(&mut self, value: Decimal) -> &mut Self {
        self.text(&value.normalize().to_string())
    }

    fn timestamp(&mut self, value: DateTime<Utc>) -> &mut Self {
        self.text(&value.to_rfc3339())
    }

    fn optional_timestamp(&mut self, value: Option<DateTime<Utc>>) -> &mut Self {
        match value {
            Some(value) => {
                self.field(b"1");
                self.timestamp(value)
            }
            None => self.field(b"0"),
        }
    }
}

impl FieldHash for Hasher {
    fn field(&mut self, bytes: &[u8]) -> &mut Self {
        self.update(&(bytes.len() as u64).to_le_bytes());
        self.update(bytes)
    }
}
