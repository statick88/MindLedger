use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::DomainError;
use crate::identifiers::{AppointmentId, PatientId, TherapistId};

/// Appointment status with explicit state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AppointmentStatus {
    Programada,
    Realizada,
    Reagendada,
    Cancelada,
}

impl fmt::Display for AppointmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppointmentStatus::Programada => write!(f, "Programada"),
            AppointmentStatus::Realizada => write!(f, "Realizada"),
            AppointmentStatus::Reagendada => write!(f, "Reagendada"),
            AppointmentStatus::Cancelada => write!(f, "Cancelada"),
        }
    }
}

impl AppointmentStatus {
    /// Returns valid next states from current state
    pub fn valid_transitions(&self) -> Vec<AppointmentStatus> {
        match self {
            AppointmentStatus::Programada => vec![
                AppointmentStatus::Realizada,
                AppointmentStatus::Reagendada,
                AppointmentStatus::Cancelada,
            ],
            AppointmentStatus::Reagendada => vec![
                AppointmentStatus::Realizada,
                AppointmentStatus::Cancelada,
            ],
            AppointmentStatus::Realizada => vec![], // Terminal
            AppointmentStatus::Cancelada => vec![], // Terminal
        }
    }

    /// Checks if transition is valid
    pub fn can_transition_to(&self, next: AppointmentStatus) -> bool {
        self.valid_transitions().contains(&next)
    }

    /// Returns true if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, AppointmentStatus::Realizada | AppointmentStatus::Cancelada)
    }
}

/// Session modality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Modality {
    Presencial,
    Virtual,
    Hibrida,
}

impl fmt::Display for Modality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modality::Presencial => write!(f, "Presencial"),
            Modality::Virtual => write!(f, "Virtual"),
            Modality::Hibrida => write!(f, "Híbrida"),
        }
    }
}

/// Time range with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateTimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateTimeRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, DomainError> {
        if start >= end {
            return Err(DomainError::Validation(
                "Start time must be before end time".to_string(),
            ));
        }
        let duration = end - start;
        let minutes = duration.num_minutes();
        if minutes < 15 || minutes > 120 {
            return Err(DomainError::InvalidDuration);
        }
        Ok(Self { start, end })
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }

    pub fn overlaps(&self, other: &DateTimeRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    pub fn contains(&self, datetime: DateTime<Utc>) -> bool {
        datetime >= self.start && datetime < self.end
    }
}

/// Appointment aggregate root
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Appointment {
    pub id: AppointmentId,
    pub patient_id: PatientId,
    pub therapist_id: TherapistId,
    pub time_range: DateTimeRange,
    pub modality: Modality,
    pub status: AppointmentStatus,
    pub fee_cents: i64,
    pub notes: Option<String>,
    pub reminder_sent: bool,
    pub reminder_external_id: Option<String>,
    pub reagendada_from_id: Option<AppointmentId>,
    pub external_calendar_id: Option<String>,
    pub calendar_provider: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Appointment {
    /// Creates a new appointment with validation
    pub fn new(
        patient_id: PatientId,
        therapist_id: TherapistId,
        time_range: DateTimeRange,
        modality: Modality,
        fee_cents: i64,
        notes: Option<String>,
    ) -> Result<Self, DomainError> {
        if fee_cents < 0 {
            return Err(DomainError::Validation("Fee cannot be negative".to_string()));
        }

        let now = Utc::now();
        Ok(Self {
            id: AppointmentId::new(),
            patient_id,
            therapist_id,
            time_range,
            modality,
            status: AppointmentStatus::Programada,
            fee_cents,
            notes,
            reminder_sent: false,
            reminder_external_id: None,
            reagendada_from_id: None,
            external_calendar_id: None,
            calendar_provider: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Transitions to Realizada (finalizada)
    /// Returns the accounting entry data needed for double-entry bookkeeping
    pub fn finalize(&mut self, final_notes: Option<String>) -> Result<AccountingEntryData, DomainError> {
        if !self.status.can_transition_to(AppointmentStatus::Realizada) {
            return Err(DomainError::InvalidTransition {
                from: self.status.to_string(),
                to: AppointmentStatus::Realizada.to_string(),
            });
        }

        self.status = AppointmentStatus::Realizada;
        self.updated_at = Utc::now();
        if let Some(notes) = final_notes {
            self.notes = Some(notes);
        }

        // Return data needed for accounting trigger
        Ok(AccountingEntryData {
            appointment_id: self.id,
            patient_id: self.patient_id,
            therapist_id: self.therapist_id,
            date: self.time_range.start.date_naive(),
            amount_cents: self.fee_cents,
            description: format!(
                "Sesión: {} - {} - {}",
                self.patient_name(), // Will be filled by caller
                self.time_range.start.format("%d/%m/%Y"),
                self.modality
            ),
        })
    }

    /// Transitions to Reagendada (rescheduled)
    pub fn reschedule(&mut self, new_time_range: DateTimeRange, reason: String) -> Result<(), DomainError> {
        if !self.status.can_transition_to(AppointmentStatus::Reagendada) {
            return Err(DomainError::InvalidTransition {
                from: self.status.to_string(),
                to: AppointmentStatus::Reagendada.to_string(),
            });
        }

        self.time_range = new_time_range;
        self.status = AppointmentStatus::Reagendada;
        self.updated_at = Utc::now();
        
        if let Some(existing) = &mut self.notes {
            existing.push_str(&format!("\n[Reagendada] {}: {}", Utc::now().format("%d/%m/%Y %H:%M"), reason));
        } else {
            self.notes = Some(format!("[Reagendada] {}: {}", Utc::now().format("%d/%m/%Y %H:%M"), reason));
        }

        Ok(())
    }

    /// Transitions to Cancelada
    pub fn cancel(&mut self, reason: String) -> Result<(), DomainError> {
        if !self.status.can_transition_to(AppointmentStatus::Cancelada) {
            return Err(DomainError::InvalidTransition {
                from: self.status.to_string(),
                to: AppointmentStatus::Cancelada.to_string(),
            });
        }

        if reason.trim().is_empty() {
            return Err(DomainError::Validation("Cancellation reason is required".to_string()));
        }

        self.status = AppointmentStatus::Cancelada;
        self.updated_at = Utc::now();
        
        if let Some(existing) = &mut self.notes {
            existing.push_str(&format!("\n[Cancelada] {}: {}", Utc::now().format("%d/%m/%Y %H:%M"), reason));
        } else {
            self.notes = Some(format!("[Cancelada] {}: {}", Utc::now().format("%d/%m/%Y %H:%M"), reason));
        }

        Ok(())
    }

    /// Marks reminder as sent
    pub fn mark_reminder_sent(&mut self, external_id: String) {
        self.reminder_sent = true;
        self.reminder_external_id = Some(external_id);
        self.updated_at = Utc::now();
    }

    /// Clears reminder (for reschedule/cancel)
    pub fn clear_reminder(&mut self) {
        self.reminder_sent = false;
        self.reminder_external_id = None;
        self.updated_at = Utc::now();
    }

    /// Sets external calendar sync info
    pub fn set_calendar_info(&mut self, external_id: String, provider: String) {
        self.external_calendar_id = Some(external_id);
        self.calendar_provider = Some(provider);
        self.updated_at = Utc::now();
    }

    /// Returns patient name placeholder - actual name filled by command layer
    fn patient_name(&self) -> String {
        format!("Paciente {}", self.patient_id)
    }

    /// Checks if appointment is within the 30-minute reminder window
    pub fn is_in_reminder_window(&self, now: DateTime<Utc>) -> bool {
        if self.reminder_sent || self.status.is_terminal() {
            return false;
        }
        let reminder_time = self.time_range.start - Duration::minutes(30);
        now >= reminder_time && now < self.time_range.start
    }

    /// Checks if appointment overlaps with another for the same therapist
    pub fn overlaps_with(&self, other: &Appointment) -> bool {
        self.therapist_id == other.therapist_id && self.time_range.overlaps(&other.time_range)
    }
}

/// Data needed to create accounting entry from appointment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingEntryData {
    pub appointment_id: AppointmentId,
    pub patient_id: PatientId,
    pub therapist_id: TherapistId,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub description: String,
}

impl AccountingEntryData {
    /// Builds the double-entry asiento lines
    /// Debit: 1.1.1.01 Caja/Banco (or 1.1.2.01 Cuentas por Cobrar)
    /// Credit: 4.1.1.01 Honorarios Profesionales
    pub fn to_asiento_lines(&self, is_paid: bool) -> (Vec<AsientoLine>, Vec<AsientoLine>) {
        let debit_account = if is_paid {
            "1.1.1.01" // Caja/Banco
        } else {
            "1.1.2.01" // Cuentas por Cobrar
        };
        let credit_account = "4.1.1.01"; // Honorarios Profesionales
        
        let amount = Decimal::from(self.amount_cents) / Decimal::new(100, 2);

        let debit_line = AsientoLine {
            account: debit_account.to_string(),
            name: if is_paid { "Caja/Banco" } else { "Cuentas por Cobrar" }.to_string(),
            debit: amount,
            credit: Decimal::ZERO,
        };

        let credit_line = AsientoLine {
            account: credit_account.to_string(),
            name: "Honorarios Profesionales".to_string(),
            debit: Decimal::ZERO,
            credit: amount,
        };

        (vec![debit_line], vec![credit_line])
    }
}

/// Simplified asiento line for domain layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsientoLine {
    pub account: String,
    pub name: String,
    pub debit: Decimal,
    pub credit: Decimal,
}

/// Domain event emitted when appointment is finalized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFinalizedEvent {
    pub appointment_id: AppointmentId,
    pub patient_id: PatientId,
    pub therapist_id: TherapistId,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub accounting_entry: AccountingEntryData,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::{PatientId, TherapistId};
    use chrono::TimeZone;

    fn create_test_appointment() -> Appointment {
        let start = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let end = start + Duration::minutes(50);
        let time_range = DateTimeRange::new(start, end).unwrap();
        
        Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range,
            Modality::Presencial,
            5000, // $50.00
            Some("Primera sesión".to_string()),
        ).unwrap()
    }

    #[test]
    fn test_appointment_creation_valid() {
        let appt = create_test_appointment();
        assert_eq!(appt.status, AppointmentStatus::Programada);
        assert_eq!(appt.fee_cents, 5000);
        assert!(!appt.id.is_nil());
    }

    #[test]
    fn test_appointment_creation_invalid_duration() {
        let start = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let end = start + Duration::minutes(10); // Too short
        let time_range = DateTimeRange::new(start, end);
        assert!(matches!(time_range, Err(DomainError::InvalidDuration)));
    }

    #[test]
    fn test_appointment_creation_negative_fee() {
        let start = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let end = start + Duration::minutes(50);
        let time_range = DateTimeRange::new(start, end).unwrap();
        
        let result = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range,
            Modality::Presencial,
            -100,
            None,
        );
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_finalize_appointment() {
        let mut appt = create_test_appointment();
        let accounting_data = appt.finalize(Some("Sesión completada".to_string())).unwrap();
        
        assert_eq!(appt.status, AppointmentStatus::Realizada);
        assert_eq!(accounting_data.amount_cents, 5000);
        assert!(accounting_data.description.contains("Sesión"));
    }

    #[test]
    fn test_finalize_from_invalid_state() {
        let mut appt = create_test_appointment();
        appt.status = AppointmentStatus::Cancelada;
        
        let result = appt.finalize(None);
        assert!(matches!(result, Err(DomainError::InvalidTransition { .. })));
    }

    #[test]
    fn test_reschedule_appointment() {
        let mut appt = create_test_appointment();
        let new_start = Utc.with_ymd_and_hms(2025, 1, 16, 10, 0, 0).single().unwrap();
        let new_end = new_start + Duration::minutes(50);
        let new_range = DateTimeRange::new(new_start, new_end).unwrap();
        
        appt.reschedule(new_range, "Paciente solicitó cambio".to_string()).unwrap();
        
        assert_eq!(appt.status, AppointmentStatus::Reagendada);
        // ID is preserved (no new ID generated)
    }

    #[test]
    fn test_reschedule_from_invalid_state() {
        let mut appt = create_test_appointment();
        appt.status = AppointmentStatus::Realizada;
        
        let new_start = Utc.with_ymd_and_hms(2025, 1, 16, 10, 0, 0).single().unwrap();
        let new_end = new_start + Duration::minutes(50);
        let new_range = DateTimeRange::new(new_start, new_end).unwrap();
        
        let result = appt.reschedule(new_range, "Test".to_string());
        assert!(matches!(result, Err(DomainError::InvalidTransition { .. })));
    }

    #[test]
    fn test_cancel_appointment() {
        let mut appt = create_test_appointment();
        appt.cancel("Paciente no asistirá".to_string()).unwrap();
        
        assert_eq!(appt.status, AppointmentStatus::Cancelada);
        assert!(appt.notes.as_ref().unwrap().contains("Cancelada"));
    }

    #[test]
    fn test_cancel_requires_reason() {
        let mut appt = create_test_appointment();
        let result = appt.cancel("".to_string());
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_state_machine_transitions() {
        // Programada can go to Realizada, Reagendada, Cancelada
        assert!(AppointmentStatus::Programada.can_transition_to(AppointmentStatus::Realizada));
        assert!(AppointmentStatus::Programada.can_transition_to(AppointmentStatus::Reagendada));
        assert!(AppointmentStatus::Programada.can_transition_to(AppointmentStatus::Cancelada));
        
        // Reagendada can go to Realizada, Cancelada
        assert!(AppointmentStatus::Reagendada.can_transition_to(AppointmentStatus::Realizada));
        assert!(AppointmentStatus::Reagendada.can_transition_to(AppointmentStatus::Cancelada));
        assert!(!AppointmentStatus::Reagendada.can_transition_to(AppointmentStatus::Programada));
        
        // Realizada is terminal
        assert!(!AppointmentStatus::Realizada.can_transition_to(AppointmentStatus::Programada));
        assert!(!AppointmentStatus::Realizada.can_transition_to(AppointmentStatus::Cancelada));
        assert!(AppointmentStatus::Realizada.is_terminal());
        
        // Cancelada is terminal
        assert!(!AppointmentStatus::Cancelada.can_transition_to(AppointmentStatus::Programada));
        assert!(AppointmentStatus::Cancelada.is_terminal());
    }

    #[test]
    fn test_overlap_detection() {
        let start = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let end = start + Duration::minutes(50);
        let time_range = DateTimeRange::new(start, end).unwrap();
        
        let appt1 = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range.clone(),
            Modality::Presencial,
            5000,
            None,
        ).unwrap();
        
        // Overlapping appointment for same therapist
        let appt2 = Appointment::new(
            PatientId::new(),
            appt1.therapist_id,
            time_range.clone(),
            Modality::Presencial,
            5000,
            None,
        ).unwrap();
        
        assert!(appt1.overlaps_with(&appt2));
        
        // Different therapist - no overlap
        let appt3 = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range,
            Modality::Presencial,
            5000,
            None,
        ).unwrap();
        
        assert!(!appt1.overlaps_with(&appt3));
    }

    #[test]
    fn test_reminder_window() {
        let mut appt = create_test_appointment();
        
        // Before reminder window
        let before = appt.time_range.start - Duration::minutes(60);
        assert!(!appt.is_in_reminder_window(before));
        
        // In reminder window (30 min before)
        let in_window = appt.time_range.start - Duration::minutes(25);
        assert!(appt.is_in_reminder_window(in_window));
        
        // After appointment started
        let after = appt.time_range.start + Duration::minutes(10);
        assert!(!appt.is_in_reminder_window(after));
        
        // Already sent
        appt.mark_reminder_sent("ext_123".to_string());
        assert!(!appt.is_in_reminder_window(in_window));
        
        // Terminal state
        let mut appt2 = create_test_appointment();
        appt2.status = AppointmentStatus::Realizada;
        assert!(!appt2.is_in_reminder_window(in_window));
    }

    #[test]
    fn test_datetime_range_overlaps() {
        let start1 = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let end1 = start1 + Duration::minutes(50);
        let range1 = DateTimeRange::new(start1, end1).unwrap();
        
        // Overlapping
        let start2 = start1 + Duration::minutes(30);
        let end2 = start2 + Duration::minutes(50);
        let range2 = DateTimeRange::new(start2, end2).unwrap();
        assert!(range1.overlaps(&range2));
        
        // Non-overlapping (adjacent)
        let start3 = end1;
        let end3 = start3 + Duration::minutes(50);
        let range3 = DateTimeRange::new(start3, end3).unwrap();
        assert!(!range1.overlaps(&range3));
        
        // Non-overlapping (separate)
        let start4 = end1 + Duration::hours(1);
        let end4 = start4 + Duration::minutes(50);
        let range4 = DateTimeRange::new(start4, end4).unwrap();
        assert!(!range1.overlaps(&range4));
    }

    #[test]
    fn test_accounting_entry_data_lines() {
        let data = AccountingEntryData {
            appointment_id: AppointmentId::new(),
            patient_id: PatientId::new(),
            therapist_id: TherapistId::new(),
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount_cents: 5000,
            description: "Test".to_string(),
        };
        
        // Paid session
        let (debits, credits) = data.to_asiento_lines(true);
        assert_eq!(debits.len(), 1);
        assert_eq!(credits.len(), 1);
        assert_eq!(debits[0].account, "1.1.1.01");
        assert_eq!(credits[0].account, "4.1.1.01");
        assert_eq!(debits[0].debit, credits[0].credit);
        
        // Unpaid session (cuentas por cobrar)
        let (debits2, credits2) = data.to_asiento_lines(false);
        assert_eq!(debits2[0].account, "1.1.2.01");
    }

    #[test]
    fn test_modality_display() {
        assert_eq!(Modality::Presencial.to_string(), "Presencial");
        assert_eq!(Modality::Virtual.to_string(), "Virtual");
        assert_eq!(Modality::Hibrida.to_string(), "Híbrida");
    }
}