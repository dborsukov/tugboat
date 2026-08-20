use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientPayload {
    pub name: String,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClientPayload {
    pub name: String,
    pub phone: Option<String>,
    pub notes: Option<String>,
}
