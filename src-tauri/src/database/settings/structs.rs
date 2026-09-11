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
    pub default_vat_rate: Decimal,
    pub default_prices_include_vat: bool,
    pub rate_variants: Vec<RateVariants>,
    pub vin_lookup_templates: Vec<VinLookupTemplates>,
    pub license_plate_lookup_templates: Vec<LicensePlateLookupTemplates>,
    pub timezone: String,
    pub default_phone_country_code: String,
    pub company_info: Option<CompanyInfo>,
    pub documents_dir: Option<String>,
}

pub(crate) fn decode_company_info(raw: Option<String>) -> Option<CompanyInfo> {
    match raw.as_deref() {
        None | Some("") => None,
        Some(text) => serde_json::from_str(text).ok(),
    }
}

impl<'r> FromRow<'r, SqliteRow> for AppSettings {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            theme: row.try_get("theme")?,
            ui_scale: row.try_get("ui_scale")?,
            default_markup: decimal_column(row, "default_markup")?,
            default_vat_rate: decimal_column(row, "default_vat_rate")?,
            default_prices_include_vat: row.try_get("default_prices_include_vat")?,
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
            timezone: row.try_get("timezone")?,
            default_phone_country_code: row.try_get("default_phone_country_code")?,
            company_info: decode_company_info(row.try_get("company_info")?),
            documents_dir: row.try_get("documents_dir")?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum CompanyKind {
    Fop,
    Company,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfo {
    pub kind: CompanyKind,
    /// Назва бізнесу (коротка/торгова) — спільне поле для обох типів.
    pub name: Option<String>,
    pub legal_name: Option<String>,
    pub legal_address: Option<String>,
    pub physical_address: Option<String>,
    /// Contact phone number of the business.
    pub phone: Option<String>,
    pub edrpou: Option<String>,
    pub rnokpp: Option<String>,
    /// ПІБ of the ФОП.
    pub surname: Option<String>,
    pub first_name: Option<String>,
    pub patronymic: Option<String>,
    /// ПІБ of the company director.
    pub director_surname: Option<String>,
    pub director_first_name: Option<String>,
    pub director_patronymic: Option<String>,
    pub iban: Option<String>,
    pub bank_name: Option<String>,
    pub vat_payer: bool,
    /// True when every field required for the chosen `kind`
    /// is filled out properly. Computed by the backend on save.
    #[serde(default)]
    pub is_complete: bool,
}
