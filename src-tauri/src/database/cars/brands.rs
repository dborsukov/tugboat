use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::Deserialize;
use sqlx::{Executor, Pool, Sqlite};
use tauri::State;

use crate::{
    database::{cars::structs::CarBrand, utils::begin_immediate},
    error::{AppError, AppResult},
    logging::Event,
};

const CATALOG_JSON: &str = include_str!("../../../data/car_brands.json");

const MAX_NAME_LENGTH: usize = 60;
pub(crate) const UNKNOWN_BRAND_SLUG: &str = "unknown-brand";

pub(crate) const BRAND_COLUMNS: &str = "id, slug, name, icon_slug, is_custom, archived_at";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogModel {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) aliases: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogBrand {
    pub(crate) slug: String,
    pub(crate) name: String,
    /// Logo file name, only declared when it differs from the slug.
    #[serde(default)]
    pub(crate) icon_slug: Option<String>,
    #[serde(default)]
    pub(crate) aliases: Vec<String>,
    #[serde(default)]
    pub(crate) models: Vec<CatalogModel>,
}

pub(crate) struct CatalogEntry {
    pub(crate) aliases: Vec<String>,
    pub(crate) models: Vec<CatalogModel>,
}

fn catalog_index() -> &'static HashMap<String, CatalogEntry> {
    static INDEX: OnceLock<HashMap<String, CatalogEntry>> = OnceLock::new();

    INDEX.get_or_init(|| match catalog() {
        Ok(brands) => brands
            .into_iter()
            .map(|brand| {
                (
                    brand.slug,
                    CatalogEntry {
                        aliases: brand.aliases,
                        models: brand.models,
                    },
                )
            })
            .collect(),
        Err(error) => {
            log::error!("Каталог марок: не вдалося прочитати каталог: {error}");
            HashMap::new()
        }
    })
}

pub(crate) fn catalog_entry(slug: &str) -> Option<&'static CatalogEntry> {
    catalog_index().get(slug)
}

fn attach_aliases(brands: &mut [CarBrand]) {
    for brand in brands.iter_mut() {
        if brand.is_custom {
            continue;
        }

        if let Some(entry) = catalog_entry(&brand.slug) {
            brand.aliases = entry.aliases.clone();
        }
    }
}

pub(crate) fn catalog() -> AppResult<Vec<CatalogBrand>> {
    serde_json::from_str(CATALOG_JSON).map_err(|error| {
        AppError::Generic(format!(
            "Не вдалося прочитати каталог марок автомобілів: {error}"
        ))
    })
}

/// Applies the bundled catalog to the database.
///
/// Runs on every launch, existing rows are matched by `slug`, which keeps
/// their ids and therefore every `cars.brand_id` stable across launches.
pub(crate) async fn sync_catalog(pool: &Pool<Sqlite>) -> AppResult<()> {
    let catalog = catalog()?;

    let mut tx = begin_immediate(pool).await?;

    let existing: Vec<(String, String)> = sqlx::query_as("SELECT slug, name_key FROM car_brands")
        .fetch_all(&mut *tx)
        .await?;
    let mut name_owners: HashMap<String, String> = existing.into_iter().collect();

    let mut synced = 0usize;

    for brand in &catalog {
        let key = name_key(&brand.name);

        if let Some(owner) = name_owners.get(&key) {
            if owner != &brand.slug {
                log::warn!(
                    "Каталог марок: «{}» пропущено, цю назву вже використовує марка «{owner}»",
                    brand.name
                );
                continue;
            }
        }

        let icon_slug = brand.icon_slug.as_deref().unwrap_or(&brand.slug);

        // A single bad row must not keep the app from starting, so failures are
        // logged and skipped rather than propagated (SQLite rolls back only the
        // failing statement, the transaction stays usable).
        let result = sqlx::query(
            r#"
            INSERT INTO car_brands (slug, name, name_key, icon_slug, is_custom)
            VALUES ($1, $2, $3, $4, 0)
            ON CONFLICT (slug) DO UPDATE SET
                name = excluded.name,
                name_key = excluded.name_key,
                icon_slug = excluded.icon_slug
            WHERE car_brands.is_custom = 0
            "#,
        )
        .bind(&brand.slug)
        .bind(&brand.name)
        .bind(&key)
        .bind(icon_slug)
        .execute(&mut *tx)
        .await;

        match result {
            Ok(done) => {
                if done.rows_affected() > 0 {
                    synced += 1;
                }
            }
            Err(error) => {
                log::error!(
                    "Каталог марок: не вдалося синхронізувати «{}»: {error}",
                    brand.slug
                );
            }
        }

        name_owners.insert(key, brand.slug.clone());
    }

    tx.commit().await?;

    log::info!("Каталог марок: синхронізовано {synced} з {}", catalog.len());

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_brands(state: State<'_, crate::AppState>) -> AppResult<Vec<CarBrand>> {
    fetch_picker_brands(&state.db_pool).await
}

pub(crate) async fn fetch_picker_brands(pool: &Pool<Sqlite>) -> AppResult<Vec<CarBrand>> {
    let sql = format!(
        "SELECT {BRAND_COLUMNS} FROM car_brands \
         WHERE archived_at IS NULL \
         ORDER BY id"
    );
    let mut brands: Vec<CarBrand> = sqlx::query_as(&sql).fetch_all(pool).await?;
    attach_aliases(&mut brands);
    Ok(brands)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_custom_car_brands(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<CarBrand>> {
    fetch_custom_brands(&state.db_pool).await
}

pub(crate) async fn fetch_custom_brands(pool: &Pool<Sqlite>) -> AppResult<Vec<CarBrand>> {
    let sql = format!(
        "SELECT {BRAND_COLUMNS} FROM car_brands \
         WHERE is_custom = 1 \
         ORDER BY name COLLATE NOCASE"
    );
    Ok(sqlx::query_as(&sql).fetch_all(pool).await?)
}

pub(crate) async fn fetch_all_brands(pool: &Pool<Sqlite>) -> AppResult<Vec<CarBrand>> {
    let sql = format!("SELECT {BRAND_COLUMNS} FROM car_brands ORDER BY id");
    let mut brands: Vec<CarBrand> = sqlx::query_as(&sql).fetch_all(pool).await?;
    attach_aliases(&mut brands);
    Ok(brands)
}

pub(crate) async fn fetch_brand_by_id(pool: &Pool<Sqlite>, id: i64) -> AppResult<CarBrand> {
    let sql = format!("SELECT {BRAND_COLUMNS} FROM car_brands WHERE id = $1");
    let mut brand: CarBrand = sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(AppError::brand_not_found)?;
    attach_aliases(std::slice::from_mut(&mut brand));
    Ok(brand)
}

#[tauri::command]
#[specta::specta]
pub async fn create_car_brand(
    state: State<'_, crate::AppState>,
    name: String,
) -> AppResult<CarBrand> {
    create_brand(&state.db_pool, &name).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_car_brand(
    state: State<'_, crate::AppState>,
    id: i32,
    name: String,
) -> AppResult<CarBrand> {
    update_brand(&state.db_pool, id as i64, &name).await
}

#[tauri::command]
#[specta::specta]
pub async fn archive_car_brand(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    set_brand_archived(&state.db_pool, id as i64, true).await
}

#[tauri::command]
#[specta::specta]
pub async fn restore_car_brand(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    set_brand_archived(&state.db_pool, id as i64, false).await
}

pub(crate) fn unknown_brand(brands: &HashMap<i64, CarBrand>) -> CarBrand {
    brands
        .values()
        .find(|brand| brand.slug == UNKNOWN_BRAND_SLUG)
        .cloned()
        .unwrap_or_else(|| CarBrand {
            id: 0,
            slug: UNKNOWN_BRAND_SLUG.to_string(),
            name: "Невідома марка".to_string(),
            icon_slug: UNKNOWN_BRAND_SLUG.to_string(),
            is_custom: false,
            archived_at: None,
            aliases: Vec::new(),
        })
}

pub(crate) async fn create_brand(pool: &Pool<Sqlite>, name: &str) -> AppResult<CarBrand> {
    let name = validate_name(name)?;
    let key = name_key(&name);

    let mut tx = begin_immediate(pool).await?;

    if let Some(existing) = fetch_by_name_key(&mut *tx, &key).await? {
        return Err(duplicate_name_error(&existing));
    }

    let slug = unique_slug(&mut *tx, &slugify(&key)).await?;

    // `icon_slug` is left empty, custom brands don't support icons
    let sql = format!(
        r#"
        INSERT INTO car_brands (slug, name, name_key, is_custom)
        VALUES ($1, $2, $3, 1)
        RETURNING {BRAND_COLUMNS}
        "#
    );
    let brand: CarBrand = sqlx::query_as(&sql)
        .bind(&slug)
        .bind(&name)
        .bind(&key)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Event::new("car_brand.create")
        .id(brand.id)
        .text("name", &name)
        .ok();

    Ok(brand)
}

pub(crate) async fn update_brand(pool: &Pool<Sqlite>, id: i64, name: &str) -> AppResult<CarBrand> {
    let name = validate_name(name)?;
    let key = name_key(&name);

    let mut tx = begin_immediate(pool).await?;

    let brand = fetch_by_id(&mut *tx, id).await?;
    ensure_custom(&brand, "перейменувати")?;

    if let Some(existing) = fetch_by_name_key(&mut *tx, &key).await? {
        if existing.id != brand.id {
            return Err(duplicate_name_error(&existing));
        }
    }

    // slug remains unchanged
    let sql = format!(
        r#"
        UPDATE car_brands SET name = $2, name_key = $3
        WHERE id = $1
        RETURNING {BRAND_COLUMNS}
        "#
    );
    let brand: CarBrand = sqlx::query_as(&sql)
        .bind(id)
        .bind(&name)
        .bind(&key)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Event::new("car_brand.update")
        .id(brand.id)
        .text("name", &name)
        .ok();

    Ok(brand)
}

pub(crate) async fn set_brand_archived(
    pool: &Pool<Sqlite>,
    id: i64,
    archived: bool,
) -> AppResult<()> {
    let mut tx = begin_immediate(pool).await?;

    let brand = fetch_by_id(&mut *tx, id).await?;
    ensure_custom(
        &brand,
        if archived {
            "архівувати"
        } else {
            "відновити"
        },
    )?;

    sqlx::query(
        r#"
        UPDATE car_brands
        SET archived_at = CASE WHEN $2 THEN strftime('%Y-%m-%dT%H:%M:%SZ', 'now') END
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(archived)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new(if archived {
        "car_brand.archive"
    } else {
        "car_brand.restore"
    })
    .id(id)
    .ok();

    Ok(())
}

fn ensure_custom(brand: &CarBrand, action: &str) -> AppResult<()> {
    if brand.is_custom {
        return Ok(());
    }

    Err(AppError::validation(format!(
        "Марку «{}» з довідника не можна {action}. Створіть власну марку, якщо потрібна інша назва.",
        brand.name
    )))
}

fn duplicate_name_error(existing: &CarBrand) -> AppError {
    if existing.archived_at.is_some() {
        AppError::validation(format!(
            "Марка «{}» вже є, але в архіві — відновіть її в налаштуваннях",
            existing.name
        ))
    } else {
        AppError::validation(format!("Марка «{}» вже є в довіднику", existing.name))
    }
}

fn validate_name(name: &str) -> AppResult<String> {
    let name = name.trim();

    if name.is_empty() {
        return Err(AppError::validation("Назва марки не може бути порожньою"));
    }

    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(AppError::validation(format!(
            "Назва марки не може бути довшою за {MAX_NAME_LENGTH} символів"
        )));
    }

    Ok(name.to_string())
}

pub(crate) fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

fn slugify(key: &str) -> String {
    let mut slug = String::with_capacity(key.len());
    let mut pending_dash = false;

    for character in key.chars() {
        if character.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(character);
        } else {
            pending_dash = true;
        }
    }

    if slug.is_empty() {
        slug.push_str("brand");
    }

    slug
}

async fn unique_slug<'e, E>(executor: E, base: &str) -> AppResult<String>
where
    E: Executor<'e, Database = Sqlite>,
{
    let taken: Vec<String> = sqlx::query_scalar("SELECT slug FROM car_brands WHERE slug LIKE $1")
        .bind(format!("{base}%"))
        .fetch_all(executor)
        .await?;
    let taken: HashSet<&str> = taken.iter().map(String::as_str).collect();

    if !taken.contains(base) {
        return Ok(base.to_string());
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !taken.contains(candidate.as_str()) {
            return Ok(candidate);
        }
        suffix += 1;
    }
}

async fn fetch_by_id<'e, E>(executor: E, id: i64) -> AppResult<CarBrand>
where
    E: Executor<'e, Database = Sqlite>,
{
    let sql = format!("SELECT {BRAND_COLUMNS} FROM car_brands WHERE id = $1");
    sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or_else(AppError::brand_not_found)
}

async fn fetch_by_name_key<'e, E>(executor: E, key: &str) -> AppResult<Option<CarBrand>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let sql = format!("SELECT {BRAND_COLUMNS} FROM car_brands WHERE name_key = $1");
    Ok(sqlx::query_as(&sql)
        .bind(key)
        .fetch_optional(executor)
        .await?)
}
