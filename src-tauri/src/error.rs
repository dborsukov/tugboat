#[derive(
    // Builtins
    Debug,
    // Third-party
    thiserror::Error,
    serde::Serialize,
    specta::Type,
)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    ConstraintViolation(String),
    #[error("{0}")]
    Sqlx(String),
    #[error("{0}")]
    Generic(String),
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        log::error!("Database failure: {:?}", error.to_string());
        match error {
            sqlx::Error::RowNotFound => {
                AppError::NotFound("Помилка БД: Запис не знайдено в базі даних".to_string())
            }

            sqlx::Error::Database(db_err) => {
                match db_err.code().as_deref() {
                    // Code 787 = SQLITE_CONSTRAINT_FOREIGNKEY
                    Some("787") => AppError::ConstraintViolation(
                        "Помилка БД: Недійсний зв'язок між таблицями або спроба видалити запис, який використовується".to_string()
                    ),
                    // Code 2067 = SQLITE_CONSTRAINT_UNIQUE, 1555 = PRIMARY KEY
                    Some("2067") | Some("1555") => AppError::ConstraintViolation(
                        "Помилка БД: Такий запис (або його унікальне поле) вже існує.".to_string()
                    ),
                    _ => AppError::Sqlx(db_err.message().to_string()),
                }
            }

            _ => AppError::Sqlx(format!("Помилка БД: {}", error)),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
