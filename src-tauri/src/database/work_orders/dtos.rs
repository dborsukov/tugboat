use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::database::{
    cars::dtos::CarDto,
    clients::structs::Client,
    work_orders::structs::{LaborItem, PartItem, WorkOrder},
};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderDetailsDto {
    pub work_order: WorkOrder,
    pub car_dto: CarDto,
    pub client: Client,
    pub labor_items: Vec<LaborItemDto>,
    pub part_items: Vec<PartItemDto>,
    pub totals: WorkOrderTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderTotals {
    pub labor_total: Decimal,
    pub parts_total: Decimal,
    pub subtotal: Decimal,
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
    pub subtotal: Decimal,
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
    pub retail_price: Decimal,
    pub discounted_price: Decimal,
    pub total: Decimal,
}
