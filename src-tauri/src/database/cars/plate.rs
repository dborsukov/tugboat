use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PlateType {
    Ua,
    UaOld,
    Military,
    Temporary,
    Foreign,
    Other,
}

pub(crate) fn normalize_plate_text(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub(crate) fn plate_search_key(value: &str) -> String {
    normalize_plate_text(value)
        .unwrap_or_default()
        .chars()
        .filter_map(|ch| {
            if ch.is_whitespace() || ch == '-' {
                return None;
            }
            Some(match ch {
                'А' => 'A',
                'В' => 'B',
                'Е' => 'E',
                'К' => 'K',
                'М' => 'M',
                'Н' => 'H',
                'О' => 'O',
                'Р' => 'P',
                'С' => 'C',
                'Т' => 'T',
                'У' => 'Y',
                'Х' => 'X',
                'І' => 'I',
                other => other,
            })
        })
        .collect()
}

type NormalizedPlateFields = (Option<String>, PlateType, Option<String>, Option<String>);

pub(crate) fn normalize_plate_fields(
    plate_type: PlateType,
    plate: Option<String>,
    plate_country: Option<String>,
) -> AppResult<NormalizedPlateFields> {
    let plate = plate.as_deref().and_then(normalize_plate_text);
    let plate_search_key = plate
        .as_deref()
        .map(plate_search_key)
        .filter(|key| !key.is_empty());

    let country = plate_country
        .map(|value| value.trim().to_uppercase())
        .filter(|value| !value.is_empty());

    let country = match plate_type {
        PlateType::Foreign => {
            let country = country
                .ok_or_else(|| AppError::validation("Для іноземного номера вкажіть країну"))?;
            if country.len() != 2 || !country.chars().all(|ch| ch.is_ascii_alphabetic()) {
                return Err(AppError::validation(
                    "Країна іноземного номера має бути дволітерним ISO-кодом",
                ));
            }
            Some(country)
        }
        _ => None,
    };

    Ok((plate, plate_type, country, plate_search_key))
}
