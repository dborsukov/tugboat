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
const RATE_VARIANT_NAME_MAX_LEN: usize = 60;
const LOOKUP_TEMPLATE_LABEL_MAX_LEN: usize = 60;
const LOOKUP_TEMPLATE_URL_MAX_LEN: usize = 500;
const LOOKUP_PLACEHOLDER: &str = "%s";
const TIMEZONE_MAX_LEN: usize = 64;

#[tauri::command]
#[specta::specta]
pub async fn fetch_settings(state: State<'_, crate::AppState>) -> AppResult<AppSettings> {
    let settings: AppSettings = sqlx::query_as("SELECT * FROM app_settings WHERE id = 1")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(settings)
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
        return Err(AppError::ConstraintViolation(format!(
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
        return Err(AppError::ConstraintViolation(format!(
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
            return Err(AppError::ConstraintViolation(
                "rate variant name is required".into(),
            ));
        }
        if name.chars().count() > RATE_VARIANT_NAME_MAX_LEN {
            return Err(AppError::ConstraintViolation(
                "rate variant name is too long".into(),
            ));
        }
        if !seen.insert(name.to_lowercase()) {
            return Err(AppError::ConstraintViolation(format!(
                "duplicate rate variant name: {name}"
            )));
        }
        if v.value.is_sign_negative() {
            return Err(AppError::ConstraintViolation(
                "rate variant value must not be negative".into(),
            ));
        }
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
            return Err(AppError::ConstraintViolation(format!(
                "{kind} label is required"
            )));
        }
        if label.chars().count() > LOOKUP_TEMPLATE_LABEL_MAX_LEN {
            return Err(AppError::ConstraintViolation(format!(
                "{kind} label is too long"
            )));
        }
        if !seen.insert(label.to_lowercase()) {
            return Err(AppError::ConstraintViolation(format!(
                "duplicate {kind} label: {label}"
            )));
        }

        let url = entry.url_template.trim();
        if url.is_empty() {
            return Err(AppError::ConstraintViolation(format!(
                "{kind} url is required"
            )));
        }
        if url.chars().count() > LOOKUP_TEMPLATE_URL_MAX_LEN {
            return Err(AppError::ConstraintViolation(format!(
                "{kind} url is too long"
            )));
        }
        if !url.contains(LOOKUP_PLACEHOLDER) {
            return Err(AppError::ConstraintViolation(format!(
                "{kind} url must contain the {LOOKUP_PLACEHOLDER} placeholder"
            )));
        }
        let lower = url.to_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(AppError::ConstraintViolation(format!(
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
        return Err(AppError::ConstraintViolation("timezone is required".into()));
    }
    if timezone.chars().count() > TIMEZONE_MAX_LEN {
        return Err(AppError::ConstraintViolation("timezone is too long".into()));
    }
    if timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(AppError::ConstraintViolation(format!(
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
        return Err(AppError::ConstraintViolation(
            "default_phone_country_code must be a two-letter ISO country code".into(),
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
