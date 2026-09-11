#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
#[serde(tag = "type", content = "message", rename_all = "camelCase")]
pub enum AppError {
    /// Generic application error.
    #[error("{0}")]
    Generic(String),
    /// Unclassified database failure.
    #[error("{0}")]
    Sqlx(String),
    /// The database is temporarily locked (SQLITE_BUSY / SQLITE_LOCKED).
    #[error("{0}")]
    Busy(String),
    /// Database connection / migration failure at startup.
    #[error("{0}")]
    Migration(String),
    /// Requested record does not exist.
    #[error("{0}")]
    NotFound(String),
    /// User-supplied input failed application-level validation.
    #[error("{0}")]
    Validation(String),
    /// A unique index / primary key constraint was violated.
    #[error("{0}")]
    UniqueViolation(String),
    /// A database constraint (CHECK / NOT NULL / trigger / etc.) was violated.
    #[error("{0}")]
    ConstraintViolation(String),
    /// A foreign key constraint was violated.
    #[error("{0}")]
    ForeignKeyViolation(String),
}

impl AppError {
    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub(crate) fn client_not_found() -> Self {
        Self::NotFound("Клієнта з таким ID не знайдено в базі даних".to_string())
    }

    pub(crate) fn client_archived() -> Self {
        Self::Generic("Клієнт архівований. Спочатку відновіть клієнта з архіву.".to_string())
    }

    pub(crate) fn car_not_found() -> Self {
        Self::NotFound("Автомобіля з таким ID не знайдено в базі даних".to_string())
    }

    pub(crate) fn car_archived() -> Self {
        Self::Generic("Автомобіль архівований. Спочатку відновіть його з архіву.".to_string())
    }

    pub(crate) fn service_not_found() -> Self {
        Self::NotFound("Сервіса з таким ID не знайдено в базі даних".to_string())
    }

    pub(crate) fn service_archived() -> Self {
        Self::Generic("Сервіс архівований. Спочатку відновіть його з архіву.".to_string())
    }

    pub(crate) fn work_order_not_found() -> Self {
        Self::NotFound("Замовлення з таким ID не знайдено в базі даних".to_string())
    }

    pub(crate) fn work_order_archived() -> Self {
        Self::Generic("Замовлення архівоване. Спочатку відновіть його з архіву.".to_string())
    }

    pub(crate) fn item_not_found() -> Self {
        Self::NotFound("Елемент з таким ID не знайдено в базі даних".to_string())
    }

    pub(crate) fn part_item_not_found() -> Self {
        Self::NotFound("Запчастини з таким ID не знайдено в базі даних".to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        log::error!("Database failure: {:?}", error.to_string());
        match error {
            sqlx::Error::RowNotFound => {
                AppError::NotFound("Помилка БД: Запис не знайдено в базі даних".to_string())
            }

            sqlx::Error::Database(db_err) => {
                // SQLite extended error codes; the primary code is `code & 0xff`.
                match db_err.code().as_deref().and_then(|code| code.parse::<i32>().ok()) {
                    // Primary codes 5 = SQLITE_BUSY, 6 = SQLITE_LOCKED
                    Some(code) if code & 0xff == 5 || code & 0xff == 6 => AppError::Busy(
                        "База даних тимчасово заблокована. Спробуйте ще раз.".to_string(),
                    ),
                    // 787 = SQLITE_CONSTRAINT_FOREIGNKEY
                    Some(787) => AppError::ForeignKeyViolation(
                        "Недійсний зв'язок між таблицями або спроба видалити запис, який використовується".to_string()
                    ),
                    // 2067 = SQLITE_CONSTRAINT_UNIQUE, 1555 = SQLITE_CONSTRAINT_PRIMARYKEY
                    Some(2067) | Some(1555) => AppError::UniqueViolation(
                        "Такий запис (або його унікальне поле) вже існує.".to_string()
                    ),
                    // 19 = SQLITE_CONSTRAINT base; catches CHECK / NOT NULL / trigger
                    // and other constraint codes not handled above.
                    Some(code) if code & 0xff == 19 => AppError::ConstraintViolation(
                        "Запис не відповідає обмеженням таблиці.".to_string(),
                    ),
                    _ => AppError::Sqlx(db_err.message().to_string()),
                }
            }

            _ => AppError::Sqlx(format!("Помилка БД: {}", error)),
        }
    }
}

impl From<rust_xlsxwriter::XlsxError> for AppError {
    fn from(error: rust_xlsxwriter::XlsxError) -> Self {
        AppError::Generic(format!("Помилка формування XLSX: {error}"))
    }
}

pub type AppResult<T> = Result<T, AppError>;
