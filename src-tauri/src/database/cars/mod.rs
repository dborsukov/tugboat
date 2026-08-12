pub mod dtos;
pub mod structs;

use std::collections::HashMap;

use structs::*;
use tauri::State;

use crate::{
    database::{
        cars::dtos::{CarDetailsDto, CarDto},
        clients::{fetch_client_by_id, fetch_clients_list, structs::Client},
    },
    error::{AppError, AppResult},
};

#[tauri::command]
#[specta::specta]
pub async fn fetch_cars_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cars")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_cars_list(state: State<'_, crate::AppState>) -> AppResult<Vec<CarDto>> {
    let cars: Vec<Car> = sqlx::query_as(
        "SELECT id, client_id, brand_id, model, year, license_plate, vin, engine_code, engine_displacement, fuel_type, drive_type, created_at FROM cars",
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
        "SELECT id, client_id, brand_id, model, year, license_plate, vin,  engine_code, engine_displacement, fuel_type, drive_type,created_at FROM cars WHERE client_id = $1",
    )
    .bind(client_id)
    .fetch_all(&state.db_pool)
    .await?;

    build_car_dtos(state.clone(), cars).await
}

pub async fn build_car_dtos(
    state: State<'_, crate::AppState>,
    cars: Vec<Car>,
) -> AppResult<Vec<CarDto>> {
    let brands: Vec<CarBrand> = fetch_car_brands(state).await?;

    let brands_by_id: HashMap<i64, CarBrand> =
        brands.into_iter().map(|brand| (brand.id, brand)).collect();

    let car_dtos: Vec<CarDto> = cars
        .into_iter()
        .filter_map(|car| {
            brands_by_id
                .get(&car.brand_id)
                .cloned()
                .map(|brand| CarDto { car, brand })
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

    let clients: Vec<Client> = fetch_clients_list(state).await?;

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
        "SELECT id, client_id, brand_id, model, year, license_plate, vin, engine_code, engine_displacement, fuel_type, drive_type, created_at FROM cars WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;

    let brand = fetch_car_brand_by_id(state, car.brand_id as i32).await?;

    Ok(CarDto { car, brand })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_brands(state: State<'_, crate::AppState>) -> AppResult<Vec<CarBrand>> {
    let car_brands: Vec<CarBrand> = sqlx::query_as("SELECT id, name, icon_slug FROM car_brands")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(car_brands)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_brand_by_id(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<CarBrand> {
    let car_brand: CarBrand =
        sqlx::query_as("SELECT id, name, icon_slug FROM car_brands WHERE id = $1")
            .bind(id)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(car_brand)
}

#[tauri::command]
#[specta::specta]
pub async fn create_car(
    state: State<'_, crate::AppState>,
    payload: CreateCarPayload,
) -> AppResult<Car> {
    let car: Car =
        sqlx::query_as("INSERT INTO cars (client_id, brand_id, model, year, license_plate, vin, engine_code, engine_displacement, fuel_type, drive_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *")
            .bind(payload.client_id)
            .bind(payload.brand_id)
            .bind(payload.model)
            .bind(payload.year)
            .bind(payload.license_plate)
            .bind(payload.vin)
            .bind(payload.engine_code)
            .bind(payload.engine_displacement)
            .bind(payload.fuel_type)
            .bind(payload.drive_type)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(car)
}

#[tauri::command]
#[specta::specta]
pub async fn update_car(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateCarPayload,
) -> AppResult<Car> {
    let car: Car =
        sqlx::query_as("UPDATE cars SET client_id = $2, brand_id = $3, model = $4, year = $5, license_plate = $6, vin = $7, engine_code = $8, engine_displacement = $9, fuel_type = $10, drive_type = $11  WHERE id = $1 RETURNING *")
            .bind(id)
            .bind(payload.client_id)
            .bind(payload.brand_id)
            .bind(payload.model)
            .bind(payload.year)
            .bind(payload.license_plate)
            .bind(payload.vin)
            .bind(payload.engine_code)
            .bind(payload.engine_displacement)
            .bind(payload.fuel_type)
            .bind(payload.drive_type)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(car)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_car(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM cars WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Автомобіля з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}
