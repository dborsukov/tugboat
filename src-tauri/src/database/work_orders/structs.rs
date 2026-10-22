use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::database::{utils::decimal_column, vat::structs::VatScheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum WorkOrderStatus {
    Received,
    InDiagnostics,
    AwaitingApproval,
    AwaitingParts,
    Working,
    ReadyForPickup,
    AwaitingPayment,
    /// Terminal state for no-payment orders (warranty claims)
    Completed,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum WorkOrderType {
    Regular,
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
    /// Client who pays for the order, may be different from the owner
    pub payer_client_id: Option<i32>,
    /// Client who initiated the order, maybe be different from the owner
    pub customer_client_id: Option<i32>,
    pub worker_id: Option<i32>,
    pub odometer_in: i32,
    pub odometer_out: Option<i32>,
    /// Driver or representative who delivers the vehicle
    pub driver_surname: Option<String>,
    pub driver_first_name: Option<String>,
    pub driver_patronymic: Option<String>,
    pub driver_phone: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub condition_report: Option<String>,
    pub status: WorkOrderStatus,
    pub r#type: WorkOrderType,
    pub warranty_of: Option<String>,
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
    pub tax_invoice_number: Option<String>,
    pub tax_invoice_date: Option<String>,
    pub exemption_code: Option<String>,
    pub vat_scheme: VatScheme,
}

impl<'r> FromRow<'r, SqliteRow> for WorkOrder {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            car_id: row.try_get("car_id")?,
            payer_client_id: row.try_get("payer_client_id")?,
            customer_client_id: row.try_get("customer_client_id")?,
            worker_id: row.try_get("worker_id")?,
            odometer_in: row.try_get("odometer_in")?,
            odometer_out: row.try_get("odometer_out")?,
            driver_surname: row.try_get("driver_surname")?,
            driver_first_name: row.try_get("driver_first_name")?,
            driver_patronymic: row.try_get("driver_patronymic")?,
            driver_phone: row.try_get("driver_phone")?,
            description: row.try_get("description")?,
            notes: row.try_get("notes")?,
            condition_report: row.try_get("condition_report")?,
            status: row.try_get("status")?,
            r#type: row.try_get("type")?,
            warranty_of: row.try_get("warranty_of")?,
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
            tax_invoice_number: row.try_get("tax_invoice_number")?,
            tax_invoice_date: row.try_get("tax_invoice_date")?,
            exemption_code: row.try_get("exemption_code")?,
            vat_scheme: row.try_get("vat_scheme")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkOrderPayload {
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub payer_client_id: Option<i32>,
    pub customer_client_id: Option<i32>,
    pub worker_id: Option<i32>,
    pub odometer_in: i32,
    pub odometer_out: Option<i32>,
    pub driver_surname: Option<String>,
    pub driver_first_name: Option<String>,
    pub driver_patronymic: Option<String>,
    pub driver_phone: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub condition_report: Option<String>,
    pub status: WorkOrderStatus,
    pub rate: Decimal,
    pub labor_cost_type: LaborCostType,
    pub labor_cost_value: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub vat_rate: Decimal,
    pub prices_include_vat: bool,
    pub vat_scheme: VatScheme,
    pub tax_invoice_number: Option<String>,
    pub tax_invoice_date: Option<String>,
    pub exemption_code: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkOrderPayload {
    pub id: String,
    #[specta(type = specta_typescript::Number)]
    pub car_id: i64,
    pub payer_client_id: Option<i32>,
    pub customer_client_id: Option<i32>,
    pub worker_id: Option<i32>,
    pub odometer_in: i32,
    pub odometer_out: Option<i32>,
    pub driver_surname: Option<String>,
    pub driver_first_name: Option<String>,
    pub driver_patronymic: Option<String>,
    pub driver_phone: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub condition_report: Option<String>,
    pub status: WorkOrderStatus,
    pub rate: Decimal,
    pub labor_cost_type: LaborCostType,
    pub labor_cost_value: Decimal,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub vat_rate: Decimal,
    pub prices_include_vat: bool,
    pub vat_scheme: VatScheme,
    pub tax_invoice_number: Option<String>,
    pub tax_invoice_date: Option<String>,
    pub exemption_code: Option<String>,
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
    pub is_free: bool,
    pub position: i32,
    pub vat_rate: Option<Decimal>,
    pub vat_scheme: Option<VatScheme>,
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
            is_free: row.try_get("is_free")?,
            position: row.try_get("position")?,
            vat_rate: row
                .try_get::<Option<String>, _>("vat_rate")?
                .and_then(|raw| raw.parse().ok()),
            vat_scheme: row.try_get("vat_scheme")?,
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
    pub is_free: bool,
    pub vat_rate: Option<Decimal>,
    pub vat_scheme: Option<VatScheme>,
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
    pub is_free: bool,
    pub vat_rate: Option<Decimal>,
    pub vat_scheme: Option<VatScheme>,
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
    pub is_free: bool,
    pub position: i32,
    pub vat_rate: Option<Decimal>,
    pub vat_scheme: Option<VatScheme>,
    pub purchase_vat_rate: Decimal,
    pub purchase_price_includes_vat: bool,
    pub purchase_scheme: VatScheme,
    pub supplier_name: Option<String>,
    pub supplier_code: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub vat_credited: bool,
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
            is_free: row.try_get("is_free")?,
            position: row.try_get("position")?,
            vat_rate: row
                .try_get::<Option<String>, _>("vat_rate")?
                .and_then(|raw| raw.parse().ok()),
            vat_scheme: row.try_get("vat_scheme")?,
            purchase_vat_rate: row
                .try_get::<String, _>("purchase_vat_rate")?
                .parse()
                .unwrap_or(Decimal::ZERO),
            purchase_price_includes_vat: row.try_get("purchase_price_includes_vat")?,
            purchase_scheme: row.try_get("purchase_scheme")?,
            supplier_name: row.try_get("supplier_name")?,
            supplier_code: row.try_get("supplier_code")?,
            invoice_number: row.try_get("invoice_number")?,
            invoice_date: row.try_get("invoice_date")?,
            vat_credited: row.try_get("vat_credited")?,
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
    pub is_free: bool,
    pub vat_rate: Option<Decimal>,
    pub vat_scheme: Option<VatScheme>,
    pub purchase_vat_rate: Decimal,
    pub purchase_price_includes_vat: bool,
    pub purchase_scheme: VatScheme,
    pub supplier_name: Option<String>,
    pub supplier_code: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub vat_credited: bool,
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
    pub is_free: bool,
    pub vat_rate: Option<Decimal>,
    pub vat_scheme: Option<VatScheme>,
    pub purchase_vat_rate: Decimal,
    pub purchase_price_includes_vat: bool,
    pub purchase_scheme: VatScheme,
    pub supplier_name: Option<String>,
    pub supplier_code: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub vat_credited: bool,
}
