use crate::database::DbPool;
use soft_gloria_domain::{
    Reminder, ReminderId, AppointmentId, PatientId, ReminderChannel, ReminderTemplate,
    ReminderRepository,
    RepositoryError,
};
use rusqlite::params;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

pub struct SqliteReminderRepository {
    pool: DbPool,
}

impl SqliteReminderRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

fn row_to_reminder(row: &rusqlite::Row) -> rusqlite::Result<Reminder> {
        let id: String = row.get("id")?;
        let appointment_id: String = row.get("appointment_id")?;
        let patient_id: String = row.get("patient_id")?;
        let remind_at: String = row.get("remind_at")?;
        let channel: String = row.get("channel")?;
        let template_id: Option<String> = row.get("template_id")?;
        let sent_at: Option<String> = row.get("sent_at")?;
        let external_id: Option<String> = row.get("external_id")?;
        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;

        Ok(Reminder {
            id: ReminderId(Uuid::parse_str(&id).unwrap()),
            appointment_id: AppointmentId(Uuid::parse_str(&appointment_id).unwrap()),
            patient_id: PatientId(Uuid::parse_str(&patient_id).unwrap()),
            remind_at: DateTime::parse_from_rfc3339(&remind_at).unwrap_or_default().into(),
            channel: match channel.as_str() {
                "push" => ReminderChannel::Push,
                "email" => ReminderChannel::Email,
                "sms" => ReminderChannel::Sms,
                "in_app" => ReminderChannel::InApp,
                _ => ReminderChannel::Push,
            },
            template_id: match template_id.as_deref() {
                Some("session_30_min") => ReminderTemplate::Session30Min,
                Some("session_1_hour") => ReminderTemplate::Session1Hour,
                Some("session_1_day") => ReminderTemplate::Session1Day,
                Some("custom") => ReminderTemplate::Custom,
                _ => ReminderTemplate::Session30Min,
            },
            sent_at: sent_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.into()),
            external_id,
            created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().into(),
            updated_at: DateTime::parse_from_rfc3339(&updated_at).unwrap_or_default().into(),
        })
    }
}

#[async_trait]
impl ReminderRepository for SqliteReminderRepository {
    async fn create(&self, reminder: &Reminder) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let reminder = reminder.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            
            let channel_str = match reminder.channel {
                ReminderChannel::Push => "push",
                ReminderChannel::Email => "email",
                ReminderChannel::Sms => "sms",
                ReminderChannel::InApp => "in_app",
            };

            let template_str = match reminder.template_id {
                ReminderTemplate::Session30Min => "session_30_min",
                ReminderTemplate::Session1Hour => "session_1_hour",
                ReminderTemplate::Session1Day => "session_1_day",
                ReminderTemplate::Custom => "custom",
            };

            let remind_at = reminder.remind_at.to_rfc3339();
            let sent_at = reminder.sent_at.map(|s| s.to_rfc3339());
            let now = Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO reminders (
                    id, appointment_id, patient_id, remind_at, channel, template_id,
                    sent_at, external_id, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    reminder.id.to_string(),
                    reminder.appointment_id.to_string(),
                    reminder.patient_id.to_string(),
                    remind_at,
                    channel_str,
                    template_str,
                    sent_at,
                    reminder.external_id,
                    now,
                    now,
                ],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_by_id(&self, id: ReminderId) -> Result<Option<Reminder>, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare("SELECT * FROM reminders WHERE id = ?1")
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut rows = stmt.query(params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| RepositoryError::Database(e.to_string()))? {
                let reminder = Self::row_to_reminder(row).map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(Some(reminder))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn list_by_appointment(&self, appointment_id: AppointmentId) -> Result<Vec<Reminder>, RepositoryError> {
        let pool = self.pool.clone();
        let appt_str = appointment_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare("SELECT * FROM reminders WHERE appointment_id = ?1 ORDER BY remind_at")
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let rows = stmt.query_map(params![appt_str], |row| Self::row_to_reminder(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut reminders = Vec::new();
            for row in rows {
                reminders.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(reminders)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn list_by_patient(&self, patient_id: PatientId) -> Result<Vec<Reminder>, RepositoryError> {
        let pool = self.pool.clone();
        let patient_str = patient_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare("SELECT * FROM reminders WHERE patient_id = ?1 ORDER BY remind_at")
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let rows = stmt.query_map(params![patient_str], |row| Self::row_to_reminder(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut reminders = Vec::new();
            for row in rows {
                reminders.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(reminders)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn mark_sent(&self, id: ReminderId, external_id: String) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        let now = Utc::now().to_rfc3339();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let affected = conn.execute(
                "UPDATE reminders SET sent_at = ?1, external_id = ?2, updated_at = ?3 WHERE id = ?4",
                params![now, external_id, now, id_str],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            if affected == 0 {
                return Err(RepositoryError::NotFound(format!("Reminder not found: {}", id_str)));
            }
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn cancel(&self, id: ReminderId) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        let now = Utc::now().to_rfc3339();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let affected = conn.execute(
                "UPDATE reminders SET sent_at = ?1, external_id = 'cancelled', updated_at = ?2 WHERE id = ?3",
                params![now, now, id_str],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            if affected == 0 {
                return Err(RepositoryError::NotFound(format!("Reminder not found: {}", id_str)));
            }
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn find_due(&self, now: DateTime<Utc>) -> Result<Vec<Reminder>, RepositoryError> {
        let pool = self.pool.clone();
        let now_str = now.to_rfc3339();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT * FROM reminders WHERE sent_at IS NULL AND remind_at <= ?1 ORDER BY remind_at LIMIT 100"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let rows = stmt.query_map(params![now_str], |row| Self::row_to_reminder(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut reminders = Vec::new();
            for row in rows {
                reminders.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(reminders)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_memory_pool;
    use soft_gloria_domain::{ReminderId, AppointmentId, PatientId, ReminderChannel, ReminderTemplate};
    use chrono::{Utc, Duration};

    fn create_test_repo() -> SqliteReminderRepository {
        let pool = create_memory_pool().unwrap();
        let conn = pool.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS reminders (
                id TEXT PRIMARY KEY NOT NULL,
                appointment_id TEXT NOT NULL,
                patient_id TEXT NOT NULL,
                remind_at TEXT NOT NULL,
                channel TEXT NOT NULL DEFAULT 'push',
                template_id TEXT,
                sent_at TEXT,
                external_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_reminders_due ON reminders(remind_at) WHERE sent_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_reminders_appointment ON reminders(appointment_id);
            CREATE INDEX IF NOT EXISTS idx_reminders_patient ON reminders(patient_id);
            "#,
        ).unwrap();
        drop(conn);
        SqliteReminderRepository::new(pool)
    }

    #[tokio::test]
    async fn test_create_and_get_reminder() {
        let repo = create_test_repo();
        let reminder = Reminder::new(
            AppointmentId::new(),
            PatientId::new(),
            Utc::now() + Duration::minutes(30),
            ReminderChannel::Push,
            ReminderTemplate::Session30Min,
        );
        
        repo.create(&reminder).await.unwrap();
        let retrieved = repo.get_by_id(reminder.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, reminder.id);
        assert_eq!(retrieved.channel, ReminderChannel::Push);
        assert_eq!(retrieved.template_id, ReminderTemplate::Session30Min);
    }

    #[tokio::test]
    async fn test_mark_sent() {
        let repo = create_test_repo();
        let reminder = Reminder::new(
            AppointmentId::new(),
            PatientId::new(),
            Utc::now() + Duration::minutes(30),
            ReminderChannel::Email,
            ReminderTemplate::Session1Hour,
        );
        
        repo.create(&reminder).await.unwrap();
        repo.mark_sent(reminder.id, "external_123".to_string()).await.unwrap();
        
        let retrieved = repo.get_by_id(reminder.id).await.unwrap().unwrap();
        assert!(retrieved.sent_at.is_some());
        assert_eq!(retrieved.external_id, Some("external_123".to_string()));
    }

    #[tokio::test]
    async fn test_find_due() {
        let repo = create_test_repo();
        
        // Due reminder
        let due_reminder = Reminder::new(
            AppointmentId::new(),
            PatientId::new(),
            Utc::now() - Duration::minutes(5),
            ReminderChannel::Push,
            ReminderTemplate::Session30Min,
        );
        repo.create(&due_reminder).await.unwrap();
        
        // Not due yet
        let future_reminder = Reminder::new(
            AppointmentId::new(),
            PatientId::new(),
            Utc::now() + Duration::minutes(60),
            ReminderChannel::Push,
            ReminderTemplate::Session30Min,
        );
        repo.create(&future_reminder).await.unwrap();
        
        let due = repo.find_due(Utc::now()).await.unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, due_reminder.id);
    }
}