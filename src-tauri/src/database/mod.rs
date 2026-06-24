use tauri::State;

pub mod structs;

use structs::*;

use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub async fn fetch_clients(state: State<'_, crate::AppState>) -> AppResult<Vec<Client>> {
    let clients: Vec<Client> = sqlx::query_as("SELECT * FROM clients")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(clients)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_cars(state: State<'_, crate::AppState>) -> AppResult<Vec<Car>> {
    let cars: Vec<Car> = sqlx::query_as("SELECT * FROM cars")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(cars)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_car_brands(state: State<'_, crate::AppState>) -> AppResult<Vec<CarBrand>> {
    let car_brands: Vec<CarBrand> = sqlx::query_as("SELECT * FROM car_brands")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(car_brands)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_services(state: State<'_, crate::AppState>) -> AppResult<Vec<Service>> {
    let services: Vec<Service> = sqlx::query_as("SELECT * FROM services")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(services)
}

#[tauri::command]
#[specta::specta]
pub async fn create_client(
    state: State<'_, crate::AppState>,
    payload: CreateClientPayload,
) -> AppResult<Client> {
    let client: Client =
        sqlx::query_as("INSERT INTO clients (name, phone) VALUES ($1, $2) RETURNING *")
            .bind(payload.name)
            .bind(payload.phone)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn update_client(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateClientPayload,
) -> AppResult<Client> {
    let client: Client =
        sqlx::query_as("UPDATE clients SET name = $1, phone = $2 WHERE id = $3 RETURNING *")
            .bind(payload.name)
            .bind(payload.phone)
            .bind(id)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_client(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Клієнта з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn create_car(
    state: State<'_, crate::AppState>,
    payload: CreateCarPayload,
) -> AppResult<Car> {
    let car: Car =
        sqlx::query_as("INSERT INTO cars (client_id, brand_id, model, year, registration, vin) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(payload.client_id)
            .bind(payload.brand_id)
            .bind(payload.model)
            .bind(payload.year)
            .bind(payload.registration)
            .bind(payload.vin)
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
        sqlx::query_as("UPDATE cars SET client_id = $2, brand_id = $3, model = $4, year = $5, registration = $6, vin = $7  WHERE id = $1 RETURNING *")
            .bind(id)
            .bind(payload.client_id)
            .bind(payload.brand_id)
            .bind(payload.model)
            .bind(payload.year)
            .bind(payload.registration)
            .bind(payload.vin)
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

#[tauri::command]
#[specta::specta]
pub async fn create_service(
    state: State<'_, crate::AppState>,
    payload: CreateServicePayload,
) -> AppResult<Service> {
    let service: Service =
        sqlx::query_as("INSERT INTO services (name, unit_price) VALUES ($1, $2) RETURNING *")
            .bind(payload.name)
            .bind(payload.unit_price)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(service)
}

#[tauri::command]
#[specta::specta]
pub async fn update_service(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateServicePayload,
) -> AppResult<Service> {
    let service: Service =
        sqlx::query_as("UPDATE services SET name = $1, unit_price = $2 WHERE id = $3 RETURNING *")
            .bind(payload.name)
            .bind(payload.unit_price)
            .bind(id)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(service)
}

#[tauri::command]
#[specta::specta]
pub async fn archive_service(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("UPDATE services SET is_archived = 1 WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Сервіса з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}
