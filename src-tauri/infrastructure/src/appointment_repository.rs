use crate::database::DbPool;
use soft_mindledger_domain::{
    Appointment, AppointmentId, PatientId, TherapistId, AppointmentStatus, Modality,
    DateRange, DateTimeRange, Pagination, AppointmentFilter,
    AppointmentRepository,
    RepositoryError,
};
use rusqlite::params;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use async_trait::async_trait;

pub struct SqliteAppointmentRepository {
    pool: DbPool,
}

impl SqliteAppointmentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

fn row_to_appointment(row: &rusqlite::Row) -> rusqlite::Result<Appointment> {
        let id: String = row.get("id")?;
        let patient_id: String = row.get("patient_id")?;
        let therapist_id: String = row.get("professional_id")?; // DB column is professional_id
        let status: String = row.get("status")?;

        // Parse scheduled date and time
        let scheduled_date: String = row.get("scheduled_date")?;
        let scheduled_time: String = row.get("scheduled_time")?;
        let scheduled_datetime = NaiveDate::parse_from_str(&scheduled_date, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid date format: {}", e)))?
            .and_hms_opt(
                scheduled_time[0..2].parse().unwrap_or(0),
                scheduled_time[3..5].parse().unwrap_or(0),
                0
            ).unwrap_or_default()
            .and_utc();

        let duration_minutes: i64 = row.get("duration_minutes")?;

        let modality: String = row.get("modality")?;
        let fee_cents: i64 = row.get("fee_cents")?;
        let notes: Option<String> = row.get("notes")?;
        let reminder_sent: i64 = row.get("reminder_sent")?;
        let reminder_external_id: Option<String> = row.get("reminder_external_id")?;
        let reagendada_from_id: Option<String> = row.get("reagendada_from_id")?;
        let external_calendar_id: Option<String> = row.get("external_calendar_id")?;
        let calendar_provider: Option<String> = row.get("calendar_provider")?;
        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;

        let end_datetime = scheduled_datetime + chrono::Duration::minutes(duration_minutes);

        Ok(Appointment {
            id: AppointmentId(Uuid::parse_str(&id).unwrap()),
            patient_id: PatientId(Uuid::parse_str(&patient_id).unwrap()),
            therapist_id: TherapistId(Uuid::parse_str(&therapist_id).unwrap()), // Map DB professional_id to domain therapist_id
            time_range: DateTimeRange::new(
                scheduled_datetime,
                end_datetime,
            ).unwrap(),
            modality: match modality.as_str() {
                "Presencial" => Modality::Presencial,
                "Virtual" => Modality::Virtual,
                "Hibrida" => Modality::Hibrida,
                _ => Modality::Presencial,
            },
            status: match status.as_str() {
                "Programada" => AppointmentStatus::Programada,
                "Realizada" => AppointmentStatus::Realizada,
                "Reagendada" => AppointmentStatus::Reagendada,
                "Cancelada" => AppointmentStatus::Cancelada,
                _ => AppointmentStatus::Programada,
            },
            fee_cents,
            notes,
            reminder_sent: reminder_sent != 0,
            reminder_external_id,
            reagendada_from_id: reagendada_from_id.map(|s| AppointmentId(Uuid::parse_str(&s).unwrap())),
            external_calendar_id,
            calendar_provider,
            created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().into(),
            updated_at: DateTime::parse_from_rfc3339(&updated_at).unwrap_or_default().into(),
        })
    }
}

#[async_trait]
impl AppointmentRepository for SqliteAppointmentRepository {
    async fn create(&self, appointment: &Appointment) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let appointment = appointment.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            
            let status_str = match appointment.status {
                AppointmentStatus::Programada => "Programada",
                AppointmentStatus::Realizada => "Realizada",
                AppointmentStatus::Reagendada => "Reagendada",
                AppointmentStatus::Cancelada => "Cancelada",
            };

            let modality_str = match appointment.modality {
                Modality::Presencial => "Presencial",
                Modality::Virtual => "Virtual",
                Modality::Hibrida => "Hibrida",
            };

            let scheduled_date = appointment.time_range.start.format("%Y-%m-%d").to_string();
            let scheduled_time = appointment.time_range.start.format("%H:%M").to_string();
            let duration_minutes = (appointment.time_range.end - appointment.time_range.start).num_minutes();
            
            let notes = appointment.notes.clone();
            let reminder_sent = if appointment.reminder_sent { 1 } else { 0 };
            let reminder_external_id = appointment.reminder_external_id.clone();
            let reagendada_from_id = appointment.reagendada_from_id.map(|id| id.to_string());
            let external_calendar_id = appointment.external_calendar_id.clone();
            let calendar_provider = appointment.calendar_provider.clone();
            
            let now = Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO appointments (
                    id, patient_id, professional_id, status, scheduled_date, scheduled_time,
                    duration_minutes, modality, fee_cents, notes,
                    reminder_sent, reminder_external_id, reagendada_from_id,
                    external_calendar_id, calendar_provider,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    appointment.id.to_string(),
                    appointment.patient_id.to_string(),
                    appointment.therapist_id.to_string(),  // Map domain therapist_id to DB professional_id
                    status_str,
                    scheduled_date,
                    scheduled_time,
                    duration_minutes,
                    modality_str,
                    appointment.fee_cents,
                    notes,
                    reminder_sent,
                    reminder_external_id,
                    reagendada_from_id,
                    external_calendar_id,
                    calendar_provider,
                    now,
                    now,
                ],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_by_id(&self, id: AppointmentId) -> Result<Option<Appointment>, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare("SELECT * FROM appointments WHERE id = ?1")
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut rows = stmt.query(params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| RepositoryError::Database(e.to_string()))? {
                let appointment = Self::row_to_appointment(row).map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(Some(appointment))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn list(&self, filter: AppointmentFilter, pagination: Pagination) -> Result<Vec<Appointment>, RepositoryError> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            
            let mut where_clauses = vec!["1=1".to_string()];
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(status) = filter.status {
                where_clauses.push(format!("status = ?{}", param_values.len() + 1));
                param_values.push(Box::new(status.to_string()));
            }
            if let Some(patient_id) = filter.patient_id {
                where_clauses.push(format!("patient_id = ?{}", param_values.len() + 1));
                param_values.push(Box::new(patient_id.to_string()));
            }
            if let Some(therapist_id) = filter.therapist_id {
                where_clauses.push(format!("professional_id = ?{}", param_values.len() + 1));
                param_values.push(Box::new(therapist_id.to_string()));
            }
            if let Some(date_range) = filter.date_range {
                where_clauses.push(format!("scheduled_date >= ?{}", param_values.len() + 1));
                param_values.push(Box::new(date_range.start.format("%Y-%m-%d").to_string()));
                where_clauses.push(format!("scheduled_date <= ?{}", param_values.len() + 1));
                param_values.push(Box::new(date_range.end.format("%Y-%m-%d").to_string()));
            }

            let limit_idx = param_values.len() + 1;
            let offset_idx = param_values.len() + 2;
            param_values.push(Box::new(pagination.limit as i64));
            param_values.push(Box::new(pagination.offset as i64));

            let sql = format!(
                "SELECT * FROM appointments WHERE {} ORDER BY scheduled_date, scheduled_time LIMIT ?{} OFFSET ?{}",
                where_clauses.join(" AND "), limit_idx, offset_idx
            );

            let mut stmt = conn.prepare(&sql).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| Self::row_to_appointment(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut appointments = Vec::new();
            for row in rows {
                appointments.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(appointments)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn count(&self, filter: AppointmentFilter) -> Result<u64, RepositoryError> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            
            let mut where_clauses = vec!["1=1".to_string()];
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(status) = filter.status {
                where_clauses.push(format!("status = ?{}", param_values.len() + 1));
                param_values.push(Box::new(status.to_string()));
            }
            if let Some(patient_id) = filter.patient_id {
                where_clauses.push(format!("patient_id = ?{}", param_values.len() + 1));
                param_values.push(Box::new(patient_id.to_string()));
            }
            if let Some(therapist_id) = filter.therapist_id {
                where_clauses.push(format!("professional_id = ?{}", param_values.len() + 1));
                param_values.push(Box::new(therapist_id.to_string()));
            }
            if let Some(date_range) = filter.date_range {
                where_clauses.push(format!("scheduled_date >= ?{}", param_values.len() + 1));
                param_values.push(Box::new(date_range.start.format("%Y-%m-%d").to_string()));
                where_clauses.push(format!("scheduled_date <= ?{}", param_values.len() + 1));
                param_values.push(Box::new(date_range.end.format("%Y-%m-%d").to_string()));
            }

            let where_sql = if where_clauses.is_empty() {
                "1=1".to_string()
            } else {
                where_clauses.join(" AND ")
            };

            let sql = format!("SELECT COUNT(*) FROM appointments WHERE {}", where_sql);

            let mut stmt = conn.prepare(&sql).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(count as u64)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn update(&self, appointment: &Appointment) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let appointment = appointment.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            
            let status_str = match appointment.status {
                AppointmentStatus::Programada => "Programada",
                AppointmentStatus::Realizada => "Realizada",
                AppointmentStatus::Reagendada => "Reagendada",
                AppointmentStatus::Cancelada => "Cancelada",
            };

            let modality_str = match appointment.modality {
                Modality::Presencial => "Presencial",
                Modality::Virtual => "Virtual",
                Modality::Hibrida => "Hibrida",
            };

            let scheduled_date = appointment.time_range.start.format("%Y-%m-%d").to_string();
            let scheduled_time = appointment.time_range.start.format("%H:%M").to_string();
            let duration_minutes = (appointment.time_range.end - appointment.time_range.start).num_minutes();

            let notes = appointment.notes.clone();
            let reminder_sent = if appointment.reminder_sent { 1 } else { 0 };
            let reminder_external_id = appointment.reminder_external_id.clone();
            let reagendada_from_id = appointment.reagendada_from_id.map(|id| id.to_string());
            let external_calendar_id = appointment.external_calendar_id.clone();
            let calendar_provider = appointment.calendar_provider.clone();

            let now = Utc::now().to_rfc3339();

            let affected = conn.execute(
                "UPDATE appointments SET
                    status = ?1, scheduled_date = ?2, scheduled_time = ?3,
                    duration_minutes = ?4, modality = ?5, fee_cents = ?6, notes = ?7,
                    reminder_sent = ?8, reminder_external_id = ?9, reagendada_from_id = ?10,
                    external_calendar_id = ?11, calendar_provider = ?12,
                    updated_at = ?13
                WHERE id = ?14",
                params![
                    status_str,
                    scheduled_date,
                    scheduled_time,
                    duration_minutes,
                    modality_str,
                    appointment.fee_cents,
                    notes,
                    reminder_sent,
                    reminder_external_id,
                    reagendada_from_id,
                    external_calendar_id,
                    calendar_provider,
                    now,
                    appointment.id.to_string(),
                ],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            if affected == 0 {
                return Err(RepositoryError::NotFound(format!("Appointment not found: {}", appointment.id)));
            }
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn delete(&self, id: AppointmentId) -> Result<bool, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let affected = conn.execute("DELETE FROM appointments WHERE id = ?1", params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(affected > 0)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn find_overlapping(&self, therapist_id: TherapistId, range: DateRange) -> Result<Vec<Appointment>, RepositoryError> {
        let pool = self.pool.clone();
        let therapist_str = therapist_id.to_string();
        let start_str = range.start.format("%Y-%m-%d %H:%M:%S").to_string();
        let end_str = range.end.format("%Y-%m-%d %H:%M:%S").to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT * FROM appointments 
                 WHERE professional_id = ?1 
                 AND datetime(scheduled_date || ' ' || scheduled_time) < ?2 
                 AND datetime(scheduled_date || ' ' || scheduled_time, '+' || duration_minutes || ' minutes') > ?3
                 AND status != 'Cancelada'
                 AND status != 'Realizada'
                 ORDER BY scheduled_date, scheduled_time"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let rows = stmt.query_map(params![therapist_str, end_str, start_str], |row| Self::row_to_appointment(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut appointments = Vec::new();
            for row in rows {
                appointments.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(appointments)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn find_by_patient(&self, patient_id: PatientId, range: Option<DateRange>) -> Result<Vec<Appointment>, RepositoryError> {
        let pool = self.pool.clone();
        let patient_str = patient_id.to_string();
        let range_clone = range.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            
            let (where_clause, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(range) = range_clone {
                (
                    "WHERE patient_id = ?1 AND scheduled_date >= ?2 AND scheduled_date <= ?3".to_string(),
                    vec![Box::new(patient_str), Box::new(range.start.format("%Y-%m-%d").to_string()), Box::new(range.end.format("%Y-%m-%d").to_string())]
                )
            } else {
                ("WHERE patient_id = ?1".to_string(), vec![Box::new(patient_str)])
            };

            let sql = format!("SELECT * FROM appointments {} ORDER BY scheduled_date DESC, scheduled_time DESC", where_clause);
            let mut stmt = conn.prepare(&sql).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| Self::row_to_appointment(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut appointments = Vec::new();
            for row in rows {
                appointments.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(appointments)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn find_reminders_due(&self, now: DateTime<Utc>) -> Result<Vec<Appointment>, RepositoryError> {
        let pool = self.pool.clone();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            // Find appointments where reminder is due (30 min before start) and not sent
            // reminder time = start_time - 30 minutes
            let mut stmt = conn.prepare(
                "SELECT * FROM appointments 
                 WHERE reminder_sent = 0 
                 AND datetime(scheduled_date || ' ' || scheduled_time, '-30 minutes') <= ?1
                 AND datetime(scheduled_date || ' ' || scheduled_time) > ?1
                 AND status = 'Programada'
                 ORDER BY scheduled_date, scheduled_time"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let rows = stmt.query_map(params![now_str], |row| Self::row_to_appointment(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut appointments = Vec::new();
            for row in rows {
                appointments.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(appointments)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_memory_pool;
    use soft_mindledger_domain::{AppointmentId, PatientId, TherapistId, AppointmentStatus, Modality, DateRange};
    use chrono::{Utc, Duration};

    fn create_test_repo() -> SqliteAppointmentRepository {
        let pool = create_memory_pool().unwrap();
        let conn = pool.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS appointments (
                id TEXT PRIMARY KEY NOT NULL,
                patient_id TEXT NOT NULL,
                professional_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'Programada',
                scheduled_date TEXT NOT NULL,
                scheduled_time TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL DEFAULT 30,
                modality TEXT NOT NULL,
                fee_cents INTEGER NOT NULL DEFAULT 0,
                notes TEXT,
                reminder_sent INTEGER NOT NULL DEFAULT 0,
                reminder_external_id TEXT,
                reagendada_from_id TEXT,
                external_calendar_id TEXT,
                calendar_provider TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_appointments_status ON appointments(status);
            CREATE INDEX IF NOT EXISTS idx_appointments_patient ON appointments(patient_id);
            CREATE INDEX IF NOT EXISTS idx_appointments_professional ON appointments(professional_id);
            CREATE INDEX IF NOT EXISTS idx_appointments_date ON appointments(scheduled_date);
            CREATE INDEX IF NOT EXISTS idx_appointments_reminder_due ON appointments(scheduled_date) WHERE reminder_sent = 0;
            "#,
        ).unwrap();
        drop(conn);
        SqliteAppointmentRepository::new(pool)
    }

    #[tokio::test]
    async fn test_create_and_get_appointment() {
        let repo = create_test_repo();
        let start = Utc::now() + Duration::hours(1);
        let end = start + Duration::minutes(50);
        let time_range = soft_mindledger_domain::DateTimeRange::new(start, end).unwrap();
        
        let appointment = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range,
            Modality::Presencial,
            50000,
            Some("Test".to_string()),
        ).unwrap();
        
        repo.create(&appointment).await.unwrap();
        let retrieved = repo.get_by_id(appointment.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, appointment.id);
        assert_eq!(retrieved.fee_cents, 50000);
        assert_eq!(retrieved.status, AppointmentStatus::Programada);
    }

    #[tokio::test]
    async fn test_update_appointment() {
        let repo = create_test_repo();
        let start = Utc::now() + Duration::hours(1);
        let end = start + Duration::minutes(50);
        let time_range = soft_mindledger_domain::DateTimeRange::new(start, end).unwrap();
        
        let mut appointment = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range,
            Modality::Virtual,
            30000,
            None,
        ).unwrap();
        
        repo.create(&appointment).await.unwrap();
        
        appointment.status = AppointmentStatus::Realizada;
        appointment.fee_cents = 35000;
        repo.update(&appointment).await.unwrap();
        
        let retrieved = repo.get_by_id(appointment.id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, AppointmentStatus::Realizada);
        assert_eq!(retrieved.fee_cents, 35000);
    }

    #[tokio::test]
    async fn test_find_overlapping() {
        let repo = create_test_repo();
        let therapist = TherapistId::new();
        
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        let time_range = soft_mindledger_domain::DateTimeRange::new(start, end).unwrap();
        
        let appt1 = Appointment::new(
            PatientId::new(),
            therapist,
            time_range.clone(),
            Modality::Presencial,
            50000,
            None,
        ).unwrap();
        
        let appt2 = Appointment::new(
            PatientId::new(),
            therapist,
            time_range.clone(),
            Modality::Virtual,
            30000,
            None,
        ).unwrap();
        
        repo.create(&appt1).await.unwrap();
        repo.create(&appt2).await.unwrap();
        
        let range = DateRange {
            start: start - Duration::hours(1),
            end: end + Duration::hours(1),
        };
        
        let overlapping = repo.find_overlapping(therapist, range).await.unwrap();
        assert_eq!(overlapping.len(), 2);
    }

    #[tokio::test]
    async fn test_find_reminders_due() {
        let repo = create_test_repo();
        
        // Appointment due for reminder (30 min before)
        let start = Utc::now() + Duration::minutes(25);
        let end = start + Duration::minutes(50);
        let time_range = soft_mindledger_domain::DateTimeRange::new(start, end).unwrap();
        
        let mut appt_due = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range,
            Modality::Presencial,
            50000,
            None,
        ).unwrap();
        appt_due.reminder_sent = false;
        repo.create(&appt_due).await.unwrap();
        
        // Appointment not due yet (60 min before)
        let start2 = Utc::now() + Duration::minutes(65);
        let end2 = start2 + Duration::minutes(50);
        let time_range2 = soft_mindledger_domain::DateTimeRange::new(start2, end2).unwrap();
        
        let mut appt_not_due = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range2,
            Modality::Presencial,
            50000,
            None,
        ).unwrap();
        appt_not_due.reminder_sent = false;
        repo.create(&appt_not_due).await.unwrap();
        
        // Already sent
        let start3 = Utc::now() + Duration::minutes(25);
        let end3 = start3 + Duration::minutes(50);
        let time_range3 = soft_mindledger_domain::DateTimeRange::new(start3, end3).unwrap();
        
        let mut appt_sent = Appointment::new(
            PatientId::new(),
            TherapistId::new(),
            time_range3,
            Modality::Presencial,
            50000,
            None,
        ).unwrap();
        appt_sent.reminder_sent = true;
        repo.create(&appt_sent).await.unwrap();
        
        let due = repo.find_reminders_due(Utc::now()).await.unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, appt_due.id);
    }
}