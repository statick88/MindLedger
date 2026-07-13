use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::accounting::ContabilidadError;

/// Domain-level errors for MindLedger business logic
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum DomainError {
    #[error("Invalid state transition: from {from} to {to}")]
    InvalidTransition {
        from: String,
        to: String,
    },

    #[error("Appointment overlaps with existing appointment for same therapist")]
    OverlapConflict,

    #[error("Appointment duration must be between 15 and 120 minutes")]
    InvalidDuration,

    #[error("Patient not found: {0}")]
    PatientNotFound(String),

    #[error("Therapist not found: {0}")]
    TherapistNotFound(String),

    #[error("Appointment not found: {0}")]
    AppointmentNotFound(String),

    #[error("Calendar error: {0}")]
    CalendarError(String),

    #[error("Reminder error: {0}")]
    ReminderError(String),

    #[error("Accounting error: {0}")]
    AccountingError(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Concurrency conflict: entity was modified by another process")]
    ConcurrencyConflict,

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl From<ContabilidadError> for DomainError {
    fn from(err: ContabilidadError) -> Self {
        DomainError::AccountingError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        let err = DomainError::InvalidTransition {
            from: "Programada".to_string(),
            to: "Cancelada".to_string(),
        };
        assert!(err.to_string().contains("Programada"));
        assert!(err.to_string().contains("Cancelada"));
    }

    #[test]
    fn test_domain_error_serialization() {
        let err = DomainError::OverlapConflict;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("OverlapConflict"));
    }

    #[test]
    fn test_contabilidad_error_conversion() {
        let contab_err = ContabilidadError::AsientoVacio;
        let domain_err: DomainError = contab_err.into();
        assert!(matches!(domain_err, DomainError::AccountingError(_)));
    }
}