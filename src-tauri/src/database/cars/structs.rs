use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum FuelType {
    Gasoline,
    Diesel,
    LpgCng,
    Hybrid,
    Electric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum DriveType {
    FrontWheel,
    RearWheel,
    AllWheel,
    FourWheel,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Car {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub client_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub brand_id: i64,
    pub model: String,
    #[specta(type = specta_typescript::Number)]
    pub year: i64,
    pub engine_code: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub engine_displacement: Option<f64>,
    pub fuel_type: Option<FuelType>,
    pub drive_type: Option<DriveType>,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateCarPayload {
    #[specta(type = specta_typescript::Number)]
    pub client_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub brand_id: i64,
    pub model: String,
    #[specta(type = specta_typescript::Number)]
    pub year: i64,
    pub engine_code: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub engine_displacement: Option<f64>,
    pub fuel_type: Option<FuelType>,
    pub drive_type: Option<DriveType>,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCarPayload {
    #[specta(type = specta_typescript::Number)]
    pub client_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub brand_id: i64,
    pub model: String,
    #[specta(type = specta_typescript::Number)]
    pub year: i64,
    pub engine_code: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub engine_displacement: Option<f64>,
    pub fuel_type: Option<FuelType>,
    pub drive_type: Option<DriveType>,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CarBrand {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub icon_slug: String,
}
