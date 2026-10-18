use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, specta::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    /// Income and expenses over a period.
    Financial,
    /// Output VAT fixed in the ledger over a period.
    Vat,
}

impl ReportKind {
    /// The folder segment a report of this kind lives under.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Financial => "financial",
            Self::Vat => "vat",
        }
    }

    /// Ukrainian title, used in the workbook heading and the list.
    pub fn title(self) -> &'static str {
        match self {
            Self::Financial => "Фінансовий звіт",
            Self::Vat => "Реєстр ПДВ",
        }
    }
}

/// A closed day range, both ends included.
///
/// Stored and compared as plain `YYYY-MM-DD` text, the same way expense dates
/// are, a report is about calendar days, so no timezone may shift its edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DayRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        if end < start {
            Self {
                start: end,
                end: start,
            }
        } else {
            Self { start, end }
        }
    }

    pub fn start_text(self) -> String {
        self.start.format("%Y-%m-%d").to_string()
    }

    pub fn end_text(self) -> String {
        self.end.format("%Y-%m-%d").to_string()
    }

    /// Folder segment naming the period, e.g. `2026-09-01_2026-09-30`.
    pub fn folder_name(self) -> String {
        format!("{}_{}", self.start_text(), self.end_text())
    }

    /// First instant of the range, as a stored timestamp bound.
    pub fn start_timestamp(self) -> DateTime<Utc> {
        self.start
            .and_hms_opt(0, 0, 0)
            .expect("midnight is always a valid time")
            .and_utc()
    }

    /// Exclusive upper bound, midnight of the day after `end`, so the whole of
    /// `end` is inside the window.
    pub fn end_timestamp(self) -> DateTime<Utc> {
        (self.end + chrono::TimeDelta::days(1))
            .and_hms_opt(0, 0, 0)
            .expect("midnight is always a valid time")
            .and_utc()
    }

    /// Every day of the range, first to last.
    pub fn days(self) -> impl Iterator<Item = NaiveDate> {
        let mut day = self.start;
        let last = self.end;
        std::iter::from_fn(move || {
            if day > last {
                return None;
            }
            let current = day;
            day += chrono::TimeDelta::days(1);
            Some(current)
        })
    }
}

/// A generated report as the list shows it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub report_type: ReportKind,
    pub period_start: String,
    pub period_end: String,
    pub file_name: String,
    pub generated_at: DateTime<Utc>,
    /// Size of the file on disk, or `None` when it is no longer there.
    #[specta(type = Option<specta_typescript::Number>)]
    pub file_size: Option<i64>,
    /// True when the sources behind this period have changed since it was made,
    /// so the numbers in it no longer describe what is in the database.
    pub is_stale: bool,
}

impl<'r> FromRow<'r, SqliteRow> for Report {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        let file_path: String = row.try_get("file_path")?;
        Ok(Self {
            id: row.try_get("id")?,
            report_type: row.try_get("report_type")?,
            period_start: row.try_get("period_start")?,
            period_end: row.try_get("period_end")?,
            file_name: std::path::Path::new(&file_path)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            generated_at: row.try_get("generated_at")?,
            // Filled in by the command: neither of these is a column.
            file_size: None,
            is_stale: false,
        })
    }
}

/// Generation results
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReportOutcome {
    /// True when a new version was written, false when the newest version
    /// already described exactly these records.
    pub created: bool,
    /// The version that holds the period now, the one just written, or the one
    /// that was already there.
    pub report: Report,
}

/// A row of the `reports` table plus everything the list does not need.
pub(crate) struct ReportRow {
    pub report: Report,
    pub file_path: String,
    pub source_hash: String,
}

impl<'r> FromRow<'r, SqliteRow> for ReportRow {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            report: Report::from_row(row)?,
            file_path: row.try_get("file_path")?,
            source_hash: row.try_get("source_hash")?,
        })
    }
}
