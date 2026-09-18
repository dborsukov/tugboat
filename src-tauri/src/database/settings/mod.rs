pub mod structs;

use std::collections::HashSet;

use rust_decimal::Decimal;
use sqlx::types::Json;
use structs::*;
use tauri::State;

use crate::{
    database::utils::decimal_column,
    error::{AppError, AppResult},
};

const UI_SCALE_MIN: f64 = 0.5;
const UI_SCALE_MAX: f64 = 2.0;
const MARKUP_MIN: i32 = 0;
const MARKUP_MAX: i32 = 300;
const VAT_RATE_MIN: u8 = 0;
const VAT_RATE_MAX: u8 = 100;
const RATE_VARIANT_NAME_MAX_LEN: usize = 60;
const LOOKUP_TEMPLATE_LABEL_MAX_LEN: usize = 60;
const LOOKUP_TEMPLATE_URL_MAX_LEN: usize = 500;
const LOOKUP_PLACEHOLDER: &str = "%s";
const TIMEZONE_MAX_LEN: usize = 64;

pub(crate) async fn fetch_settings_from_pool(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> AppResult<AppSettings> {
    let settings: AppSettings = sqlx::query_as(
        r#"
        SELECT
            id,
            theme,
            ui_scale,
            default_markup,
            default_vat_rate,
            default_prices_include_vat,
            rate_variants,
            vin_lookup_templates,
            license_plate_lookup_templates,
            timezone,
            default_phone_country_code,
            company_info,
            documents_dir
        FROM app_settings
        WHERE id = 1
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(settings)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_settings(state: State<'_, crate::AppState>) -> AppResult<AppSettings> {
    fetch_settings_from_pool(&state.db_pool).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_theme(
    theme: ThemeMode,
    state: State<'_, crate::AppState>,
) -> AppResult<ThemeMode> {
    let theme: ThemeMode =
        sqlx::query_scalar("UPDATE app_settings SET theme = $1 WHERE id = 1 RETURNING theme")
            .bind(theme)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(theme)
}

#[tauri::command]
#[specta::specta]
pub async fn update_ui_scale(ui_scale: f64, state: State<'_, crate::AppState>) -> AppResult<f64> {
    if !(UI_SCALE_MIN..=UI_SCALE_MAX).contains(&ui_scale) {
        return Err(AppError::validation(format!(
            "ui_scale must be between {UI_SCALE_MIN} and {UI_SCALE_MAX}"
        )));
    }

    let ui_scale: f64 =
        sqlx::query_scalar("UPDATE app_settings SET ui_scale = $1 WHERE id = 1 RETURNING ui_scale")
            .bind(ui_scale)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(ui_scale)
}

#[tauri::command]
#[specta::specta]
pub async fn update_default_markup(
    default_markup: Decimal,
    state: State<'_, crate::AppState>,
) -> AppResult<Decimal> {
    if default_markup < Decimal::from(MARKUP_MIN) || default_markup > Decimal::from(MARKUP_MAX) {
        return Err(AppError::validation(format!(
            "default_markup must be between {MARKUP_MIN} and {MARKUP_MAX}"
        )));
    }
    let row = sqlx::query(
        "UPDATE app_settings SET default_markup = $1 WHERE id = 1 RETURNING default_markup",
    )
    .bind(default_markup.to_string())
    .fetch_one(&state.db_pool)
    .await?;
    let default_markup = decimal_column(&row, "default_markup")?;
    Ok(default_markup)
}

#[tauri::command]
#[specta::specta]
pub async fn update_default_vat_rate(
    default_vat_rate: Decimal,
    state: State<'_, crate::AppState>,
) -> AppResult<Decimal> {
    if default_vat_rate < Decimal::from(VAT_RATE_MIN)
        || default_vat_rate > Decimal::from(VAT_RATE_MAX)
    {
        return Err(AppError::validation(format!(
            "default_vat_rate must be between {VAT_RATE_MIN} and {VAT_RATE_MAX}"
        )));
    }
    if default_vat_rate.scale() > 2 {
        return Err(AppError::validation(
            "default_vat_rate must have at most 2 decimal places",
        ));
    }
    let row = sqlx::query(
        "UPDATE app_settings SET default_vat_rate = $1 WHERE id = 1 RETURNING default_vat_rate",
    )
    .bind(default_vat_rate.to_string())
    .fetch_one(&state.db_pool)
    .await?;
    let default_vat_rate = decimal_column(&row, "default_vat_rate")?;
    Ok(default_vat_rate)
}

#[tauri::command]
#[specta::specta]
pub async fn update_default_prices_include_vat(
    default_prices_include_vat: bool,
    state: State<'_, crate::AppState>,
) -> AppResult<bool> {
    let saved: bool = sqlx::query_scalar(
        "UPDATE app_settings SET default_prices_include_vat = $1 WHERE id = 1 RETURNING default_prices_include_vat",
    )
    .bind(default_prices_include_vat)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(saved)
}

#[tauri::command]
#[specta::specta]
pub async fn update_rate_variants(
    rate_variants: Vec<RateVariants>,
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<RateVariants>> {
    validate_rate_variants(&rate_variants)?;
    let saved: Json<Vec<RateVariants>> = sqlx::query_scalar(
        "UPDATE app_settings SET rate_variants = $1 WHERE id = 1 RETURNING rate_variants",
    )
    .bind(Json(rate_variants))
    .fetch_one(&state.db_pool)
    .await?;
    Ok(saved.0)
}

fn validate_rate_variants(variants: &[RateVariants]) -> Result<(), AppError> {
    let mut seen = HashSet::new();
    for v in variants {
        let name = v.name.trim();
        if name.is_empty() {
            return Err(AppError::validation("rate variant name is required"));
        }
        if name.chars().count() > RATE_VARIANT_NAME_MAX_LEN {
            return Err(AppError::validation("rate variant name is too long"));
        }
        if !seen.insert(name.to_lowercase()) {
            return Err(AppError::validation(format!(
                "duplicate rate variant name: {name}"
            )));
        }
        if v.value.is_sign_negative() {
            return Err(AppError::validation(
                "rate variant value must not be negative",
            ));
        }
    }
    if variants.iter().filter(|v| v.is_default).count() > 1 {
        return Err(AppError::validation(
            "only one rate variant can be the default",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct LookupTemplateEntry<'a> {
    label: &'a str,
    url_template: &'a str,
}

fn validate_lookup_templates(
    entries: &[LookupTemplateEntry<'_>],
    kind: &str,
) -> Result<(), AppError> {
    let mut seen = HashSet::new();
    for entry in entries {
        let label = entry.label.trim();
        if label.is_empty() {
            return Err(AppError::validation(format!("{kind} label is required")));
        }
        if label.chars().count() > LOOKUP_TEMPLATE_LABEL_MAX_LEN {
            return Err(AppError::validation(format!("{kind} label is too long")));
        }
        if !seen.insert(label.to_lowercase()) {
            return Err(AppError::validation(format!(
                "duplicate {kind} label: {label}"
            )));
        }

        let url = entry.url_template.trim();
        if url.is_empty() {
            return Err(AppError::validation(format!("{kind} url is required")));
        }
        if url.chars().count() > LOOKUP_TEMPLATE_URL_MAX_LEN {
            return Err(AppError::validation(format!("{kind} url is too long")));
        }
        if !url.contains(LOOKUP_PLACEHOLDER) {
            return Err(AppError::validation(format!(
                "{kind} url must contain the {LOOKUP_PLACEHOLDER} placeholder"
            )));
        }
        let lower = url.to_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(AppError::validation(format!(
                "{kind} url must start with http:// or https://"
            )));
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_vin_lookup_templates(
    vin_lookup_templates: Vec<VinLookupTemplates>,
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<VinLookupTemplates>> {
    let entries: Vec<LookupTemplateEntry> = vin_lookup_templates
        .iter()
        .map(|t| LookupTemplateEntry {
            label: &t.label,
            url_template: &t.url_template,
        })
        .collect();
    validate_lookup_templates(&entries, "vin lookup template")?;

    // Persist trimmed values so labels and urls are stored in canonical form.
    let templates: Vec<VinLookupTemplates> = vin_lookup_templates
        .into_iter()
        .map(|t| VinLookupTemplates {
            label: t.label.trim().to_string(),
            url_template: t.url_template.trim().to_string(),
        })
        .collect();

    let saved: Json<Vec<VinLookupTemplates>> = sqlx::query_scalar(
        "UPDATE app_settings SET vin_lookup_templates = $1 WHERE id = 1 RETURNING vin_lookup_templates",
    )
    .bind(Json(&templates))
    .fetch_one(&state.db_pool)
    .await?;
    Ok(saved.0)
}

#[tauri::command]
#[specta::specta]
pub async fn update_license_plate_lookup_templates(
    license_plate_lookup_templates: Vec<LicensePlateLookupTemplates>,
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<LicensePlateLookupTemplates>> {
    let entries: Vec<LookupTemplateEntry> = license_plate_lookup_templates
        .iter()
        .map(|t| LookupTemplateEntry {
            label: &t.label,
            url_template: &t.url_template,
        })
        .collect();
    validate_lookup_templates(&entries, "license plate lookup template")?;

    // Persist trimmed values so labels and urls are stored in canonical form.
    let templates: Vec<LicensePlateLookupTemplates> = license_plate_lookup_templates
        .into_iter()
        .map(|t| LicensePlateLookupTemplates {
            label: t.label.trim().to_string(),
            url_template: t.url_template.trim().to_string(),
        })
        .collect();

    let saved: Json<Vec<LicensePlateLookupTemplates>> = sqlx::query_scalar(
        "UPDATE app_settings SET license_plate_lookup_templates = $1 WHERE id = 1 RETURNING license_plate_lookup_templates",
    )
    .bind(Json(&templates))
    .fetch_one(&state.db_pool)
    .await?;
    Ok(saved.0)
}

#[tauri::command]
#[specta::specta]
pub async fn update_timezone(
    timezone: String,
    state: State<'_, crate::AppState>,
) -> AppResult<String> {
    let timezone = timezone.trim();
    if timezone.is_empty() {
        return Err(AppError::validation("timezone is required"));
    }
    if timezone.chars().count() > TIMEZONE_MAX_LEN {
        return Err(AppError::validation("timezone is too long"));
    }
    if timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(AppError::validation(format!(
            "unknown timezone: {timezone}"
        )));
    }

    let saved: String =
        sqlx::query_scalar("UPDATE app_settings SET timezone = $1 WHERE id = 1 RETURNING timezone")
            .bind(timezone)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(saved)
}

#[tauri::command]
#[specta::specta]
pub async fn update_default_phone_country_code(
    default_phone_country_code: String,
    state: State<'_, crate::AppState>,
) -> AppResult<String> {
    let code = default_phone_country_code.trim().to_ascii_uppercase();
    if code.len() != 2 || !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::validation(
            "default_phone_country_code must be a two-letter ISO country code",
        ));
    }

    let saved: String = sqlx::query_scalar(
        "UPDATE app_settings SET default_phone_country_code = $1 WHERE id = 1 RETURNING default_phone_country_code",
    )
    .bind(code)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(saved)
}

const COMPANY_INFO_TEXT_MAX_LEN: usize = 500;

fn validate_company_info(info: &CompanyInfo) -> Result<(), AppError> {
    for v in [
        &info.name,
        &info.legal_name,
        &info.legal_address,
        &info.physical_address,
        &info.phone,
        &info.edrpou,
        &info.rnokpp,
        &info.surname,
        &info.first_name,
        &info.patronymic,
        &info.director_surname,
        &info.director_first_name,
        &info.director_patronymic,
        &info.iban,
        &info.bank_name,
    ]
    .into_iter()
    .flatten()
    {
        if v.trim().chars().count() > COMPANY_INFO_TEXT_MAX_LEN {
            return Err(AppError::validation(format!(
                "company info field is too long (max {COMPANY_INFO_TEXT_MAX_LEN} characters)"
            )));
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_company_info(
    mut company_info: CompanyInfo,
    state: State<'_, crate::AppState>,
) -> AppResult<CompanyInfo> {
    validate_company_info(&company_info)?;

    company_info.is_complete = company_info_is_complete(&company_info);

    let saved: Json<CompanyInfo> = sqlx::query_scalar(
        "UPDATE app_settings SET company_info = $1 WHERE id = 1 RETURNING company_info",
    )
    .bind(Json(&company_info))
    .fetch_one(&state.db_pool)
    .await?;
    Ok(saved.0)
}

fn is_n_digits(value: &str, n: usize) -> bool {
    value.len() == n && value.bytes().all(|b| b.is_ascii_digit())
}

fn is_valid_iban(value: &str) -> bool {
    value.len() >= 15
        && value.len() <= 34
        && value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn filled(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|v| !v.trim().is_empty())
}

fn company_info_is_complete(info: &CompanyInfo) -> bool {
    fn bank_details_ok(info: &CompanyInfo) -> bool {
        filled(&info.bank_name)
            && info
                .iban
                .as_deref()
                .is_some_and(|iban| is_valid_iban(iban.trim()))
    }

    match info.kind {
        CompanyKind::Fop => {
            filled(&info.surname)
                && filled(&info.first_name)
                && filled(&info.patronymic)
                && filled(&info.rnokpp)
                && is_n_digits(info.rnokpp.as_deref().unwrap_or_default(), 10)
                && filled(&info.legal_address)
                && filled(&info.physical_address)
                && bank_details_ok(info)
        }
        CompanyKind::Company => {
            filled(&info.name)
                && filled(&info.legal_name)
                && filled(&info.legal_address)
                && filled(&info.physical_address)
                && filled(&info.edrpou)
                && is_n_digits(info.edrpou.as_deref().unwrap_or_default(), 8)
                && filled(&info.director_surname)
                && filled(&info.director_first_name)
                && filled(&info.director_patronymic)
                && bank_details_ok(info)
        }
    }
}
