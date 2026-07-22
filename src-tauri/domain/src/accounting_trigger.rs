use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::identifiers::{AppointmentId, PatientId, TherapistId};
use crate::value_objects::FullName;

/// Account codes used in the Ecuadorian chart of accounts (simplified)
pub const CUENTA_CAJA: &str = "1.1.1.01";           // Caja/Banco
pub const CUENTA_CUENTAS_POR_COBRAR: &str = "1.1.2.01"; // Cuentas por Cobrar
pub const CUENTA_HONORARIOS: &str = "4.1.1.01";     // Honorarios Profesionales

/// Patient data needed for accounting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientAccountingData {
    pub id: PatientId,
    pub full_name: FullName,
    pub session_fee_cents: i64,
}

/// Therapist data needed for accounting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TherapistAccountingData {
    pub id: TherapistId,
    pub full_name: FullName,
    pub specialty_code: String,
}

/// Session accounting entry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAccountingEntry {
    pub appointment_id: AppointmentId,
    pub debit_account: String,
    pub credit_account: String,
    pub amount_cents: i64,
    pub description: String,
}

/// Accounting trigger helper - creates balanced double-entry asientos from sessions
pub struct AccountingTrigger;

impl AccountingTrigger {
    /// Builds a double-entry asiento for a completed session
    /// 
    /// # Arguments
    /// * `appointment_id` - Unique appointment identifier
    /// * `patient` - Patient data (for description)
    /// * `therapist` - Therapist data (for description)
    /// * `date` - Session date
    /// * `amount_cents` - Session fee in centavos
    /// * `is_paid` - Whether payment was received at session
    /// * `modality` - Session modality for description
    /// 
    /// # Returns
    /// Balanced `AsientoContable` ready for persistence
    pub fn build_session_asiento(
        _appointment_id: AppointmentId,
        patient: &PatientAccountingData,
        therapist: &TherapistAccountingData,
        date: NaiveDate,
        amount_cents: i64,
        is_paid: bool,
        modality: &str,
    ) -> Result<crate::accounting::AsientoContable, DomainError> {
        if amount_cents <= 0 {
            return Err(DomainError::Validation("Amount must be positive".to_string()));
        }

        let description = format!(
            "Sesión: {} - {} - {} - {}",
            patient.full_name.full_name(),
            date.format("%d/%m/%Y"),
            therapist.full_name.full_name(),
            modality
        );

        let amount = Decimal::from(amount_cents) / Decimal::from(100);

        // Debit: Caja/Banco (paid) or Cuentas por Cobrar (unpaid)
        let debit_account = if is_paid {
            CUENTA_CAJA
        } else {
            CUENTA_CUENTAS_POR_COBRAR
        };

        let debit_line = crate::accounting::LineaAsiento::new_debito(
            debit_account.to_string(),
            amount,
        )?;

        // Credit: Honorarios Profesionales
        let credit_line = crate::accounting::LineaAsiento::new_credito(
            CUENTA_HONORARIOS.to_string(),
            amount,
        )?;

        let asiento = crate::accounting::AsientoContable::new(
            date,
            description,
            vec![debit_line, credit_line],
        )?;

        // Double-check balance
        if !asiento.is_balanced() {
            return Err(DomainError::AccountingError(
                "Generated asiento is not balanced".to_string()
            ));
        }

        Ok(asiento)
    }

    /// Validates that an asiento is balanced
    pub fn validate_asiento_balance(
        asiento: &crate::accounting::AsientoContable,
    ) -> Result<(), DomainError> {
        if asiento.is_balanced() {
            Ok(())
        } else {
            Err(DomainError::AccountingError(
                format!("Asiento {} is not balanced: Debitos={}, Creditos={}", 
                    asiento.id, asiento.total_debitos(), asiento.total_creditos())
            ))
        }
    }

    /// Builds a batch asiento from multiple sessions
    /// All entries must balance individually and in aggregate
    pub fn build_batch_asiento(
        entries: Vec<SessionAccountingEntry>,
        date: NaiveDate,
        description: String,
    ) -> Result<crate::accounting::AsientoContable, DomainError> {
        if entries.is_empty() {
            return Err(DomainError::Validation("No entries provided".to_string()));
        }

        let mut debit_lines = Vec::new();
        let mut credit_lines = Vec::new();

        for entry in entries {
            let amount = Decimal::from(entry.amount_cents) / Decimal::from(100);
            
            debit_lines.push(crate::accounting::LineaAsiento::new_debito(
                entry.debit_account,
                amount,
            ).map_err(|e| DomainError::AccountingError(e.to_string()))?);

            credit_lines.push(crate::accounting::LineaAsiento::new_credito(
                entry.credit_account,
                amount,
            ).map_err(|e| DomainError::AccountingError(e.to_string()))?);
        }

        let mut all_lines = debit_lines;
        all_lines.extend(credit_lines);

        crate::accounting::AsientoContable::new(date, description, all_lines)
            .map_err(|e| DomainError::AccountingError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::AsientoContable;
    use crate::identifiers::{AppointmentId, PatientId, TherapistId};
    use crate::value_objects::FullName;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn sample_patient() -> PatientAccountingData {
        PatientAccountingData {
            id: PatientId::new(),
            full_name: FullName::new("Juan".to_string(), "Pérez".to_string(), Some("Carlos".to_string())).unwrap(),
            session_fee_cents: 50000,
        }
    }

    fn sample_therapist() -> TherapistAccountingData {
        TherapistAccountingData {
            id: TherapistId::new(),
            full_name: FullName::new("Dra. Ana".to_string(), "García".to_string(), None).unwrap(),
            specialty_code: "PSI".to_string(),
        }
    }

    #[test]
    fn test_build_session_asiento_paid() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let asiento = AccountingTrigger::build_session_asiento(
            AppointmentId::new(),
            &sample_patient(),
            &sample_therapist(),
            date,
            50000, // $500.00 in cents
            true,  // paid
            "Presencial",
        ).unwrap();

        assert_eq!(asiento.fecha, date);
        assert_eq!(asiento.lineas.len(), 2);
        
        // Check debit line (Caja)
        let debit = asiento.lineas.iter().find(|l| l.is_debito()).unwrap();
        assert_eq!(debit.cuenta, "1.1.1.01");
        // 50000 cents = 500.00
        assert_eq!(debit.debito, dec!(500));  // Decimal with scale 0 is OK
        assert_eq!(debit.credito, Decimal::ZERO);
        
        // Check credit line (Honorarios)
        let credit = asiento.lineas.iter().find(|l| l.is_credito()).unwrap();
        assert_eq!(credit.cuenta, "4.1.1.01");
        assert_eq!(credit.credito, dec!(500));
        assert_eq!(credit.debito, Decimal::ZERO);
        
        // Balanced
        assert!(asiento.is_balanced());
        assert_eq!(asiento.total_debitos(), asiento.total_creditos());
    }

    #[test]
    fn test_build_session_asiento_unpaid() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let asiento = AccountingTrigger::build_session_asiento(
            AppointmentId::new(),
            &sample_patient(),
            &sample_therapist(),
            date,
            50000,
            false, // unpaid - cuentas por cobrar
            "Virtual",
        ).unwrap();

        // Check debit line (Cuentas por Cobrar)
        let debit = asiento.lineas.iter().find(|l| l.is_debito()).unwrap();
        assert_eq!(debit.cuenta, "1.1.2.01");
        
        // Balanced
        assert!(asiento.is_balanced());
    }

    #[test]
    fn test_build_session_asiento_zero_amount_fails() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let result = AccountingTrigger::build_session_asiento(
            AppointmentId::new(),
            &sample_patient(),
            &sample_therapist(),
            date,
            0,
            true,
            "Presencial",
        );
        
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_build_session_asiento_negative_fails() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let result = AccountingTrigger::build_session_asiento(
            AppointmentId::new(),
            &sample_patient(),
            &sample_therapist(),
            date,
            -100,
            true,
            "Presencial",
        );
        
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_validate_asiento_balance() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let asiento = AccountingTrigger::build_session_asiento(
            AppointmentId::new(),
            &sample_patient(),
            &sample_therapist(),
            date,
            50000,
            true,
            "Presencial",
        ).unwrap();

        assert!(AccountingTrigger::validate_asiento_balance(&asiento).is_ok());
    }

    #[test]
    fn test_build_batch_asiento() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let entries = vec![
            SessionAccountingEntry {
                appointment_id: AppointmentId::new(),
                debit_account: CUENTA_CAJA.to_string(),
                credit_account: CUENTA_HONORARIOS.to_string(),
                amount_cents: 50000,
                description: "Session 1".to_string(),
            },
            SessionAccountingEntry {
                appointment_id: AppointmentId::new(),
                debit_account: CUENTA_CUENTAS_POR_COBRAR.to_string(),
                credit_account: CUENTA_HONORARIOS.to_string(),
                amount_cents: 30000,
                description: "Session 2".to_string(),
            },
        ];

        let asiento = AccountingTrigger::build_batch_asiento(entries, date, "Lote de sesiones".to_string()).unwrap();
        
        assert_eq!(asiento.lineas.len(), 4); // 2 debit + 2 credit
        assert!(asiento.is_balanced());
        assert_eq!(asiento.total_debitos(), dec!(800));
        assert_eq!(asiento.total_creditos(), dec!(800));
    }

    #[test]
    fn test_build_batch_asiento_empty_fails() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let result = AccountingTrigger::build_batch_asiento(vec![], date, "Empty".to_string());
        
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_account_codes_constants() {
        assert_eq!(CUENTA_CAJA, "1.1.1.01");
        assert_eq!(CUENTA_CUENTAS_POR_COBRAR, "1.1.2.01");
        assert_eq!(CUENTA_HONORARIOS, "4.1.1.01");
    }

    #[test]
    fn test_asiento_description_format() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        
        let patient = PatientAccountingData {
            id: PatientId::new(),
            full_name: FullName::new("María".to_string(), "González".to_string(), None).unwrap(),
            session_fee_cents: 50000,
        };
        
        let therapist = TherapistAccountingData {
            id: TherapistId::new(),
            full_name: FullName::new("Dr. Carlos".to_string(), "Rodríguez".to_string(), None).unwrap(),
            specialty_code: "PSI".to_string(),
        };
        
        let asiento = AccountingTrigger::build_session_asiento(
            AppointmentId::new(),
            &patient,
            &therapist,
            date,
            50000,
            true,
            "Presencial",
        ).unwrap();

        assert!(asiento.descripcion.contains("María González"));
        assert!(asiento.descripcion.contains("15/01/2025"));
        assert!(asiento.descripcion.contains("Dr. Carlos Rodríguez"));
        assert!(asiento.descripcion.contains("Presencial"));
    }
}