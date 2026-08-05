use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum DiscountType {
    Fixed,
    Percentage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum PaymentMethod {
    Cash,
    Card,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderTotal {
    pub work_order_id: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_total_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub parts_total_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub subtotal_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub grand_total_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrder {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub odometer_in: i32,
    pub customer_complaint: Option<String>,
    pub mechanic_notes: Option<String>,
    pub status: WorkOrderStatus,
    #[specta(type = specta_typescript::Number)]
    pub rate_cents: i64,
    pub discount_type: DiscountType,
    #[specta(type = specta_typescript::Number)]
    pub discount_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub discount_percent: f64,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_method: Option<PaymentMethod>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkOrderPayload {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub odometer_in: i32,
    pub customer_complaint: Option<String>,
    pub mechanic_notes: Option<String>,
    pub status: WorkOrderStatus,
    #[specta(type = specta_typescript::Number)]
    pub rate_cents: i64,
    pub discount_type: DiscountType,
    #[specta(type = specta_typescript::Number)]
    pub discount_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub discount_percent: f64,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkOrderPayload {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub odometer_in: i32,
    pub customer_complaint: Option<String>,
    pub mechanic_notes: Option<String>,
    pub status: WorkOrderStatus,
    #[specta(type = specta_typescript::Number)]
    pub rate_cents: i64,
    pub discount_type: DiscountType,
    #[specta(type = specta_typescript::Number)]
    pub discount_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub discount_percent: f64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_method: Option<PaymentMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkOrderLaborItemPayload {
    #[specta(type = Option<specta_typescript::Number>)]
    pub service_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
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
    pub cost_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub markup: f64,
    pub status: PartStatus,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkOrderPartItemPayload {
    pub work_order_id: String,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub quantity: f64,
    pub unit_type: String,
    #[specta(type = specta_typescript::Number)]
    pub cost_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub markup: f64,
    pub status: PartStatus,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkOrderPartItemPayload {
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub quantity: f64,
    pub unit_type: String,
    #[specta(type = specta_typescript::Number)]
    pub cost_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub markup: f64,
    pub status: PartStatus,
}
