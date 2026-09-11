use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::database::{
    cars::dtos::CarDto,
    clients::structs::{Client, ClientDetails},
    work_orders::structs::{LaborItem, PartItem, WorkOrder},
};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderDetailsDto {
    pub work_order: WorkOrder,
    pub car_dto: CarDto,
    pub client: Client,
    pub client_details: Option<ClientDetails>,
    pub labor_items: Vec<LaborItemDto>,
    pub part_items: Vec<PartItemDto>,
    pub totals: WorkOrderTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderTotals {
    pub labor_total: Decimal,
    pub labor_cost: Decimal,
    pub parts_total: Decimal,
    pub parts_cost: Decimal,
    /// Parts staged but not yet installed
    pub parts_pending_total: Decimal,
    pub subtotal: Decimal,
    pub labor_discount: Decimal,
    pub parts_discount: Decimal,
    pub order_discount: Decimal,
    pub grand_total: Decimal,
    /// Taxable base: total after discounts, excluding VAT.
    /// Equals `grand_total` when `prices_include_vat` is false.
    pub vat_base: Decimal,
    pub vat_amount: Decimal,
    pub total_with_vat: Decimal,
    pub below_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LaborItemDto {
    #[serde(flatten)]
    pub item: LaborItem,
    pub totals: LaborItemTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LaborItemTotals {
    pub cost: Decimal,
    pub price: Decimal,
    pub discounted_price: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub below_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PartItemDto {
    #[serde(flatten)]
    pub item: PartItem,
    pub totals: PartItemTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PartItemTotals {
    pub cost: Decimal,
    pub retail_price: Decimal,
    pub discounted_price: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub below_cost: bool,
}
