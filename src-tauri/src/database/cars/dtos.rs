use serde::{Deserialize, Serialize};

use crate::database::{
    cars::structs::{Car, CarBrand},
    clients::structs::Client,
};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CarDto {
    pub car: Car,
    pub brand: CarBrand,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CarDetailsDto {
    pub car_dto: CarDto,
    pub client: Client,
}
