use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateServicePayload {
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServicePayload {
    pub code: Option<String>,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub labor_hours: f64,
}
