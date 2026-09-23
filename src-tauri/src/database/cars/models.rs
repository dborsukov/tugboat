use std::collections::{HashMap, HashSet};

use serde::Serialize;
use tauri::State;

use crate::{database::cars::brands, error::AppResult};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CarModelSuggestion {
    pub name: String,
    pub aliases: Vec<String>,
}

fn suggestion(name: String, aliases: Vec<String>) -> CarModelSuggestion {
    CarModelSuggestion { name, aliases }
}

/// Curated catalog list first, in catalog order, then custom names in the database.
#[tauri::command]
#[specta::specta]
pub async fn fetch_car_models(
    state: State<'_, crate::AppState>,
    brand_id: i32,
) -> AppResult<Vec<CarModelSuggestion>> {
    let slug: Option<String> = sqlx::query_scalar("SELECT slug FROM car_brands WHERE id = $1")
        .bind(brand_id)
        .fetch_optional(&state.db_pool)
        .await?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut suggestions: Vec<CarModelSuggestion> = Vec::new();

    if let Some(entry) = slug.as_deref().and_then(brands::catalog_entry) {
        for model in &entry.models {
            if seen.insert(brands::name_key(&model.name)) {
                suggestions.push(suggestion(model.name.clone(), model.aliases.clone()));
            }
        }
    }

    let used: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT model
        FROM cars
        WHERE brand_id = $1 AND TRIM(model) <> ''
        ORDER BY created_at DESC
        "#,
    )
    .bind(brand_id)
    .fetch_all(&state.db_pool)
    .await?;

    // One entry per distinct name, spelled the way the newest car spells it.
    let mut typed_counts: HashMap<String, (String, usize)> = HashMap::new();
    for model in used {
        let name = model.trim().to_string();
        typed_counts
            .entry(brands::name_key(&name))
            .and_modify(|entry| entry.1 += 1)
            .or_insert((name, 1));
    }

    let mut typed: Vec<(String, String, usize)> = typed_counts
        .into_iter()
        .filter(|(key, _)| !seen.contains(key))
        .map(|(key, (name, count))| (key, name, count))
        .collect();

    typed.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    suggestions.extend(
        typed
            .into_iter()
            .map(|(_, name, _)| suggestion(name, Vec::new())),
    );

    Ok(suggestions)
}
