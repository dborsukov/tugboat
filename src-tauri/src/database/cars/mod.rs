pub mod brands;
pub mod dtos;
pub mod models;
pub mod plate;
pub mod structs;

use std::collections::HashMap;

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Pool, QueryBuilder, Sqlite};
use structs::*;
use tauri::State;

use crate::{
    database::{
        cars::dtos::{CarDetailsDto, CarDto},
        clients,
        clients::{fetch_client_by_id, fetch_clients_list, structs::Client},
        utils::{begin_immediate, optional_text},
    },
    error::{AppError, AppResult},
    logging::Event,
};

#[tauri::command]
#[specta::specta]
pub async fn fetch_cars_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cars WHERE archived_at IS NULL")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_cars_list(state: State<'_, crate::AppState>) -> AppResult<Vec<CarDto>> {
    let cars: Vec<Car> = sqlx::query_as(
        r#"
        SELECT
            id,
            client_id,
            brand_id,
            model,
            year,
            license_plate,
            plate_type,
            plate_country,
            plate_search_key,
            vin,
            engine_code,
            engine_displacement,
            fuel_type,
            drive_type,
            notes,
            created_at,
            archived_at
        FROM cars
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    build_car_dtos(state.clone(), cars).await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_cars_list_by_client_id(
    state: State<'_, crate::AppState>,
    client_id: i32,
) -> AppResult<Vec<CarDto>> {
    let cars: Vec<Car> = sqlx::query_as(
        r#"
        SELECT
            id,
            client_id,
            brand_id,
            model,
            year,
            license_plate,
            plate_type,
            plate_country,
            plate_search_key,
            vin,
            engine_code,
            engine_displacement,
            fuel_type,
            drive_type,
            notes,
            created_at,
            archived_at
        FROM cars
        WHERE client_id = $1
        "#,
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await?;

    build_car_dtos(state.clone(), cars).await
}

pub(crate) async fn build_car_dtos(
    state: State<'_, crate::AppState>,
    cars: Vec<Car>,
) -> AppResult<Vec<CarDto>> {
    build_car_dtos_with(&state.db_pool, cars).await
}

pub(crate) async fn build_car_dtos_with(
    pool: &Pool<Sqlite>,
    cars: Vec<Car>,
) -> AppResult<Vec<CarDto>> {
    let brands: Vec<CarBrand> = brands::fetch_all_brands(pool).await?;

    let brands_by_id: HashMap<i64, CarBrand> =
        brands.into_iter().map(|brand| (brand.id, brand)).collect();

    let fallback = brands::unknown_brand(&brands_by_id);

    let car_dtos: Vec<CarDto> = cars
        .into_iter()
        .map(|car| {
            let brand = brands_by_id
                .get(&car.brand_id)
                .cloned()
                .unwrap_or_else(|| fallback.clone());
            CarDto { car, brand }
        })
        .collect();

    Ok(car_dtos)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_details_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<CarDetailsDto>> {
    let car_dtos: Vec<CarDto> = fetch_cars_list(state.clone()).await?;

    let clients: Vec<Client> = fetch_clients_list(state)
        .await?
        .into_iter()
        .map(Client::from)
        .collect();

    let clients_by_id: HashMap<i64, Client> = clients
        .into_iter()
        .map(|client| (client.id, client))
        .collect();

    let car_details_dtos: Vec<CarDetailsDto> = car_dtos
        .into_iter()
        .filter_map(|car_dto| {
            clients_by_id
                .get(&car_dto.car.client_id)
                .cloned()
                .map(|client| CarDetailsDto { car_dto, client })
        })
        .collect();

    Ok(car_details_dtos)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_details(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<CarDetailsDto> {
    let car_dto: CarDto = fetch_car_by_id(state.clone(), id).await?;
    let client: Client = fetch_client_by_id(state.clone(), car_dto.car.client_id as i32).await?;
    Ok(CarDetailsDto { car_dto, client })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<CarDto> {
    let car: Car = sqlx::query_as(
        r#"
        SELECT
            id,
            client_id,
            brand_id,
            model,
            year,
            license_plate,
            plate_type,
            plate_country,
            plate_search_key,
            vin,
            engine_code,
            engine_displacement,
            fuel_type,
            drive_type,
            notes,
            created_at,
            archived_at
        FROM cars
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;

    let brand = brands::fetch_brand_by_id(&state.db_pool, car.brand_id).await?;

    Ok(CarDto { car, brand })
}

pub(crate) async fn fetch_cars_by_ids(
    state: &State<'_, crate::AppState>,
    ids: &[i64],
) -> AppResult<HashMap<i64, CarDto>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut qb = QueryBuilder::new(
        r#"
        SELECT
            id,
            client_id,
            brand_id,
            model,
            year,
            license_plate,
            plate_type,
            plate_country,
            plate_search_key,
            vin,
            engine_code,
            engine_displacement,
            fuel_type,
            drive_type,
            notes,
            created_at,
            archived_at
        FROM cars
        WHERE id IN ("#,
    );
    let mut separated = qb.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    qb.push(")");
    let cars: Vec<Car> = qb.build_query_as().fetch_all(&state.db_pool).await?;

    let mut brand_ids: Vec<i64> = Vec::new();
    for car in &cars {
        if !brand_ids.contains(&car.brand_id) {
            brand_ids.push(car.brand_id);
        }
    }

    let mut qb = QueryBuilder::new(format!(
        "SELECT {} FROM car_brands WHERE id IN (",
        brands::BRAND_COLUMNS
    ));
    let mut separated = qb.separated(", ");
    for id in &brand_ids {
        separated.push_bind(id);
    }
    qb.push(")");
    let brands: Vec<CarBrand> = qb.build_query_as().fetch_all(&state.db_pool).await?;
    let brands_by_id: HashMap<i64, CarBrand> =
        brands.into_iter().map(|brand| (brand.id, brand)).collect();

    let fallback = brands::unknown_brand(&brands_by_id);

    Ok(cars
        .into_iter()
        .map(|car| {
            let brand = brands_by_id
                .get(&car.brand_id)
                .cloned()
                .unwrap_or_else(|| fallback.clone());
            (car.id, CarDto { car, brand })
        })
        .collect())
}

const MAX_MODEL_LENGTH: usize = 100;
const MAX_VIN_LENGTH: usize = 32;
const MAX_ENGINE_CODE_LENGTH: usize = 60;
const MAX_NOTES_LENGTH: usize = 2000;
const MIN_YEAR: i64 = 1950;

/// The car fields the plate helpers do not cover. Values are normalized so a
/// stored row is comparable: `vin` is upper-cased and stripped of separators,
/// because the unique index on `cars.vin` compares raw strings.
#[derive(Debug)]
struct CarFields {
    model: String,
    year: i64,
    vin: Option<String>,
    engine_code: Option<String>,
    engine_displacement: Option<f64>,
    notes: Option<String>,
}

impl CarFields {
    fn validate(
        model: String,
        year: i64,
        vin: Option<String>,
        engine_code: Option<String>,
        engine_displacement: Option<f64>,
        notes: Option<String>,
    ) -> AppResult<Self> {
        let model = model.trim();
        if model.is_empty() {
            return Err(AppError::validation("Вкажіть модель автомобіля"));
        }
        if model.chars().count() > MAX_MODEL_LENGTH {
            return Err(AppError::validation(format!(
                "Модель не може бути довшою за {MAX_MODEL_LENGTH} символів"
            )));
        }

        let max_year = i64::from(Utc::now().year()) + 1;
        if !(MIN_YEAR..=max_year).contains(&year) {
            return Err(AppError::validation(format!(
                "Рік випуску має бути в межах {MIN_YEAR}–{max_year}"
            )));
        }

        if let Some(value) = engine_displacement {
            if !value.is_finite() || value <= 0.0 {
                return Err(AppError::validation(
                    "Об'єм двигуна має бути більшим за нуль",
                ));
            }
        }

        let engine_code = optional_text(engine_code, MAX_ENGINE_CODE_LENGTH, "Код двигуна")?;

        Ok(Self {
            model: model.to_string(),
            year,
            vin: normalize_vin(vin)?,
            engine_code,
            engine_displacement,
            notes: optional_text(notes, MAX_NOTES_LENGTH, "Нотатки")?,
        })
    }
}

fn normalize_vin(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(vin) = raw else {
        return Ok(None);
    };

    let normalized: String = vin
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-')
        .flat_map(char::to_uppercase)
        .collect();

    if normalized.is_empty() {
        return Ok(None);
    }
    if normalized.chars().count() > MAX_VIN_LENGTH {
        return Err(AppError::validation(format!(
            "VIN не може бути довшим за {MAX_VIN_LENGTH} символів"
        )));
    }

    Ok(Some(normalized))
}

#[tauri::command]
#[specta::specta]
pub async fn create_car(
    state: State<'_, crate::AppState>,
    payload: CreateCarPayload,
) -> AppResult<Car> {
    let fields = CarFields::validate(
        payload.model,
        payload.year,
        payload.vin,
        payload.engine_code,
        payload.engine_displacement,
        payload.notes,
    )?;

    let mut tx = begin_immediate(&state.db_pool).await?;

    clients::ensure_client_active(&mut *tx, payload.client_id as i32).await?;

    let (license_plate, plate_type, plate_country, plate_search_key) =
        plate::normalize_plate_fields(
            payload.plate_type,
            payload.license_plate,
            payload.plate_country,
        )?;

    let car: Car = sqlx::query_as(
        r#"
        INSERT INTO cars (
            client_id,
            brand_id,
            model,
            year,
            license_plate,
            plate_type,
            plate_country,
            plate_search_key,
            vin,
            engine_code,
            engine_displacement,
            fuel_type,
            drive_type,
            notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING
            id,
            client_id,
            brand_id,
            model,
            year,
            license_plate,
            plate_type,
            plate_country,
            plate_search_key,
            vin,
            engine_code,
            engine_displacement,
            fuel_type,
            drive_type,
            notes,
            created_at,
            archived_at
        "#,
    )
    .bind(payload.client_id)
    .bind(payload.brand_id)
    .bind(fields.model)
    .bind(fields.year)
    .bind(license_plate)
    .bind(plate_type)
    .bind(plate_country)
    .bind(plate_search_key)
    .bind(fields.vin)
    .bind(fields.engine_code)
    .bind(fields.engine_displacement)
    .bind(payload.fuel_type)
    .bind(payload.drive_type)
    .bind(fields.notes)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("car.create").id(car.id).ok();

    Ok(car)
}

#[tauri::command]
#[specta::specta]
pub async fn update_car(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateCarPayload,
) -> AppResult<Car> {
    let fields = CarFields::validate(
        payload.model,
        payload.year,
        payload.vin,
        payload.engine_code,
        payload.engine_displacement,
        payload.notes,
    )?;

    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_car_active(&mut *tx, id).await?;
    clients::ensure_client_active(&mut *tx, payload.client_id as i32).await?;

    let (license_plate, plate_type, plate_country, plate_search_key) =
        plate::normalize_plate_fields(
            payload.plate_type,
            payload.license_plate,
            payload.plate_country,
        )?;

    let car: Car = sqlx::query_as(
        r#"
        UPDATE cars SET
            client_id = $2,
            brand_id = $3,
            model = $4,
            year = $5,
            license_plate = $6,
            plate_type = $7,
            plate_country = $8,
            plate_search_key = $9,
            vin = $10,
            engine_code = $11,
            engine_displacement = $12,
            fuel_type = $13,
            drive_type = $14,
            notes = $15
        WHERE id = $1
        RETURNING
            id,
            client_id,
            brand_id,
            model,
            year,
            license_plate,
            plate_type,
            plate_country,
            plate_search_key,
            vin,
            engine_code,
            engine_displacement,
            fuel_type,
            drive_type,
            notes,
            created_at,
            archived_at
        "#,
    )
    .bind(id)
    .bind(payload.client_id)
    .bind(payload.brand_id)
    .bind(fields.model)
    .bind(fields.year)
    .bind(license_plate)
    .bind(plate_type)
    .bind(plate_country)
    .bind(plate_search_key)
    .bind(fields.vin)
    .bind(fields.engine_code)
    .bind(fields.engine_displacement)
    .bind(payload.fuel_type)
    .bind(payload.drive_type)
    .bind(fields.notes)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("car.update").id(id).ok();

    Ok(car)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_car(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    delete_car_guarded(&state.db_pool, id).await
}

pub(crate) async fn delete_car_guarded(pool: &sqlx::Pool<Sqlite>, id: i32) -> AppResult<()> {
    let mut tx = begin_immediate(pool).await?;

    let work_orders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_orders WHERE car_id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    if work_orders > 0 {
        Event::new("car.delete")
            .id(id)
            .field("work_orders", work_orders)
            .denied("has_work_orders");
        tx.rollback().await?;
        return Err(AppError::validation(format!(
            "В автомобіля {} {}. Архівуйте автомобіль, щоб зберегти історію замовлень.",
            work_orders,
            crate::utils::plural(
                work_orders as usize,
                "замовлення",
                "замовлення",
                "замовлень"
            ),
        )));
    }

    let result = sqlx::query("DELETE FROM cars WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::car_not_found());
    }

    tx.commit().await?;

    Event::new("car.delete").id(id).ok();

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CarDeleteImpact {
    #[specta(type = specta_typescript::Number)]
    pub work_order_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub labor_item_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub part_item_count: i64,
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_delete_impact(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<CarDeleteImpact> {
    let mut tx = state.db_pool.begin().await?;

    let work_order_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_orders WHERE car_id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

    let labor_item_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM labor_items li \
         JOIN work_orders wo ON wo.id = li.work_order_id \
         WHERE wo.car_id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    let part_item_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM part_items pi \
         JOIN work_orders wo ON wo.id = pi.work_order_id \
         WHERE wo.car_id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(CarDeleteImpact {
        work_order_count,
        labor_item_count,
        part_item_count,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn archive_car(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let result = sqlx::query(
        r#"
        UPDATE cars SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = $1
          AND archived_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::car_not_found());
    }

    sqlx::query(
        r#"
        UPDATE work_orders SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE car_id = $1
          AND archived_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("car.archive").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_car(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let mut tx = begin_immediate(&state.db_pool).await?;

    let result = sqlx::query("UPDATE cars SET archived_at = NULL WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::car_not_found());
    }

    sqlx::query("UPDATE work_orders SET archived_at = NULL WHERE car_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Event::new("car.restore").id(id).ok();

    Ok(())
}

pub(crate) async fn ensure_car_active<'e, E>(executor: E, id: i32) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let archived_at: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM cars WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await?;
    match archived_at {
        None => Err(AppError::car_not_found()),
        Some(Some(_)) => Err(AppError::car_archived()),
        Some(None) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn car(model: &str, year: i64) -> AppResult<CarFields> {
        CarFields::validate(model.to_string(), year, None, None, None, None)
    }

    #[test]
    fn trims_model_and_blanks_empty_optional_fields() {
        let fields = CarFields::validate(
            "  Octavia  ".to_string(),
            Utc::now().year() as i64,
            Some("   ".to_string()),
            Some("  CJSA  ".to_string()),
            Some(1.8),
            Some("  нотатка  ".to_string()),
        )
        .expect("valid car");

        assert_eq!(fields.model, "Octavia");
        assert_eq!(fields.vin, None);
        assert_eq!(fields.engine_code.as_deref(), Some("CJSA"));
        assert_eq!(fields.notes.as_deref(), Some("нотатка"));
    }

    #[test]
    fn rejects_blank_model() {
        assert!(car("   ", 2020).is_err());
    }

    #[test]
    fn rejects_year_outside_range() {
        assert!(car("Octavia", MIN_YEAR - 1).is_err());
        assert!(car("Octavia", i64::from(Utc::now().year()) + 2).is_err());
    }

    #[test]
    fn accepts_current_year_and_next_year() {
        let current = i64::from(Utc::now().year());
        assert!(car("Octavia", current).is_ok());
        assert!(car("Octavia", current + 1).is_ok());
    }

    #[test]
    fn normalizes_vin() {
        let fields = CarFields::validate(
            "Octavia".to_string(),
            2020,
            Some("wbx-123 456".to_string()),
            None,
            None,
            None,
        )
        .expect("valid car");

        assert_eq!(fields.vin.as_deref(), Some("WBX123456"));
    }

    #[test]
    fn rejects_non_positive_engine_displacement() {
        let rejected = |value: f64| {
            CarFields::validate("Octavia".to_string(), 2020, None, None, Some(value), None).is_err()
        };
        assert!(rejected(0.0));
        assert!(rejected(-1.6));
        assert!(rejected(f64::NAN));
    }
}

#[cfg(test)]
mod delete_tests {
    use super::*;
    use crate::database::test_support::{count, pool, seed_client_with_order};

    #[test]
    fn refuses_to_delete_a_car_that_has_work_orders() {
        tauri::async_runtime::block_on(async {
            let pool = pool().await;
            seed_client_with_order(&pool).await;

            let error = delete_car_guarded(&pool, 1)
                .await
                .expect_err("a car with orders is guarded");
            assert!(matches!(error, AppError::Validation(_)));
            assert_eq!(count(&pool, "cars").await, 1);

            sqlx::query("DELETE FROM work_orders")
                .execute(&pool)
                .await
                .expect("clear orders");
            delete_car_guarded(&pool, 1)
                .await
                .expect("a car without orders can be deleted");

            assert_eq!(count(&pool, "cars").await, 0);
        });
    }
}
