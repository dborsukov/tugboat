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
    #[specta(type = specta_typescript::Number)]
    pub labor_total_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub parts_total_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub subtotal_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub grand_total_cents: i64,
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
    #[specta(type = specta_typescript::Number)]
    pub subtotal_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub total_cents: i64,
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
    #[specta(type = specta_typescript::Number)]
    pub retail_price_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub discounted_price_cents: i64,
    #[specta(type = specta_typescript::Number)]
    pub total_cents: i64,
}
