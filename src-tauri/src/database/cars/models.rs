use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use tauri::State;

use crate::{database::cars::brands, error::AppResult};

fn catalog_models() -> &'static HashMap<String, Vec<String>> {
    static CATALOG_MODELS: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

    CATALOG_MODELS.get_or_init(|| match brands::catalog() {
        Ok(brands) => brands
            .into_iter()
            .filter(|brand| !brand.models.is_empty())
            .map(|brand| (brand.slug, brand.models))
            .collect(),
        Err(error) => {
            log::error!("Каталог моделей: не вдалося прочитати каталог марок: {error}");
            HashMap::new()
        }
    })
}

/// Curated catalog list first, in catalog order, then custom names in the database.
#[tauri::command]
#[specta::specta]
pub async fn fetch_car_models(
    state: State<'_, crate::AppState>,
    brand_id: i32,
) -> AppResult<Vec<String>> {
    let slug: Option<String> = sqlx::query_scalar("SELECT slug FROM car_brands WHERE id = $1")
        .bind(brand_id)
        .fetch_optional(&state.db_pool)
        .await?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut suggestions: Vec<String> = Vec::new();

    if let Some(models) = slug.as_deref().and_then(|slug| catalog_models().get(slug)) {
        for name in models {
            if seen.insert(brands::name_key(name)) {
                suggestions.push(name.clone());
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
    suggestions.extend(typed.into_iter().map(|(_, name, _)| name));

    Ok(suggestions)
}
