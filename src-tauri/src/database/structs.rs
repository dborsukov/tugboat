use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(
    // Builtins
    Debug,
    Clone,
    Copy,
    // Comparisons and hashing
    PartialEq,
    Eq,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    sqlx::Type,
    specta::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: ThemeMode,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
pub struct Client {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateClientPayload {
    pub name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateClientPayload {
    pub name: String,
    pub phone: Option<String>,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
pub struct CarBrand {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub icon_slug: String,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
pub struct Car {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub client_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub brand_id: i64,
    pub created_at: DateTime<Utc>,
    pub model: String,
    #[specta(type = specta_typescript::Number)]
    pub year: i64,
    pub registration: Option<String>,
    pub vin: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateCarPayload {
    #[specta(type = specta_typescript::Number)]
    pub client_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub brand_id: i64,
    pub model: String,
    #[specta(type = specta_typescript::Number)]
    pub year: i64,
    pub registration: Option<String>,
    pub vin: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateCarPayload {
    #[specta(type = specta_typescript::Number)]
    pub client_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub brand_id: i64,
    pub model: String,
    #[specta(type = specta_typescript::Number)]
    pub year: i64,
    pub registration: Option<String>,
    pub vin: Option<String>,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
pub struct Service {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateServicePayload {
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateServicePayload {
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    Copy,
    // Comparisons and hashing
    PartialEq,
    Eq,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    sqlx::Type,
    specta::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum WorkOrderStatus {
    Received,
    InDiagnostics,
    AwaitingApproval,
    AwaitingParts,
    Working,
    ReadyForPickup,
    Paid,
    Canceled,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
pub struct WorkOrder {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub odometer_in: i32,
    pub customer_complaint: Option<String>,
    pub mechanic_notes: Option<String>,
    pub status: WorkOrderStatus,
    #[specta(type = specta_typescript::Number)]
    pub rate: i64,
    pub is_archived: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateWorkOrderPayload {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub created_at: String,
    pub odometer_in: i32,
    pub customer_complaint: Option<String>,
    pub mechanic_notes: Option<String>,
    pub status: WorkOrderStatus,
    #[specta(type = specta_typescript::Number)]
    pub rate: i64,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateWorkOrderPayload {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub odometer_in: i32,
    pub customer_complaint: Option<String>,
    pub mechanic_notes: Option<String>,
    pub status: WorkOrderStatus,
    #[specta(type = specta_typescript::Number)]
    pub rate: i64,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
pub struct WorkOrderLaborItem {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub work_order_id: String,
    #[specta(type = Option<specta_typescript::Number>)]
    pub service_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateWorkOrderLaborItemPayload {
    pub work_order_id: String,
    #[specta(type = Option<specta_typescript::Number>)]
    pub service_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateWorkOrderLaborItemPayload {
    #[specta(type = Option<specta_typescript::Number>)]
    pub service_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    Copy,
    // Comparisons and hashing
    PartialEq,
    Eq,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    sqlx::Type,
    specta::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum PartStatus {
    Quoted,
    Ordered,
    InStock,
    Installing,
    Installed,
    AwaitingReturn,
    Returned,
    Warranty,
}

#[derive(
    // Builtins
    Debug,
    Clone,
    // Thrid-party: Serde
    Serialize,
    Deserialize,
    // Thrid-party: Sqlx
    FromRow,
    // Thrid-party: Specta
    specta::Type,
)]
pub struct WorkOrderPartItem {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub work_order_id: String,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub quantity: f64,
    pub unit_type: String,
    #[specta(type = specta_typescript::Number)]
    pub unit_price_cents: i64,
    pub status: PartStatus,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateWorkOrderPartItemPayload {
    pub work_order_id: String,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub quantity: f64,
    pub unit_type: String,
    #[specta(type = specta_typescript::Number)]
    pub unit_price_cents: i64,
    pub status: PartStatus,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateWorkOrderPartItemPayload {
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub quantity: f64,
    pub unit_type: String,
    #[specta(type = specta_typescript::Number)]
    pub unit_price_cents: i64,
    pub status: PartStatus,
}
