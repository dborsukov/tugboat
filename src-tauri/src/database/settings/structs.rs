use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, types::Json, FromRow, Row};

use crate::database::utils::decimal_column;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: ThemeMode,
    #[specta(type = specta_typescript::Number)]
    pub ui_scale: f64,
    pub default_markup: Decimal,
    pub rate_variants: Vec<RateVariants>,
    pub vin_lookup_templates: Vec<VinLookupTemplates>,
    pub license_plate_lookup_templates: Vec<LicensePlateLookupTemplates>,
}

impl<'r> FromRow<'r, SqliteRow> for AppSettings {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            theme: row.try_get("theme")?,
            ui_scale: row.try_get("ui_scale")?,
            default_markup: decimal_column(row, "default_markup")?,
            rate_variants: row
                .try_get::<Json<Vec<RateVariants>>, _>("rate_variants")?
                .0,
            vin_lookup_templates: row
                .try_get::<Json<Vec<VinLookupTemplates>>, _>("vin_lookup_templates")?
                .0,
            license_plate_lookup_templates: row
                .try_get::<Json<Vec<LicensePlateLookupTemplates>>, _>(
                    "license_plate_lookup_templates",
                )?
                .0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RateVariants {
    pub name: String,
    pub value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VinLookupTemplates {
    pub label: String,
    pub url_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LicensePlateLookupTemplates {
    pub label: String,
    pub url_template: String,
}
