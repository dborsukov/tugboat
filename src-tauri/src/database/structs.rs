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
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum InvoiceStatus {
    New,
    WaitingForParts,
    Working,
    Paid,
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
    pub name: String,
    pub unit_price: i64,
    pub is_archived: bool,
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
pub struct Invoice {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub created_at: String,
    pub status: InvoiceStatus,
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
pub struct InvoiceItem {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub invoice_id: i64,
    pub service_id: Option<i64>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub quantity: i64,
    #[specta(type = specta_typescript::Number)]
    pub unit_price: i64,
}
