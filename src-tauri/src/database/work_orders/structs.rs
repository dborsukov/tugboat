use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::database::utils::decimal_column;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
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
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
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
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum LaborPriceType {
    Hourly,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum LaborCostType {
    Hourly,
    Percentage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum DiscountType {
    None,
    Fixed,
    Percentage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Card,
    BankTransfer,
    CardTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrder {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub odometer_in: i32,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub status: WorkOrderStatus,
    pub rate: Decimal,
    pub labor_cost_type: LaborCostType,
    pub labor_cost_value: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub vat_rate: Decimal,
    pub prices_include_vat: bool,
    pub created_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_method: Option<PaymentMethod>,
}

impl<'r> FromRow<'r, SqliteRow> for WorkOrder {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            car_id: row.try_get("car_id")?,
            odometer_in: row.try_get("odometer_in")?,
            description: row.try_get("description")?,
            notes: row.try_get("notes")?,
            status: row.try_get("status")?,
            rate: decimal_column(row, "rate")?,
            labor_cost_type: row.try_get("labor_cost_type")?,
            labor_cost_value: decimal_column(row, "labor_cost_value")?,
            discount_type: row.try_get("discount_type")?,
            discount_value: decimal_column(row, "discount_value")?,
            vat_rate: decimal_column(row, "vat_rate")?,
            prices_include_vat: row.try_get("prices_include_vat")?,
            created_at: row.try_get("created_at")?,
            archived_at: row.try_get("archived_at")?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            paid_at: row.try_get("paid_at")?,
            payment_method: row.try_get("payment_method")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkOrderPayload {
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub odometer_in: i32,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub status: WorkOrderStatus,
    pub rate: Decimal,
    pub labor_cost_type: LaborCostType,
    pub labor_cost_value: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub vat_rate: Decimal,
    pub prices_include_vat: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkOrderPayload {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub odometer_in: i32,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub status: WorkOrderStatus,
    pub rate: Decimal,
    pub labor_cost_type: LaborCostType,
    pub labor_cost_value: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub vat_rate: Decimal,
    pub prices_include_vat: bool,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_method: Option<PaymentMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LaborItem {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = Option<specta_typescript::Number>)]
    pub service_id: Option<i64>,
    pub work_order_id: String,
    pub code: Option<String>,
    pub name: String,
    pub price_type: LaborPriceType,
    pub fixed_price: Decimal,
    pub labor_hours: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
}

impl<'r> FromRow<'r, SqliteRow> for LaborItem {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            service_id: row.try_get("service_id")?,
            work_order_id: row.try_get("work_order_id")?,
            code: row.try_get("code")?,
            name: row.try_get("name")?,
            price_type: row.try_get("price_type")?,
            fixed_price: decimal_column(row, "fixed_price")?,
            labor_hours: decimal_column(row, "labor_hours")?,
            discount_type: row.try_get("discount_type")?,
            discount_value: decimal_column(row, "discount_value")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateLaborItemPayload {
    pub work_order_id: String,
    #[specta(type = Option<specta_typescript::Number>)]
    pub service_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    pub price_type: LaborPriceType,
    pub fixed_price: Decimal,
    pub labor_hours: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLaborItemPayload {
    #[specta(type = Option<specta_typescript::Number>)]
    pub service_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    pub price_type: LaborPriceType,
    pub fixed_price: Decimal,
    pub labor_hours: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PartItem {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub work_order_id: String,
    pub status: PartStatus,
    pub code: Option<String>,
    pub name: String,
    pub quantity: Decimal,
    pub unit_type: String,
    pub base_cost: Decimal,
    pub markup: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
}

impl<'r> FromRow<'r, SqliteRow> for PartItem {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            work_order_id: row.try_get("work_order_id")?,
            status: row.try_get("status")?,
            code: row.try_get("code")?,
            name: row.try_get("name")?,
            quantity: decimal_column(row, "quantity")?,
            unit_type: row.try_get("unit_type")?,
            base_cost: decimal_column(row, "base_cost")?,
            markup: decimal_column(row, "markup")?,
            discount_type: row.try_get("discount_type")?,
            discount_value: decimal_column(row, "discount_value")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreatePartItemPayload {
    pub work_order_id: String,
    pub status: PartStatus,
    pub code: Option<String>,
    pub name: String,
    pub quantity: Decimal,
    pub unit_type: String,
    pub base_cost: Decimal,
    pub markup: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePartItemPayload {
    pub status: PartStatus,
    pub code: Option<String>,
    pub name: String,
    pub quantity: Decimal,
    pub unit_type: String,
    pub base_cost: Decimal,
    pub markup: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
}
