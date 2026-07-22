use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use crate::error::DomainError;
use crate::identifiers::AppointmentId;

/// Calendar event DTO for external calendar sync
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Option<String>, // External ID (Google event ID, Outlook ID, etc.)
    pub summary: String,
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub attendees: Vec<CalendarAttendee>,
    pub location: Option<String>,
    pub recurrence_rule: Option<String>, // RRULE format
    pub reminders: Vec<CalendarReminder>,
    pub status: CalendarEventStatus,
    pub transparency: CalendarTransparency,
}

impl CalendarEvent {
    pub fn new(summary: String, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            id: None,
            summary,
            description: None,
            start,
            end,
            attendees: Vec::new(),
            location: None,
            recurrence_rule: None,
            reminders: vec![CalendarReminder::default_30min()],
            status: CalendarEventStatus::Confirmed,
            transparency: CalendarTransparency::Opaque,
        }
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }
}

/// Calendar event attendee
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarAttendee {
    pub email: String,
    pub name: Option<String>,
    pub role: AttendeeRole,
    pub status: AttendeeStatus,
}

/// Attendee role in the event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttendeeRole {
    Required,
    Optional,
    Organizer,
}

/// Attendee response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttendeeStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
}

/// Calendar reminder configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarReminder {
    pub method: ReminderMethod,
    pub minutes_before: i32,
}

impl CalendarReminder {
    pub fn default_30min() -> Self {
        Self {
            method: ReminderMethod::Popup,
            minutes_before: 30,
        }
    }

    pub fn email(minutes_before: i32) -> Self {
        Self {
            method: ReminderMethod::Email,
            minutes_before,
        }
    }
}

/// Reminder delivery method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReminderMethod {
    Popup,
    Email,
    Sms,
}

/// Calendar event status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalendarEventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

/// Calendar event transparency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalendarTransparency {
    Opaque,
    Transparent,
}

/// Date range for calendar queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, DomainError> {
        if start >= end {
            return Err(DomainError::Validation(
                "Start must be before end".to_string(),
            ));
        }
        Ok(Self { start, end })
    }

    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = start + Duration::days(1);
        Self { start, end }
    }

    pub fn this_week() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = start + Duration::weeks(1);
        Self { start, end }
    }

    pub fn this_month() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = start + Duration::days(32); // Approximate
        Self { start, end }
    }
}

/// Calendar synchronization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
    pub conflicts: Vec<SyncConflict>,
    pub sync_token: Option<String>,
}

/// Synchronization conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub local_event: CalendarEvent,
    pub remote_event: CalendarEvent,
    pub conflict_type: ConflictType,
}

/// Type of synchronization conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    ModifiedBoth,
    DeletedLocalModifiedRemote,
    ModifiedLocalDeletedRemote,
}

/// Calendar provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalendarProviderType {
    Os,
    Google,
    Outlook,
}

impl fmt::Display for CalendarProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarProviderType::Os => write!(f, "os"),
            CalendarProviderType::Google => write!(f, "google"),
            CalendarProviderType::Outlook => write!(f, "outlook"),
        }
    }
}

/// Calendar provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarProviderConfig {
    pub provider_type: CalendarProviderType,
    pub calendar_id: Option<String>,
    pub sync_enabled: bool,
    pub sync_range: DateRange,
    pub auth_config: Option<AuthConfig>,
}

/// Authentication configuration for external providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    Google {
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
        token_expiry: Option<DateTime<Utc>>,
    },
    Outlook {
        tenant_id: String,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
        token_expiry: Option<DateTime<Utc>>,
    },
    Os {
        calendar_name: Option<String>, // macOS: calendar name in Calendar.app
    },
}

/// Calendar provider trait for external calendar integration
#[async_trait]
pub trait CalendarProvider: Send + Sync {
    /// Returns the provider type
    fn provider_type(&self) -> CalendarProviderType;

    /// Lists events in the given date range
    async fn list_events(&self, range: DateRange) -> Result<Vec<CalendarEvent>, CalendarError>;

    /// Creates a new event
    async fn create_event(&self, event: CalendarEvent) -> Result<String, CalendarError>;

    /// Updates an existing event
    async fn update_event(&self, external_id: &str, event: CalendarEvent) -> Result<(), CalendarError>;

    /// Deletes an event
    async fn delete_event(&self, external_id: &str) -> Result<(), CalendarError>;

    /// Gets the sync token for incremental sync
    async fn get_sync_token(&self) -> Result<Option<String>, CalendarError>;

    /// Performs incremental sync using sync token
    async fn sync_incremental(&self, token: &str) -> Result<SyncResult, CalendarError>;

    /// Checks if provider is available and authenticated
    async fn is_available(&self) -> bool;
}

/// Calendar errors
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum CalendarError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Event not found: {0}")]
    NotFound(String),

    #[error("Rate limited: retry after {retry_after_seconds} seconds")]
    RateLimited { retry_after_seconds: u64 },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Invalid event data: {0}")]
    InvalidEvent(String),

    #[error("Sync conflict: {0}")]
    Conflict(String),

    #[error("Provider not available: {0}")]
    NotAvailable(String),

    #[error("Not implemented for this provider")]
    NotImplemented,

    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl From<CalendarError> for DomainError {
    fn from(err: CalendarError) -> Self {
        DomainError::CalendarError(err.to_string())
    }
}

/// Calendar sync state for an appointment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSyncState {
    pub appointment_id: AppointmentId,
    pub external_id: Option<String>,
    pub provider_type: CalendarProviderType,
    pub calendar_id: Option<String>,
    pub last_synced: Option<DateTime<Utc>>,
    pub sync_token: Option<String>,
    pub is_deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_calendar_event_creation() {
        let start = Utc.with_ymd_and_hms(2025, 1, 15, 10, 0, 0).single().unwrap();
        let end = start + Duration::minutes(50);
        
        let event = CalendarEvent::new("Sesión: Juan Pérez".to_string(), start, end);
        
        assert_eq!(event.summary, "Sesión: Juan Pérez");
        assert_eq!(event.duration_minutes(), 50);
        assert_eq!(event.reminders.len(), 1);
        assert_eq!(event.reminders[0].minutes_before, 30);
    }

    #[test]
    fn test_date_range_creation() {
        let start = Utc.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).single().unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 16, 0, 0, 0).single().unwrap();
        
        let range = DateRange::new(start, end).unwrap();
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_date_range_invalid() {
        let start = Utc.with_ymd_and_hms(2025, 1, 15, 0, 0, 0).single().unwrap();
        let end = start - Duration::hours(1);
        
        let result = DateRange::new(start, end);
        assert!(result.is_err());
    }

    #[test]
    fn test_date_range_helpers() {
        let today = DateRange::today();
        assert!(today.end > today.start);
        
        let week = DateRange::this_week();
        assert!((week.end - week.start).num_days() == 7);
        
        let month = DateRange::this_month();
        assert!((month.end - month.start).num_days() >= 28);
    }

    #[test]
    fn test_calendar_event_status_serialization() {
        let status = CalendarEventStatus::Confirmed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"confirmed\"");
        
        let parsed: CalendarEventStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CalendarEventStatus::Confirmed);
    }

    #[test]
    fn test_reminder_methods() {
        let default = CalendarReminder::default_30min();
        assert_eq!(default.minutes_before, 30);
        assert_eq!(default.method, ReminderMethod::Popup);
        
        let email = CalendarReminder::email(60);
        assert_eq!(email.minutes_before, 60);
        assert_eq!(email.method, ReminderMethod::Email);
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(CalendarProviderType::Os.to_string(), "os");
        assert_eq!(CalendarProviderType::Google.to_string(), "google");
        assert_eq!(CalendarProviderType::Outlook.to_string(), "outlook");
    }

    #[test]
    fn test_calendar_error_conversion() {
        let cal_err = CalendarError::AuthFailed("Invalid token".to_string());
        let domain_err: DomainError = cal_err.into();
        assert!(matches!(domain_err, DomainError::CalendarError(_)));
    }
}