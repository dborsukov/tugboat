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
    pub subtotal: Decimal,
    pub labor_discount: Decimal,
    pub parts_discount: Decimal,
    pub order_discount: Decimal,
    pub grand_total: Decimal,
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
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
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
}
