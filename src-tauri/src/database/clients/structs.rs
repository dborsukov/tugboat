use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum ClientType {
    Individual,
    Fop,
    Company,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    /// Composed display name: fullname for people, short/trade name for companies.
    pub name: String,
    pub surname: Option<String>,
    pub first_name: Option<String>,
    pub patronymic: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub client_type: ClientType,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientListItem {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub surname: Option<String>,
    pub first_name: Option<String>,
    pub patronymic: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub client_type: ClientType,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub rnokpp: Option<String>,
    pub edrpou: Option<String>,
    pub iban: Option<String>,
}

impl From<ClientListItem> for Client {
    fn from(item: ClientListItem) -> Self {
        Self {
            id: item.id,
            name: item.name,
            surname: item.surname,
            first_name: item.first_name,
            patronymic: item.patronymic,
            phone: item.phone,
            email: item.email,
            client_type: item.client_type,
            notes: item.notes,
            created_at: item.created_at,
            archived_at: item.archived_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientDetails {
    #[specta(type = specta_typescript::Number)]
    pub client_id: i64,
    pub legal_name: Option<String>,
    pub legal_address: Option<String>,
    pub edrpou: Option<String>,
    pub rnokpp: Option<String>,
    pub director_surname: Option<String>,
    pub director_first_name: Option<String>,
    pub director_patronymic: Option<String>,
    pub contact_surname: Option<String>,
    pub contact_first_name: Option<String>,
    pub contact_patronymic: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub iban: Option<String>,
    pub bank_name: Option<String>,
    pub vat_payer: bool,
    pub archived_at: Option<DateTime<Utc>>,
}

/// Discriminated payload
#[derive(Debug, Deserialize, specta::Type)]
#[serde(
    tag = "clientType",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ClientPayload {
    Individual {
        surname: Option<String>,
        first_name: String,
        patronymic: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        notes: Option<String>,
    },
    Fop {
        surname: Option<String>,
        first_name: String,
        patronymic: Option<String>,
        notes: Option<String>,
        rnokpp: String,
        legal_address: Option<String>,
        contact_surname: Option<String>,
        contact_first_name: Option<String>,
        contact_patronymic: Option<String>,
        contact_phone: Option<String>,
        contact_email: Option<String>,
        iban: Option<String>,
        bank_name: Option<String>,
    },
    Company {
        name: String,
        notes: Option<String>,
        legal_name: String,
        legal_address: String,
        edrpou: String,
        director_surname: Option<String>,
        director_first_name: Option<String>,
        director_patronymic: Option<String>,
        contact_surname: Option<String>,
        contact_first_name: Option<String>,
        contact_patronymic: Option<String>,
        contact_phone: Option<String>,
        contact_email: Option<String>,
        iban: Option<String>,
        bank_name: Option<String>,
        vat_payer: bool,
    },
}
