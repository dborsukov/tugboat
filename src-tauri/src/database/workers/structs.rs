use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Worker {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub first_name: String,
    pub surname: String,
    pub patronymic: Option<String>,
    pub position: Option<String>,
    pub is_active: bool,
}

impl<'r> FromRow<'r, SqliteRow> for Worker {
    fn from_row(row: &'r SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            first_name: row.try_get("first_name")?,
            surname: row.try_get("surname")?,
            patronymic: row.try_get("patronymic")?,
            position: row.try_get("position")?,
            is_active: row.try_get("is_active")?,
        })
    }
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkerPayload {
    pub first_name: String,
    pub surname: String,
    pub patronymic: Option<String>,
    pub position: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkerPayload {
    pub first_name: String,
    pub surname: String,
    pub patronymic: Option<String>,
    pub position: Option<String>,
    pub is_active: bool,
}
