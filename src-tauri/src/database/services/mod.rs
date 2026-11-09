pub mod structs;

use rust_decimal::Decimal;
use sqlx::{Executor, Sqlite, Transaction};
use structs::*;
use tauri::State;

use crate::{
    database::{utils::begin_immediate, work_orders::structs::LaborPriceType},
    error::{AppError, AppResult},
    logging::Event,
};

const MAX_NAME_LENGTH: usize = 200;
const MAX_CODE_LENGTH: usize = 32;
const MAX_FIXED_PRICE: u64 = 10_000_000;
const MAX_LABOR_HOURS: u64 = 10_000;

#[tauri::command]
#[specta::specta]
pub async fn fetch_services_count(state: State<'_, crate::AppState>) -> AppResult<i32> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE archived_at IS NULL")
        .fetch_one(&state.db_pool)
        .await?;
    Ok(count as i32)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_services_list(state: State<'_, crate::AppState>) -> AppResult<Vec<Service>> {
    let services: Vec<Service> = sqlx::query_as(
        r#"
        SELECT
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        FROM services
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;
    Ok(services)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_service_by_id(state: State<'_, crate::AppState>, id: i32) -> AppResult<Service> {
    let service: Service = sqlx::query_as(
        r#"
        SELECT
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        FROM services
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;
    Ok(service)
}

#[derive(Debug)]
struct ServiceFields {
    code: Option<String>,
    name: String,
    name_key: String,
    code_key: Option<String>,
    price_type: LaborPriceType,
    fixed_price: Decimal,
    labor_hours: Decimal,
}

impl ServiceFields {
    fn validate(
        code: Option<String>,
        name: String,
        price_type: LaborPriceType,
        fixed_price: Decimal,
        labor_hours: Decimal,
    ) -> AppResult<Self> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::validation("Вкажіть назву сервісу"));
        }
        if name.chars().count() > MAX_NAME_LENGTH {
            return Err(AppError::validation(format!(
                "Назва сервісу не може бути довшою за {MAX_NAME_LENGTH} символів"
            )));
        }

        let code = code.as_deref().map(str::trim).filter(|c| !c.is_empty());
        if let Some(code) = code {
            if code.chars().count() > MAX_CODE_LENGTH {
                return Err(AppError::validation(format!(
                    "Код сервісу не може бути довшим за {MAX_CODE_LENGTH} символів"
                )));
            }
        }

        if fixed_price.is_sign_negative() {
            return Err(AppError::validation("Ціна сервісу не може бути від'ємною"));
        }
        if labor_hours.is_sign_negative() {
            return Err(AppError::validation(
                "Кількість годин не може бути від'ємною",
            ));
        }
        if fixed_price > Decimal::from(MAX_FIXED_PRICE) {
            return Err(AppError::validation(format!(
                "Ціна сервісу не може перевищувати {MAX_FIXED_PRICE}"
            )));
        }
        if labor_hours > Decimal::from(MAX_LABOR_HOURS) {
            return Err(AppError::validation(format!(
                "Кількість годин не може перевищувати {MAX_LABOR_HOURS}"
            )));
        }

        let (fixed_price, labor_hours) = match price_type {
            LaborPriceType::Hourly => (Decimal::ZERO, labor_hours),
            LaborPriceType::Fixed => (fixed_price, Decimal::ZERO),
        };

        Ok(Self {
            code: code.map(str::to_string),
            name: name.to_string(),
            name_key: name.to_lowercase(),
            code_key: code.map(str::to_lowercase),
            price_type,
            fixed_price,
            labor_hours,
        })
    }
}

#[tauri::command]
#[specta::specta]
pub async fn create_service(
    state: State<'_, crate::AppState>,
    payload: CreateServicePayload,
) -> AppResult<Service> {
    let fields = ServiceFields::validate(
        payload.code,
        payload.name,
        payload.price_type,
        payload.fixed_price,
        payload.labor_hours,
    )?;

    let mut tx = begin_immediate(&state.db_pool).await?;
    ensure_service_unique(&mut tx, &fields.name_key, fields.code_key.as_deref(), None).await?;

    let service: Service = sqlx::query_as(
        r#"
        INSERT INTO services (
            code,
            code_key,
            name,
            name_key,
            price_type,
            fixed_price,
            labor_hours
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        "#,
    )
    .bind(fields.code)
    .bind(fields.code_key)
    .bind(fields.name)
    .bind(fields.name_key)
    .bind(fields.price_type)
    .bind(fields.fixed_price.to_string())
    .bind(fields.labor_hours.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("service.create").id(service.id).ok();

    Ok(service)
}

#[tauri::command]
#[specta::specta]
pub async fn update_service(
    state: State<'_, crate::AppState>,
    id: i32,
    payload: UpdateServicePayload,
) -> AppResult<Service> {
    let fields = ServiceFields::validate(
        payload.code,
        payload.name,
        payload.price_type,
        payload.fixed_price,
        payload.labor_hours,
    )?;

    let mut tx = begin_immediate(&state.db_pool).await?;

    ensure_service_active(&mut *tx, id).await?;
    ensure_service_unique(
        &mut tx,
        &fields.name_key,
        fields.code_key.as_deref(),
        Some(id),
    )
    .await?;

    let service: Service = sqlx::query_as(
        r#"
        UPDATE services SET
            code = $2,
            code_key = $3,
            name = $4,
            name_key = $5,
            price_type = $6,
            fixed_price = $7,
            labor_hours = $8
        WHERE id = $1
        RETURNING
            id,
            code,
            name,
            price_type,
            fixed_price,
            labor_hours,
            archived_at
        "#,
    )
    .bind(id)
    .bind(fields.code)
    .bind(fields.code_key)
    .bind(fields.name)
    .bind(fields.name_key)
    .bind(fields.price_type)
    .bind(fields.fixed_price.to_string())
    .bind(fields.labor_hours.to_string())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Event::new("service.update").id(id).ok();

    Ok(service)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_service(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM services WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::service_not_found());
    }

    Event::new("service.delete").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_services(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query("DELETE FROM services WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::service_not_found());
        }
    }
    tx.commit().await?;

    Event::new("service.delete_many")
        .field("count", ids.len())
        .ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn archive_service(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE services SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = $1
          AND archived_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&state.db_pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::service_not_found());
    }

    Event::new("service.archive").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn archive_services(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query(
            r#"
            UPDATE services SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
            WHERE id = $1
              AND archived_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::service_not_found());
        }
    }
    tx.commit().await?;

    Event::new("service.archive_many")
        .field("count", ids.len())
        .ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_service(state: State<'_, crate::AppState>, id: i32) -> AppResult<()> {
    let result = sqlx::query("UPDATE services SET archived_at = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::service_not_found());
    }

    Event::new("service.restore").id(id).ok();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_services(state: State<'_, crate::AppState>, ids: Vec<i32>) -> AppResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut tx = begin_immediate(&state.db_pool).await?;
    for id in &ids {
        let result = sqlx::query("UPDATE services SET archived_at = NULL WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::service_not_found());
        }
    }
    tx.commit().await?;

    Event::new("service.restore_many")
        .field("count", ids.len())
        .ok();

    Ok(())
}

async fn ensure_service_unique(
    tx: &mut Transaction<'static, Sqlite>,
    name_key: &str,
    code_key: Option<&str>,
    exclude_id: Option<i32>,
) -> AppResult<()> {
    let name_clash: Option<Option<String>> = sqlx::query_scalar(
        "SELECT archived_at FROM services
         WHERE name_key = $1
           AND ($2 IS NULL OR id <> $2)",
    )
    .bind(name_key)
    .bind(exclude_id)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(archived_at) = name_clash {
        return Err(if archived_at.is_some() {
            AppError::validation("Сервіс з такою назвою вже є в архіві — відновіть його")
        } else {
            AppError::validation("Сервіс з такою назвою вже існує")
        });
    }

    let Some(code_key) = code_key else {
        return Ok(());
    };

    let code_clash: Option<Option<String>> = sqlx::query_scalar(
        "SELECT archived_at FROM services
         WHERE code_key = $1
           AND ($2 IS NULL OR id <> $2)",
    )
    .bind(code_key)
    .bind(exclude_id)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(archived_at) = code_clash {
        return Err(if archived_at.is_some() {
            AppError::validation("Сервіс з таким кодом вже є в архіві — відновіть його")
        } else {
            AppError::validation("Сервіс з таким кодом вже існує")
        });
    }

    Ok(())
}

async fn ensure_service_active<'e, E>(executor: E, id: i32) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let archived_at: Option<Option<String>> =
        sqlx::query_scalar("SELECT archived_at FROM services WHERE id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await?;
    match archived_at {
        None => Err(AppError::service_not_found()),
        Some(Some(_)) => Err(AppError::service_archived()),
        Some(None) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hourly(name: &str, hours: Decimal) -> AppResult<ServiceFields> {
        ServiceFields::validate(
            None,
            name.to_string(),
            LaborPriceType::Hourly,
            Decimal::ZERO,
            hours,
        )
    }

    fn fixed(name: &str, price: Decimal) -> AppResult<ServiceFields> {
        ServiceFields::validate(
            None,
            name.to_string(),
            LaborPriceType::Fixed,
            price,
            Decimal::ZERO,
        )
    }

    #[test]
    fn trims_name_and_blanks_empty_code() {
        let fields = ServiceFields::validate(
            Some("   ".to_string()),
            "  Заміна оливи  ".to_string(),
            LaborPriceType::Hourly,
            Decimal::ZERO,
            Decimal::from(1),
        )
        .expect("valid payload");

        assert_eq!(fields.name, "Заміна оливи");
        assert_eq!(fields.code, None);
    }

    #[test]
    fn rejects_blank_name() {
        let error = hourly("   ", Decimal::from(1)).expect_err("blank name");
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn rejects_negative_amounts() {
        assert!(hourly("Робота", Decimal::from(-1)).is_err());
        assert!(fixed("Робота", Decimal::from(-1)).is_err());
    }

    #[test]
    fn zeroes_the_amount_column_price_type_does_not_use() {
        let hourly_fields = ServiceFields::validate(
            None,
            "Робота".to_string(),
            LaborPriceType::Hourly,
            Decimal::from(500),
            Decimal::from(2),
        )
        .expect("valid payload");
        assert!(hourly_fields.fixed_price.is_zero());
        assert_eq!(hourly_fields.labor_hours, Decimal::from(2));

        let fixed_fields = ServiceFields::validate(
            None,
            "Робота".to_string(),
            LaborPriceType::Fixed,
            Decimal::from(500),
            Decimal::from(2),
        )
        .expect("valid payload");
        assert_eq!(fixed_fields.fixed_price, Decimal::from(500));
        assert!(fixed_fields.labor_hours.is_zero());
    }

    #[test]
    fn rejects_out_of_range_amounts() {
        let too_many_hours = Decimal::from(MAX_LABOR_HOURS) + Decimal::ONE;
        assert!(hourly("Робота", too_many_hours).is_err());

        let too_expensive = Decimal::from(MAX_FIXED_PRICE) + Decimal::ONE;
        assert!(fixed("Робота", too_expensive).is_err());
    }

    #[test]
    fn rejects_overlong_name() {
        let name = "я".repeat(MAX_NAME_LENGTH + 1);
        assert!(hourly(&name, Decimal::from(1)).is_err());
    }
}

#[cfg(test)]
mod uniqueness_tests {
    use super::*;
    use crate::database::test_support::pool;

    async fn seed() -> sqlx::SqlitePool {
        let pool = pool().await;
        sqlx::query(
            "INSERT INTO services (id, code, code_key, name, name_key, price_type)
             VALUES (1, 'OL', 'ol', 'Заміна оливи', 'заміна оливи', 'fixed')",
        )
        .execute(&pool)
        .await
        .expect("seed service");
        pool
    }

    #[test]
    fn rejects_a_name_or_code_another_service_uses() {
        tauri::async_runtime::block_on(async {
            let pool = seed().await;
            let mut tx = begin_immediate(&pool).await.expect("tx");

            assert!(ensure_service_unique(&mut tx, "заміна оливи", None, None)
                .await
                .is_err());
            assert!(
                ensure_service_unique(&mut tx, "нова назва", Some("ol"), None)
                    .await
                    .is_err()
            );
            // A different name and no code is fine.
            assert!(ensure_service_unique(&mut tx, "нова назва", None, None)
                .await
                .is_ok());
        });
    }

    #[test]
    fn a_service_may_keep_its_own_name() {
        tauri::async_runtime::block_on(async {
            let pool = seed().await;
            let mut tx = begin_immediate(&pool).await.expect("tx");

            assert!(
                ensure_service_unique(&mut tx, "заміна оливи", Some("ol"), Some(1))
                    .await
                    .is_ok()
            );
        });
    }

    #[test]
    fn an_archived_service_is_reported_as_archived() {
        tauri::async_runtime::block_on(async {
            let pool = seed().await;
            sqlx::query("UPDATE services SET archived_at = '2026-01-01T00:00:00Z' WHERE id = 1")
                .execute(&pool)
                .await
                .expect("archive");

            let mut tx = begin_immediate(&pool).await.expect("tx");
            let error = ensure_service_unique(&mut tx, "заміна оливи", None, None)
                .await
                .expect_err("archived name is taken");
            assert!(error.to_string().contains("архіві"));
        });
    }
}
