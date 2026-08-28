pub mod dtos;
pub mod structs;

use sqlx::{Executor, Sqlite};
use structs::*;
use tauri::State;

use crate::{
    database::{
        cars::{dtos::CarDto, fetch_cars_list_by_client_id},
        clients::dtos::ClientDetailsDto,
    },
    error::{AppError, AppResult},
};

const CLIENT_COLUMNS: &str =
    "id, name, surname, first_name, patronymic, phone, email, client_type, notes, created_at";

const CLIENT_DETAILS_COLUMNS: &str = "client_id, legal_name, legal_address, edrpou, rnokpp, director_surname, director_first_name, director_patronymic, contact_surname, contact_first_name, contact_patronymic, contact_phone, contact_email, iban, bank_name, vat_payer";

struct ClientBase {
    name: String,
    surname: Option<String>,
    first_name: Option<String>,
    patronymic: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    notes: Option<String>,
}

fn trim_opt(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn compose_name(surname: &Option<String>, first_name: &str, patronymic: &Option<String>) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if let Some(s) = surname {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    let first = first_name.trim();
    if !first.is_empty() {
        parts.push(first);
    }
    if let Some(p) = patronymic {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" ")
}

fn payload_common_fields(payload: &ClientPayload) -> (ClientType, ClientBase) {
    match payload {
        ClientPayload::Individual {
            surname,
            first_name,
            patronymic,
            phone,
            email,
            notes,
        } => (
            ClientType::Individual,
            ClientBase {
                name: compose_name(surname, first_name, patronymic),
                surname: trim_opt(surname),
                first_name: Some(first_name.trim().to_string()),
                patronymic: trim_opt(patronymic),
                phone: phone.clone(),
                email: email.clone(),
                notes: notes.clone(),
            },
        ),
        ClientPayload::Fop {
            surname,
            first_name,
            patronymic,
            notes,
            ..
        } => (
            ClientType::Fop,
            ClientBase {
                name: compose_name(surname, first_name, patronymic),
                surname: trim_opt(surname),
                first_name: Some(first_name.trim().to_string()),
                patronymic: trim_opt(patronymic),
                phone: None,
                email: None,
                notes: notes.clone(),
            },
        ),
        ClientPayload::Company { name, notes, .. } => (
            ClientType::Company,
            ClientBase {
                name: name.trim().to_string(),
                surname: None,
                first_name: None,
                patronymic: None,
                phone: None,
                email: None,
                notes: notes.clone(),
            },
        ),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_clients_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_clients_list(
    state: State<'_, crate::AppState>,
) -> AppResult<Vec<ClientListItem>> {
    let clients: Vec<ClientListItem> = sqlx::query_as(
        "SELECT c.id, c.name, c.surname, c.first_name, c.patronymic, c.phone, c.email, c.client_type, c.notes, c.created_at, d.contact_phone, d.contact_email, d.rnokpp, d.edrpou, d.iban \
         FROM clients c \
         LEFT JOIN client_details d ON d.client_id = c.id",
    )
    .fetch_all(&state.db_pool)
    .await?;
    Ok(clients)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_client_details_dto(
    state: State<'_, crate::AppState>,
    id: i32,
) -> AppResult<ClientDetailsDto> {
    let client: Client = fetch_client_by_id(state.clone(), id).await?;
    let details: Option<ClientDetails> = fetch_client_details(state.clone(), id).await?;
    let cars: Vec<CarDto> = fetch_cars_list_by_client_id(state.clone(), client.id as i32).await?;
    Ok(ClientDetailsDto {
        client,
        details,
        cars,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_client_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<Client> {
    let client: Client = sqlx::query_as(&format!(
        "SELECT {CLIENT_COLUMNS} FROM clients WHERE id = $1"
    ))
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(client)
}

pub async fn fetch_client_details(
    state: State<'_, crate::AppState>,
    client_id: i32,
) -> AppResult<Option<ClientDetails>> {
    let details: Option<ClientDetails> = sqlx::query_as(&format!(
        "SELECT {CLIENT_DETAILS_COLUMNS} FROM client_details WHERE client_id = $1"
    ))
    .bind(client_id)
    .fetch_optional(&state.db_pool)
    .await?;
    Ok(details)
}

#[tauri::command]
#[specta::specta]
pub async fn create_client(
    state: State<'_, crate::AppState>,
    payload: ClientPayload,
) -> AppResult<Client> {
    validate_payload(&payload)?;

    let mut tx = state.db_pool.begin().await?;

    let (client_type, base) = payload_common_fields(&payload);

    let client: Client = sqlx::query_as(&format!(
        "INSERT INTO clients (name, surname, first_name, patronymic, phone, email, client_type, notes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING {CLIENT_COLUMNS}"
    ))
    .bind(&base.name)
    .bind(&base.surname)
    .bind(&base.first_name)
    .bind(&base.patronymic)
    .bind(&base.phone)
    .bind(&base.email)
    .bind(client_type)
    .bind(&base.notes)
    .fetch_one(&mut *tx)
    .await?;

    insert_client_details(&mut *tx, client.id, &payload).await?;

    tx.commit().await?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn update_client(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: ClientPayload,
) -> AppResult<Client> {
    validate_payload(&payload)?;

    let mut tx = state.db_pool.begin().await?;

    let (client_type, base) = payload_common_fields(&payload);

    let client: Client = sqlx::query_as(&format!(
        "UPDATE clients SET name = $2, surname = $3, first_name = $4, patronymic = $5, phone = $6, email = $7, client_type = $8, notes = $9 WHERE id = $1 RETURNING {CLIENT_COLUMNS}"
    ))
    .bind(id)
    .bind(&base.name)
    .bind(&base.surname)
    .bind(&base.first_name)
    .bind(&base.patronymic)
    .bind(&base.phone)
    .bind(&base.email)
    .bind(client_type)
    .bind(&base.notes)
    .fetch_one(&mut *tx)
    .await?;

    match payload_to_details(client.id, &payload) {
        Some(details) => upsert_client_details(&mut *tx, details).await?,
        None => {
            sqlx::query("DELETE FROM client_details WHERE client_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(client)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_client(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Клієнта з таким ID не знайдено в базі даних".to_string(),
        ));
    }
    Ok(())
}

fn payload_to_details(client_id: i64, payload: &ClientPayload) -> Option<ClientDetails> {
    match payload {
        ClientPayload::Individual { .. } => None,
        ClientPayload::Fop {
            rnokpp,
            legal_address,
            contact_surname,
            contact_first_name,
            contact_patronymic,
            contact_phone,
            contact_email,
            iban,
            bank_name,
            ..
        } => Some(ClientDetails {
            client_id,
            legal_name: None,
            legal_address: trim_opt(legal_address),
            edrpou: None,
            rnokpp: Some(rnokpp.clone()),
            director_surname: None,
            director_first_name: None,
            director_patronymic: None,
            contact_surname: trim_opt(contact_surname),
            contact_first_name: trim_opt(contact_first_name),
            contact_patronymic: trim_opt(contact_patronymic),
            contact_phone: trim_opt(contact_phone),
            contact_email: trim_opt(contact_email),
            iban: trim_opt(iban),
            bank_name: trim_opt(bank_name),
            vat_payer: false,
        }),
        ClientPayload::Company {
            legal_name,
            legal_address,
            edrpou,
            director_surname,
            director_first_name,
            director_patronymic,
            contact_surname,
            contact_first_name,
            contact_patronymic,
            contact_phone,
            contact_email,
            iban,
            bank_name,
            vat_payer,
            ..
        } => Some(ClientDetails {
            client_id,
            legal_name: Some(legal_name.trim().to_string()),
            legal_address: Some(legal_address.trim().to_string()),
            edrpou: Some(edrpou.trim().to_string()),
            rnokpp: None,
            director_surname: trim_opt(director_surname),
            director_first_name: trim_opt(director_first_name),
            director_patronymic: trim_opt(director_patronymic),
            contact_surname: trim_opt(contact_surname),
            contact_first_name: trim_opt(contact_first_name),
            contact_patronymic: trim_opt(contact_patronymic),
            contact_phone: trim_opt(contact_phone),
            contact_email: trim_opt(contact_email),
            iban: trim_opt(iban),
            bank_name: trim_opt(bank_name),
            vat_payer: *vat_payer,
        }),
    }
}

fn is_n_digits(value: &str, n: usize) -> bool {
    value.len() == n && value.bytes().all(|b| b.is_ascii_digit())
}

fn validate_payload(payload: &ClientPayload) -> AppResult<()> {
    match payload {
        ClientPayload::Individual { first_name, .. } => {
            if first_name.trim().is_empty() {
                return Err(AppError::Generic("Ім'я клієнта обов'язкове".to_string()));
            }
        }
        ClientPayload::Fop {
            first_name, rnokpp, ..
        } => {
            if first_name.trim().is_empty() {
                return Err(AppError::Generic("Ім'я клієнта обов'язкове".to_string()));
            }
            if !is_n_digits(rnokpp, 10) {
                return Err(AppError::Generic(
                    "РНОКПП має містити рівно 10 цифр".to_string(),
                ));
            }
        }
        ClientPayload::Company {
            name,
            legal_name,
            legal_address,
            edrpou,
            ..
        } => {
            if name.trim().is_empty() {
                return Err(AppError::Generic(
                    "Назва організації обов'язкова".to_string(),
                ));
            }
            if legal_name.trim().is_empty() {
                return Err(AppError::Generic(
                    "Юридична назва організації обов'язкова".to_string(),
                ));
            }
            if legal_address.trim().is_empty() {
                return Err(AppError::Generic(
                    "Юридична адреса організації обов'язкова".to_string(),
                ));
            }
            if !is_n_digits(edrpou, 8) {
                return Err(AppError::Generic(
                    "ЄДРПОУ має містити рівно 8 цифр".to_string(),
                ));
            }
        }
    }
    Ok(())
}

async fn insert_client_details<'e, E>(
    executor: E,
    client_id: i64,
    payload: &ClientPayload,
) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let Some(details) = payload_to_details(client_id, payload) else {
        return Ok(());
    };

    sqlx::query(&format!(
        "INSERT INTO client_details ({CLIENT_DETAILS_COLUMNS}) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"
    ))
    .bind(details.client_id)
    .bind(details.legal_name)
    .bind(details.legal_address)
    .bind(details.edrpou)
    .bind(details.rnokpp)
    .bind(details.director_surname)
    .bind(details.director_first_name)
    .bind(details.director_patronymic)
    .bind(details.contact_surname)
    .bind(details.contact_first_name)
    .bind(details.contact_patronymic)
    .bind(details.contact_phone)
    .bind(details.contact_email)
    .bind(details.iban)
    .bind(details.bank_name)
    .bind(details.vat_payer)
    .execute(executor)
    .await?;

    Ok(())
}

async fn upsert_client_details<'e, E>(executor: E, details: ClientDetails) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query(&format!(
        "INSERT INTO client_details ({CLIENT_DETAILS_COLUMNS}) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) \
         ON CONFLICT (client_id) DO UPDATE SET \
         legal_name = excluded.legal_name, \
         legal_address = excluded.legal_address, \
         edrpou = excluded.edrpou, \
         rnokpp = excluded.rnokpp, \
         director_surname = excluded.director_surname, \
         director_first_name = excluded.director_first_name, \
         director_patronymic = excluded.director_patronymic, \
         contact_surname = excluded.contact_surname, \
         contact_first_name = excluded.contact_first_name, \
         contact_patronymic = excluded.contact_patronymic, \
         contact_phone = excluded.contact_phone, \
         contact_email = excluded.contact_email, \
         iban = excluded.iban, \
         bank_name = excluded.bank_name, \
         vat_payer = excluded.vat_payer"
    ))
    .bind(details.client_id)
    .bind(details.legal_name)
    .bind(details.legal_address)
    .bind(details.edrpou)
    .bind(details.rnokpp)
    .bind(details.director_surname)
    .bind(details.director_first_name)
    .bind(details.director_patronymic)
    .bind(details.contact_surname)
    .bind(details.contact_first_name)
    .bind(details.contact_patronymic)
    .bind(details.contact_phone)
    .bind(details.contact_email)
    .bind(details.iban)
    .bind(details.bank_name)
    .bind(details.vat_payer)
    .execute(executor)
    .await?;

    Ok(())
}
