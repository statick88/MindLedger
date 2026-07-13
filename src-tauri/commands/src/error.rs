use thiserror::Error;

#[derive(Debug, Error, serde::Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Accounting error: {0}")]
    Accounting(String),
    #[error("Diagnostics error: {0}")]
    Diagnostics(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<soft_gloria_domain::RepositoryError> for AppError {
    fn from(err: soft_gloria_domain::RepositoryError) -> Self {
        match err {
            soft_gloria_domain::RepositoryError::NotFound(msg) => AppError::NotFound(msg),
            soft_gloria_domain::RepositoryError::Constraint(msg) => AppError::Conflict(msg),
            _ => AppError::Database(err.to_string()),
        }
    }
}

impl From<soft_gloria_domain::DocumentNumberError> for AppError {
    fn from(err: soft_gloria_domain::DocumentNumberError) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<soft_gloria_domain::FullNameError> for AppError {
    fn from(err: soft_gloria_domain::FullNameError) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<soft_gloria_domain::EmailError> for AppError {
    fn from(err: soft_gloria_domain::EmailError) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<soft_gloria_domain::PhoneNumberError> for AppError {
    fn from(err: soft_gloria_domain::PhoneNumberError) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<soft_gloria_domain::AddressError> for AppError {
    fn from(err: soft_gloria_domain::AddressError) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<soft_gloria_domain::EmergencyContactError> for AppError {
    fn from(err: soft_gloria_domain::EmergencyContactError) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::Validation(format!("Invalid UUID: {}", err))
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::Validation(format!("Invalid date: {}", err))
    }
}

impl From<soft_gloria_domain::ContabilidadError> for AppError {
    fn from(err: soft_gloria_domain::ContabilidadError) -> Self {
        AppError::Accounting(err.to_string())
    }
}

impl From<soft_gloria_domain::DomainError> for AppError {
    fn from(err: soft_gloria_domain::DomainError) -> Self {
        AppError::Validation(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;