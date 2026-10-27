//! Part measurement units.
//!
//! The presets live in `data/part_units.json` so the UI and the Rust document
//! renderers read the same definitions. Custom units are kept in the existing
//! `part_items.unit_type` column, presets in `unit_key`.

use std::sync::OnceLock;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::error::{AppError, AppResult};

pub(crate) const DEFAULT_UNIT_KEY: &str = "pcs";
pub(crate) const CUSTOM_UNIT_MAX_LENGTH: usize = 12;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PartUnitPreset {
    pub key: String,
    #[allow(dead_code)]
    pub name: String,
    pub display: String,
    pub decimals: bool,
}

fn presets() -> &'static [PartUnitPreset] {
    static PRESETS: OnceLock<Vec<PartUnitPreset>> = OnceLock::new();
    PRESETS.get_or_init(|| {
        serde_json::from_str(include_str!("../../../data/part_units.json")).unwrap_or_else(
            |error| {
                log::error!("Не вдалося прочитати довідник одиниць вимірювання: {error}");
                Vec::new()
            },
        )
    })
}

pub(crate) fn preset(key: &str) -> Option<&'static PartUnitPreset> {
    presets().iter().find(|unit| unit.key == key)
}

pub(crate) fn normalize_unit(
    unit_key: Option<String>,
    unit_type: Option<String>,
) -> AppResult<(Option<String>, Option<String>)> {
    let key = unit_key
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty());
    let custom = unit_type
        .map(|unit| unit.trim().to_string())
        .filter(|unit| !unit.is_empty());

    if let Some(key) = key {
        let preset =
            preset(&key).ok_or_else(|| AppError::validation("Невідома одиниця вимірювання"))?;
        return Ok((Some(preset.key.clone()), None));
    }

    match custom {
        Some(custom) => {
            if custom.chars().count() > CUSTOM_UNIT_MAX_LENGTH {
                return Err(AppError::validation(format!(
                    "Одиниця вимірювання має містити не більше {CUSTOM_UNIT_MAX_LENGTH} символів"
                )));
            }
            Ok((None, Some(custom)))
        }
        None => Ok((Some(DEFAULT_UNIT_KEY.to_string()), None)),
    }
}

pub(crate) fn display_unit(unit_key: Option<&str>, unit_type: Option<&str>) -> String {
    if let Some(key) = unit_key {
        if let Some(preset) = preset(key) {
            return preset.display.clone();
        }
    }

    let custom = unit_type.map(str::trim).filter(|unit| !unit.is_empty());
    if let Some(custom) = custom {
        return custom.to_string();
    }

    preset(DEFAULT_UNIT_KEY)
        .map(|unit| unit.display.clone())
        .unwrap_or_default()
}

/// `2.50` -> `2,5`, `2` -> `2`.
pub(crate) fn format_qty_number(value: Decimal) -> String {
    value.normalize().to_string().replace('.', ",")
}

pub(crate) fn format_quantity_with_unit(value: Decimal, unit: &str) -> String {
    format!("{} {unit}", format_qty_number(value))
}

pub(crate) fn format_qty(
    value: Decimal,
    unit_key: Option<&str>,
    unit_type: Option<&str>,
) -> String {
    format_quantity_with_unit(value, &display_unit(unit_key, unit_type))
}

pub(crate) fn validate_quantity(
    quantity: Decimal,
    unit_key: Option<&str>,
    unit_type: Option<&str>,
) -> AppResult<()> {
    let allows_decimals = unit_key
        .and_then(preset)
        .map(|unit| unit.decimals)
        .unwrap_or(true);

    let normalized = quantity.normalize();
    if allows_decimals {
        if quantity < Decimal::new(1, 3) {
            return Err(AppError::validation("Кількість має бути не менше 0,001"));
        }
        if normalized.scale() > 3 {
            return Err(AppError::validation(
                "Кількість може мати не більше 3 знаків після коми",
            ));
        }
    } else {
        if quantity < Decimal::ONE || normalized.scale() > 0 {
            let name = display_unit(unit_key, unit_type);
            return Err(AppError::validation(format!(
                "Для одиниці «{name}» кількість має бути цілим числом не менше 1"
            )));
        }
    }

    Ok(())
}
