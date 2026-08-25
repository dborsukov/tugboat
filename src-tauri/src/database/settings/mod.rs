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
