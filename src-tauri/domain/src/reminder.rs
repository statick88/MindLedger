use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

use crate::error::DomainError;
use crate::identifiers::{AppointmentId, PatientId, ReminderId};

/// Reminder channel for notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReminderChannel {
    Push,
    Email,
    Sms,
    InApp,
}

impl fmt::Display for ReminderChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReminderChannel::Push => write!(f, "push"),
            ReminderChannel::Email => write!(f, "email"),
            ReminderChannel::Sms => write!(f, "sms"),
            ReminderChannel::InApp => write!(f, "in_app"),
        }
    }
}

/// Reminder template identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReminderTemplate {
    Session30Min,
    Session1Hour,
    Session1Day,
    Custom,
}

/// Reminder domain entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reminder {
    pub id: ReminderId,
    pub appointment_id: AppointmentId,
    pub patient_id: PatientId,
    pub remind_at: DateTime<Utc>,
    pub channel: ReminderChannel,
    pub template_id: ReminderTemplate,
    pub sent_at: Option<DateTime<Utc>>,
    pub external_id: Option<String>, // For OS notification ID
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Reminder {
    pub fn new(
        appointment_id: AppointmentId,
        patient_id: PatientId,
        remind_at: DateTime<Utc>,
        channel: ReminderChannel,
        template_id: ReminderTemplate,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ReminderId::new(),
            appointment_id,
            patient_id,
            remind_at,
            channel,
            template_id,
            sent_at: None,
            external_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_sent(&mut self, external_id: Option<String>) {
        self.sent_at = Some(Utc::now());
        self.external_id = external_id;
        self.updated_at = Utc::now();
    }

    pub fn is_sent(&self) -> bool {
        self.sent_at.is_some()
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        !self.is_sent() && now >= self.remind_at
    }
}

/// Due reminder with appointment context for notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DueReminder {
    pub reminder: Reminder,
    pub appointment_summary: AppointmentSummary,
}

/// Lightweight appointment summary for notification context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentSummary {
    pub id: AppointmentId,
    pub patient_name: String,
    pub therapist_name: String,
    pub start_time: DateTime<Utc>,
    pub modality: String,
    pub location: Option<String>,
}

/// Reminder notifier trait for sending notifications
#[async_trait]
pub trait ReminderNotifier: Send + Sync {
    /// Sends a reminder notification
    async fn notify(&self, reminder: &DueReminder) -> Result<(), ReminderError>;

    /// Returns the channel this notifier handles
    fn channel(&self) -> ReminderChannel;
}

/// Reminder scheduler trait
#[async_trait]
pub trait ReminderScheduler: Send + Sync {
    /// Schedules a new reminder
    async fn schedule(&self, reminder: Reminder) -> Result<(), ReminderError>;

    /// Cancels a scheduled reminder
    async fn cancel(&self, reminder_id: ReminderId) -> Result<(), ReminderError>;

    /// Processes due reminders and returns count processed
    async fn process_due(&self, now: DateTime<Utc>) -> Result<usize, ReminderError>;

    /// Gets all pending reminders
    async fn get_pending(&self) -> Result<Vec<Reminder>, ReminderError>;

    /// Starts the background scheduler
    async fn start(&self) -> Result<(), ReminderError>;

    /// Stops the background scheduler gracefully
    async fn stop(&self) -> Result<(), ReminderError>;
}

/// Reminder errors
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum ReminderError {
    #[error("Reminder not found: {0}")]
    NotFound(String),

    #[error("Reminder already sent: {0}")]
    AlreadySent(String),

    #[error("Cannot schedule reminder in the past: {0}")]
    PastTime(String),

    #[error("Notification failed: {0}")]
    NotificationFailed(String),

    #[error("Scheduler not running")]
    NotRunning,

    #[error("Scheduler already running")]
    AlreadyRunning,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<ReminderError> for DomainError {
    fn from(err: ReminderError) -> Self {
        DomainError::ReminderError(err.to_string())
    }
}

/// Configuration for reminder scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderSchedulerConfig {
    /// Tick interval for checking due reminders
    pub tick_interval: Duration,
    /// Maximum reminders to process per tick
    pub batch_size: usize,
    /// Retry interval for failed notifications
    pub retry_interval: Duration,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for ReminderSchedulerConfig {
    fn default() -> Self {
        Self {
            tick_interval: Duration::minutes(1),
            batch_size: 100,
            retry_interval: Duration::minutes(5),
            max_retries: 3,
        }
    }
}

impl ReminderSchedulerConfig {
    pub fn new(tick_minutes: u64) -> Self {
        Self {
            tick_interval: Duration::minutes(tick_minutes as i64),
            ..Default::default()
        }
    }
}

/// Reminder policy - hardcoded 30 minutes before session
pub struct ReminderPolicy;

impl ReminderPolicy {
    /// Returns the reminder time for a session (30 minutes before start)
    pub fn reminder_time(session_start: DateTime<Utc>) -> DateTime<Utc> {
        session_start - Duration::minutes(30)
    }

    /// Returns the reminder time for a session with custom minutes
    pub fn reminder_time_custom(session_start: DateTime<Utc>, minutes_before: i64) -> DateTime<Utc> {
        session_start - Duration::minutes(minutes_before)
    }

    /// Validates that reminder time is in the future
    pub fn validate_reminder_time(remind_at: DateTime<Utc>) -> Result<(), ReminderError> {
        let now = Utc::now();
        if remind_at <= now {
            return Err(ReminderError::PastTime(
                format!("Reminder time {} must be in the future", remind_at)
            ));
        }
        Ok(())
    }

    /// Creates a reminder for an appointment with default 30-min policy
    pub fn create_reminder(
        appointment_id: AppointmentId,
        patient_id: PatientId,
        session_start: DateTime<Utc>,
        channel: ReminderChannel,
        template_id: ReminderTemplate,
    ) -> Result<Reminder, ReminderError> {
        let remind_at = Self::reminder_time(session_start);
        Self::validate_reminder_time(remind_at)?;
        
        Ok(Reminder::new(
            appointment_id,
            patient_id,
            remind_at,
            channel,
            template_id,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::{AppointmentId, PatientId};
    use chrono::{TimeZone, Timelike};

    #[test]
    fn test_reminder_policy_30_minutes() {
        let session_start = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let reminder_time = ReminderPolicy::reminder_time(session_start);
        
        assert_eq!(reminder_time, session_start - Duration::minutes(30));
        assert_eq!(reminder_time.minute(), 30);
    }

    #[test]
    fn test_reminder_policy_custom_minutes() {
        let session_start = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let reminder_time = ReminderPolicy::reminder_time_custom(session_start, 60);
        
        assert_eq!(reminder_time, session_start - Duration::minutes(60));
    }

    #[test]
    fn test_reminder_policy_validate_future() {
        let future = Utc::now() + Duration::minutes(60);
        assert!(ReminderPolicy::validate_reminder_time(future).is_ok());
    }

    #[test]
    fn test_reminder_policy_validate_past() {
        let past = Utc::now() - Duration::minutes(10);
        let result = ReminderPolicy::validate_reminder_time(past);
        assert!(matches!(result, Err(ReminderError::PastTime(_))));
    }

    #[test]
    fn test_reminder_creation() {
        let appointment_id = AppointmentId::new();
        let patient_id = PatientId::new();
        let session_start = Utc::now() + Duration::hours(1);
        
        let reminder = ReminderPolicy::create_reminder(
            appointment_id,
            patient_id,
            session_start,
            ReminderChannel::Push,
            ReminderTemplate::Session30Min,
        ).unwrap();
        
        assert_eq!(reminder.appointment_id, appointment_id);
        assert_eq!(reminder.patient_id, patient_id);
        assert_eq!(reminder.channel, ReminderChannel::Push);
        assert_eq!(reminder.template_id, ReminderTemplate::Session30Min);
        assert!(!reminder.is_sent());
        assert!(reminder.remind_at > Utc::now());
    }

    #[test]
    fn test_reminder_is_due() {
        let mut reminder = Reminder::new(
            AppointmentId::new(),
            PatientId::new(),
            Utc::now() + Duration::minutes(5),
            ReminderChannel::Push,
            ReminderTemplate::Session30Min,
        );
        
        // Not due yet
        assert!(!reminder.is_due(Utc::now()));
        
        // Due now
        assert!(reminder.is_due(reminder.remind_at));
        
        // After reminder time
        assert!(reminder.is_due(reminder.remind_at + Duration::minutes(1)));
        
        // After sent
        reminder.mark_sent(Some("ext_123".to_string()));
        assert!(!reminder.is_due(Utc::now()));
    }

    #[test]
    fn test_reminder_mark_sent() {
        let mut reminder = Reminder::new(
            AppointmentId::new(),
            PatientId::new(),
            Utc::now() + Duration::minutes(30),
            ReminderChannel::Email,
            ReminderTemplate::Session30Min,
        );
        
        assert!(!reminder.is_sent());
        assert!(reminder.sent_at.is_none());
        
        reminder.mark_sent(Some("notification_123".to_string()));
        
        assert!(reminder.is_sent());
        assert!(reminder.sent_at.is_some());
        assert_eq!(reminder.external_id, Some("notification_123".to_string()));
    }

    #[test]
    fn test_reminder_channel_display() {
        assert_eq!(ReminderChannel::Push.to_string(), "push");
        assert_eq!(ReminderChannel::Email.to_string(), "email");
        assert_eq!(ReminderChannel::Sms.to_string(), "sms");
        assert_eq!(ReminderChannel::InApp.to_string(), "in_app");
    }

    #[test]
    fn test_reminder_template_serialization() {
        let template = ReminderTemplate::Session30Min;
        let json = serde_json::to_string(&template).unwrap();
        // serde snake_case converts Session30Min -> session30_min (no underscore between number and letter)
        assert_eq!(json, "\"session30_min\"");
        
        let parsed: ReminderTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ReminderTemplate::Session30Min);
    }

    #[test]
    fn test_reminder_scheduler_config() {
        let config = ReminderSchedulerConfig::new(2);
        assert_eq!(config.tick_interval, Duration::minutes(2));
        
        let default = ReminderSchedulerConfig::default();
        assert_eq!(default.tick_interval, Duration::minutes(1));
        assert_eq!(default.batch_size, 100);
        assert_eq!(default.max_retries, 3);
    }
}