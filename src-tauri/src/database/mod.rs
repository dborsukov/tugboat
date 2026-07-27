use tauri::State;

pub mod structs;

use structs::*;

use crate::error::{AppError, AppResult};

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
pub async fn update_settings(
    settings: AppSettings,
    state: tauri::State<'_, crate::AppState>,
) -> AppResult<AppSettings> {
    let settings: AppSettings =
        sqlx::query_as("UPDATE app_settings SET theme = $1 WHERE id = 1 RETURNING *")
            .bind(settings.theme)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(settings)
}

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
pub async fn fetch_work_orders(state: State<'_, crate::AppState>) -> AppResult<Vec<WorkOrder>> {
    let work_orders: Vec<WorkOrder> = sqlx::query_as("SELECT * FROM work_orders")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(work_orders)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_labor_items(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrderLaborItem>> {
    let work_order_labor_items: Vec<WorkOrderLaborItem> =
        sqlx::query_as("SELECT * FROM work_order_labor_items")
            .fetch_all(&state.db_pool)
            .await?;
    Ok(work_order_labor_items)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_work_order_part_items(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<WorkOrderPartItem>> {
    let work_order_part_items: Vec<WorkOrderPartItem> =
        sqlx::query_as("SELECT * FROM work_order_part_items")
            .fetch_all(&state.db_pool)
            .await?;
    Ok(work_order_part_items)
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
    let service: Service = sqlx::query_as(
        "INSERT INTO services (code, name, labor_hours) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.labor_hours)
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
    let service: Service = sqlx::query_as(
        "UPDATE services SET code = $2, name = $3, labor_hours = $4 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(payload.code)
    .bind(payload.name)
    .bind(payload.labor_hours)
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

#[tauri::command]
#[specta::specta]
pub async fn create_work_order(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderPayload,
) -> AppResult<WorkOrder> {
    let work_order: WorkOrder =
        sqlx::query_as("INSERT INTO work_orders (id, car_id, created_at, odometer_in, customer_complaint, mechanic_notes, status, rate) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *")
            .bind(payload.id)
            .bind(payload.car_id)
            .bind(payload.created_at)
            .bind(payload.odometer_in)
            .bind(payload.customer_complaint)
            .bind(payload.mechanic_notes)
            .bind(payload.status)
            .bind(payload.rate)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(work_order)
}

#[tauri::command]
#[specta::specta]
pub async fn update_work_order(
    state: State<'_, crate::AppState>,
    payload: UpdateWorkOrderPayload,
) -> AppResult<WorkOrder> {
    let work_order: WorkOrder = sqlx::query_as(
        "UPDATE work_orders SET car_id = $2, completed_at = $3, odometer_in = $4, customer_complaint = $5, mechanic_notes = $6, status = $7, rate = $8 WHERE id = $1 RETURNING *",
    )
    .bind(payload.id)
    .bind(payload.car_id)
    .bind(payload.completed_at)
    .bind(payload.odometer_in)
    .bind(payload.customer_complaint)
    .bind(payload.mechanic_notes)
    .bind(payload.status)
    .bind(payload.rate)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(work_order)
}

#[tauri::command]
#[specta::specta]
pub async fn archive_work_order(state: State<'_, crate::AppState>, id: String) -> AppResult<()> {
    let result = sqlx::query("UPDATE work_orders SET is_archived = 1 WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Замовлення з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn create_work_order_labor_item(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderLaborItemPayload,
) -> AppResult<WorkOrderLaborItem> {
    let work_order_labor_item: WorkOrderLaborItem =
        sqlx::query_as("INSERT INTO work_order_labor_items (work_order_id, service_id, code, name, labor_hours) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(payload.work_order_id)
            .bind(payload.service_id)
            .bind(payload.code)
            .bind(payload.name)
            .bind(payload.labor_hours)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(work_order_labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_work_order_labor_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateWorkOrderLaborItemPayload,
) -> AppResult<WorkOrderLaborItem> {
    let work_order_labor_item: WorkOrderLaborItem = sqlx::query_as(
        "UPDATE work_order_labor_items SET service_id = $2, code = $3, name = $4, labor_hours = $5 WHERE id = $1 RETURNING *",
    )
        .bind(id)
        .bind(payload.service_id)
        .bind(payload.code)
        .bind(payload.name)
        .bind(payload.labor_hours)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(work_order_labor_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_work_order_labor_item(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM work_order_labor_items WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Сервісу з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn create_work_order_part_item(
    state: State<'_, crate::AppState>,
    payload: CreateWorkOrderPartItemPayload,
) -> AppResult<WorkOrderPartItem> {
    let work_order_part_item: WorkOrderPartItem =
        sqlx::query_as("INSERT INTO work_order_part_items (work_order_id, code, name, quantity, unit_type, unit_price_cents, status) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *")
            .bind(payload.work_order_id)
            .bind(payload.code)
            .bind(payload.name)
            .bind(payload.quantity)
            .bind(payload.unit_type)
            .bind(payload.unit_price_cents)
            .bind(payload.status)
            .fetch_one(&state.db_pool)
            .await?;
    Ok(work_order_part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn update_work_order_part_item(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateWorkOrderPartItemPayload,
) -> AppResult<WorkOrderPartItem> {
    let work_order_part_item: WorkOrderPartItem = sqlx::query_as(
        "UPDATE work_order_part_items SET code = $2, name = $3, quantity = $4, unit_type = $5, unit_price_cents = $6, status = $7 WHERE id = $1 RETURNING *",
    )
        .bind(id)
        .bind(payload.code)
        .bind(payload.name)
        .bind(payload.quantity)
        .bind(payload.unit_type)
        .bind(payload.unit_price_cents)
        .bind(payload.status)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(work_order_part_item)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_work_order_part_item(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM work_order_part_items WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Запчастини з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}
