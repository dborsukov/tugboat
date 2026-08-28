use serde::{Deserialize, Serialize};

use crate::database::{
    cars::dtos::CarDto,
    clients::structs::{Client, ClientDetails},
};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientDetailsDto {
    pub client: Client,
    pub details: Option<ClientDetails>,
    pub cars: Vec<CarDto>,
}
