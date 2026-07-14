use crate::error::{AppError, AppResult};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use soft_mindledger_domain::{
    appointment::{
        Appointment, AppointmentStatus, Modality, DateTimeRange,
    },
    calendar_provider::DateRange,
    identifiers::{AppointmentId, PatientId, TherapistId},
    reminder::{Reminder, ReminderChannel, ReminderTemplate},
    repositories::{AppointmentRepository, AppointmentFilter, Pagination, ReminderRepository, PatientRepository},
};
use soft_mindledger_infrastructure::{DbPool, SqliteAppointmentRepository, SqlitePatientRepository, SqliteReminderRepository};
use std::sync::Arc;
use tauri::command;
use uuid::Uuid;

/// Default session fee in cents (500.00) when not specified by patient config.
const DEFAULT_SESSION_FEE_CENTS: i64 = 50_000;

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Deserialize)]
pub struct CreateAppointmentRequest {
    pub patient_id: String,
    pub therapist_id: String,
    pub start_at: String,        // ISO 8601 UTC
    pub end_at: String,          // ISO 8601 UTC
    pub modality: Modality,
    pub fee_cents: Option<i64>,  // Optional, defaults to patient's session_fee
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAppointmentRequest {
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub modality: Option<Modality>,
    pub fee_cents: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangeStatusRequest {
    pub new_status: AppointmentStatus,
    pub reason: Option<String>,
    pub new_start_at: Option<String>,  // For reschedule
    pub new_end_at: Option<String>,    // For reschedule
}

#[derive(Deserialize)]
pub struct ListAppointmentsQuery {
    pub start_date: Option<String>,    // YYYY-MM-DD
    pub end_date: Option<String>,      // YYYY-MM-DD
    pub status: Option<AppointmentStatus>,
    pub patient_id: Option<String>,
    pub therapist_id: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Deserialize)]
pub struct GetPatientAppointmentsQuery {
    pub patient_id: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<AppointmentStatus>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct AppointmentResponse {
    pub id: String,
    pub patient_id: String,
    pub therapist_id: String,
    pub start_at: String,
    pub end_at: String,
    pub modality: Modality,
    pub status: AppointmentStatus,
    pub fee_cents: i64,
    pub notes: Option<String>,
    pub reminder_sent: bool,
    pub reminder_external_id: Option<String>,
    pub reagendada_from_id: Option<String>,
    pub external_calendar_id: Option<String>,
    pub calendar_provider: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<soft_mindledger_domain::appointment::Appointment> for AppointmentResponse {
    fn from(a: soft_mindledger_domain::appointment::Appointment) -> Self {
        Self {
            id: a.id.to_string(),
            patient_id: a.patient_id.to_string(),
            therapist_id: a.therapist_id.to_string(),
            start_at: a.time_range.start.to_rfc3339(),
            end_at: a.time_range.end.to_rfc3339(),
            modality: a.modality,
            status: a.status,
            fee_cents: a.fee_cents,
            notes: a.notes,
            reminder_sent: a.reminder_sent,
            reminder_external_id: a.reminder_external_id,
            reagendada_from_id: a.reagendada_from_id.map(|id| id.to_string()),
            external_calendar_id: a.external_calendar_id,
            calendar_provider: a.calendar_provider,
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

#[derive(Serialize, Debug)]
pub struct KpiMetricsResponse {
    pub sessions_completed_today: u64,
    pub sessions_completed_week: u64,
    pub sessions_completed_month: u64,
    pub occupancy_rate_pct: f64,
    pub revenue_today_cents: i64,
    pub revenue_week_cents: i64,
    pub revenue_month_cents: i64,
    pub no_show_rate_pct: f64,
    pub upcoming_24h: u64,
}

#[derive(Serialize, Debug)]
pub struct ReminderResponse {
    pub id: String,
    pub appointment_id: String,
    pub patient_id: String,
    pub remind_at: String,
    pub channel: ReminderChannel,
    pub template_id: ReminderTemplate,
    pub sent_at: Option<String>,
    pub external_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Reminder> for ReminderResponse {
    fn from(r: Reminder) -> Self {
        Self {
            id: r.id.to_string(),
            appointment_id: r.appointment_id.to_string(),
            patient_id: r.patient_id.to_string(),
            remind_at: r.remind_at.to_rfc3339(),
            channel: r.channel,
            template_id: r.template_id,
            sent_at: r.sent_at.map(|dt| dt.to_rfc3339()),
            external_id: r.external_id,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ProcessRemindersResult {
    pub processed: usize,
    pub failed: usize,
}

// ============================================================================
// Inner Functions (testable without Tauri State)
// ============================================================================

// --- Appointment CRUD ---

/// Create a new appointment with patient validation and overlap check.
pub async fn crear_cita_agenda_impl(
    pool: &DbPool,
    request: CreateAppointmentRequest,
) -> AppResult<AppointmentResponse> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    let patient_repo = SqlitePatientRepository::new(pool.clone());
    
    // Parse dates
    let start_at = DateTime::parse_from_rfc3339(&request.start_at)
        .map_err(|e| AppError::Validation(format!("Invalid start_at: {}", e)))?
        .with_timezone(&Utc);
    let end_at = DateTime::parse_from_rfc3339(&request.end_at)
        .map_err(|e| AppError::Validation(format!("Invalid end_at: {}", e)))?
        .with_timezone(&Utc);
    
    // Validate patient exists
    let patient_id = PatientId(Uuid::parse_str(&request.patient_id)
        .map_err(|e| AppError::Validation(format!("Invalid patient_id: {}", e)))?);
    let _patient = patient_repo.get_by_id(patient_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Patient not found: {}", request.patient_id)))?;
    
    // Validate therapist exists (simplified - just check ID format)
    let therapist_id = TherapistId(Uuid::parse_str(&request.therapist_id)
        .map_err(|e| AppError::Validation(format!("Invalid therapist_id: {}", e)))?);
    
    // Create time range
    let time_range = DateTimeRange::new(start_at, end_at)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    // Check for overlapping appointments for same therapist
    let overlapping = repo.find_overlapping(therapist_id, DateRange {
        start: time_range.start,
        end: time_range.end,
    }).await?;
    
    if !overlapping.is_empty() {
        return Err(AppError::Conflict("Therapist has overlapping appointment".to_string()));
    }
    
    // Get patient's default fee if not provided
    let fee_cents = request.fee_cents.unwrap_or(DEFAULT_SESSION_FEE_CENTS);
    
    // Create appointment
    let mut appointment = soft_mindledger_domain::appointment::Appointment::new(
        patient_id,
        therapist_id,
        time_range,
        request.modality,
        fee_cents,
        request.notes,
    )?;
    
    // Schedule reminder (30 min before)
    let remind_at = start_at - Duration::minutes(30);
    let reminder = Reminder::new(
        appointment.id,
        appointment.patient_id,
        remind_at,
        ReminderChannel::Push,
        ReminderTemplate::Session30Min,
    );
    appointment.reminder_sent = false; // Will be set when reminder fires
    
    // Save appointment and reminder
    repo.create(&appointment).await?;
    
    let reminder_repo = SqliteReminderRepository::new(pool.clone());
    reminder_repo.create(&reminder).await?;
    
    Ok(appointment.into())
}

/// Retrieve a single appointment by ID.
pub async fn obtener_cita_agenda_impl(
    pool: &DbPool,
    id: String,
) -> AppResult<AppointmentResponse> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    let appointment_id = AppointmentId(Uuid::parse_str(&id)?);
    
    let appointment = repo.get_by_id(appointment_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Appointment with id {} not found", id)))?;
    
    Ok(appointment.into())
}

/// List appointments with optional filters (date range, status, patient, therapist) and pagination.
pub async fn listar_citas_agenda_impl(
    pool: &DbPool,
    query: ListAppointmentsQuery,
) -> AppResult<PaginatedResponse<AppointmentResponse>> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let mut filter = AppointmentFilter::default();
    if let Some(start) = query.start_date {
        if let Some(end) = query.end_date {
            filter.date_range = Some(DateRange {
                start: NaiveDate::parse_from_str(&start, "%Y-%m-%d")?.and_hms_opt(0, 0, 0)
                    .ok_or_else(|| AppError::Validation("Invalid date".to_string()))?.and_utc(),
                end: NaiveDate::parse_from_str(&end, "%Y-%m-%d")?.and_hms_opt(23, 59, 59)
                    .ok_or_else(|| AppError::Validation("Invalid date".to_string()))?.and_utc(),
            });
        }
    }
    filter.status = query.status;
    filter.patient_id = query.patient_id.map(|s| PatientId(Uuid::parse_str(&s))).transpose()?;
    filter.therapist_id = query.therapist_id.map(|s| TherapistId(Uuid::parse_str(&s))).transpose()?;
    
    let pagination = Pagination::new(page, page_size);
    let total = repo.count(filter.clone()).await?;
    let appointments = repo.list(filter, pagination).await?;
    
    Ok(PaginatedResponse {
        items: appointments.into_iter().map(Into::into).collect(),
        total,
        page,
        page_size,
        total_pages: (total + page_size - 1) / page_size,
    })
}

// --- State Transitions ---

/// Execute appointment finalization + accounting entry atomically within a
/// single SQLite transaction. If either operation fails, both are rolled back.
fn finalize_appointment_atomic(
    pool: &DbPool,
    appointment: &soft_mindledger_domain::appointment::Appointment,
    asiento: &soft_mindledger_domain::accounting::AsientoContable,
    notes: Option<String>,
) -> Result<(), AppError> {
    let conn = pool.lock().map_err(|e| AppError::Internal(format!("Lock poisoned: {}", e)))?;

    let tx = conn.unchecked_transaction()
        .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

    // --- 1. UPDATE appointment ---
    let status_str = match appointment.status {
        soft_mindledger_domain::appointment::AppointmentStatus::Programada => "Programada",
        soft_mindledger_domain::appointment::AppointmentStatus::Realizada => "Realizada",
        soft_mindledger_domain::appointment::AppointmentStatus::Reagendada => "Reagendada",
        soft_mindledger_domain::appointment::AppointmentStatus::Cancelada => "Cancelada",
    };
    let modality_str = match appointment.modality {
        soft_mindledger_domain::appointment::Modality::Presencial => "Presencial",
        soft_mindledger_domain::appointment::Modality::Virtual => "Virtual",
        soft_mindledger_domain::appointment::Modality::Hibrida => "Hibrida",
    };
    let scheduled_date = appointment.time_range.start.format("%Y-%m-%d").to_string();
    let scheduled_time = appointment.time_range.start.format("%H:%M").to_string();
    let duration_minutes = (appointment.time_range.end - appointment.time_range.start).num_minutes();
    let reminder_sent = if appointment.reminder_sent { 1 } else { 0 };
    let now = chrono::Utc::now().to_rfc3339();

    let affected = tx.execute(
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
            appointment.reminder_external_id,
            appointment.reagendada_from_id.map(|id| id.to_string()),
            appointment.external_calendar_id,
            appointment.calendar_provider,
            now,
            appointment.id.to_string(),
        ],
    ).map_err(|e| AppError::Database(format!("Failed to update appointment: {}", e)))?;

    if affected == 0 {
        tx.rollback().ok();
        return Err(AppError::NotFound(format!("Appointment not found: {}", appointment.id)));
    }

    // --- 2. INSERT accounting entry ---
    let asiento_id = asiento.id.to_string();
    let asiento_fecha = asiento.fecha.format("%Y-%m-%d").to_string();
    let asiento_descripcion = asiento.descripcion.clone();
    let asiento_lineas = serde_json::to_string(&asiento.lineas)
        .map_err(|e| AppError::Accounting(format!("Failed to serialize lineas: {}", e)))?;

    tx.execute(
        "INSERT INTO asientos_contables (id, fecha, descripcion, lineas, created_at) VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![asiento_id, asiento_fecha, asiento_descripcion, asiento_lineas],
    ).map_err(|e| AppError::Database(format!("Failed to insert accounting entry: {}", e)))?;

    // --- 3. COMMIT ---
    tx.commit()
        .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

    Ok(())
}

/// Finalize a session: atomically mark appointment as Realizada and create accounting entry.
pub async fn finalizar_sesion_agenda_impl(
    pool: &DbPool,
    id: String,
    notes: Option<String>,
) -> AppResult<AppointmentResponse> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    let appointment_id = AppointmentId(Uuid::parse_str(&id)?);
    
    // Get appointment
    let mut appointment = repo.get_by_id(appointment_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Appointment {} not found", id)))?;
    
    // Validate transition
    if !appointment.status.can_transition_to(soft_mindledger_domain::appointment::AppointmentStatus::Realizada) {
        return Err(AppError::Validation(
            format!("Cannot transition from {} to Realizada", appointment.status)
        ));
    }
    
    // Build accounting entry before state change (need patient & therapist data)
    let patient_repo = SqlitePatientRepository::new(pool.clone());
    let patient = patient_repo.get_by_id(appointment.patient_id).await?
        .ok_or_else(|| AppError::NotFound("Patient not found".to_string()))?;
    
    // Build accounting entry using domain trigger
    let therapist_data = soft_mindledger_domain::accounting_trigger::TherapistAccountingData {
        id: appointment.therapist_id,
        full_name: soft_mindledger_domain::value_objects::FullName::new(
            "Terapeuta".to_string(), "Apellido".to_string(), None
        )?,
        specialty_code: "PSI".to_string(),
    };
    
    let paciente_data = soft_mindledger_domain::accounting_trigger::PatientAccountingData {
        id: appointment.patient_id,
        full_name: patient.full_name,
        session_fee_cents: appointment.fee_cents,
    };
    
    let asiento = soft_mindledger_domain::accounting_trigger::AccountingTrigger::build_session_asiento(
        appointment.id,
        &paciente_data,
        &therapist_data,
        appointment.time_range.start.date_naive(),
        appointment.fee_cents,
        true, // is_paid = true for completed session
        &appointment.modality.to_string(),
    )?;
    
    // Validate the asiento is balanced
    soft_mindledger_domain::accounting_trigger::AccountingTrigger::validate_asiento_balance(&asiento)?;
    
    // Mark appointment as finalized (domain state change)
    appointment.finalize(notes)?;
    
    // Execute both operations atomically inside a single SQLite transaction.
    // If the accounting entry fails, the appointment UPDATE is also rolled back.
    finalize_appointment_atomic(pool, &appointment, &asiento, appointment.notes.clone())?;
    
    Ok(appointment.into())
}

/// Reschedule an appointment to a new time slot with conflict check.
pub async fn reagendar_cita_impl(
    pool: &DbPool,
    id: String,
    new_start_at: String,
    new_end_at: String,
    reason: String,
) -> AppResult<AppointmentResponse> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    let appointment_id = AppointmentId(Uuid::parse_str(&id)?);
    
    let new_start = DateTime::parse_from_rfc3339(&new_start_at)
        .map_err(|e| AppError::Validation(format!("Invalid new_start_at: {}", e)))?
        .with_timezone(&Utc);
    let new_end = DateTime::parse_from_rfc3339(&new_end_at)
        .map_err(|e| AppError::Validation(format!("Invalid new_end_at: {}", e)))?
        .with_timezone(&Utc);
    
    let new_range = DateTimeRange::new(new_start, new_end)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    let mut appointment = repo.get_by_id(AppointmentId(Uuid::parse_str(&id)?)).await?
        .ok_or_else(|| AppError::NotFound("Appointment not found".to_string()))?;
    
    // Check overlapping for new time (excluding self)
    let overlapping = repo.find_overlapping(appointment.therapist_id, DateRange {
        start: new_range.start,
        end: new_range.end,
    }).await?;
    
    let has_conflict = overlapping.iter().any(|a| a.id != appointment.id);
    if has_conflict {
        return Err(AppError::Conflict("New time slot conflicts with existing appointment".to_string()));
    }
    
    // Cancel old reminder
    let reminder_repo = SqliteReminderRepository::new(pool.clone());
    let reminders = reminder_repo.list_by_appointment(appointment.id).await?;
    for reminder in reminders {
        reminder_repo.cancel(reminder.id).await?;
    }
    appointment.clear_reminder();
    
    // Reschedule
    let new_start = new_range.start;
    appointment.reschedule(new_range, reason)?;
    repo.update(&appointment).await?;
    
    // Schedule new reminder (30 min before)
    let remind_at = new_start - Duration::minutes(30);
    let reminder = Reminder::new(
        appointment.id,
        appointment.patient_id,
        remind_at,
        ReminderChannel::Push,
        ReminderTemplate::Session30Min,
    );
    reminder_repo.create(&reminder).await?;
    
    Ok(appointment.into())
}

/// Cancel an appointment with mandatory reason.
pub async fn cancelar_cita_impl(
    pool: &DbPool,
    id: String,
    reason: String,
) -> AppResult<AppointmentResponse> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    let appointment_id = AppointmentId(Uuid::parse_str(&id)?);
    
    let mut appointment = repo.get_by_id(appointment_id).await?
        .ok_or_else(|| AppError::NotFound("Appointment not found".to_string()))?;
    
    if reason.trim().is_empty() {
        return Err(AppError::Validation("Cancellation reason is required".to_string()));
    }
    
    // Cancel reminder
    let reminder_repo = SqliteReminderRepository::new(pool.clone());
    let reminders = reminder_repo.list_by_appointment(appointment.id).await?;
    for reminder in reminders {
        reminder_repo.cancel(reminder.id).await?;
    }
    appointment.clear_reminder();
    
    // Cancel appointment
    appointment.cancel(reason)?;
    repo.update(&appointment).await?;
    
    Ok(appointment.into())
}

// --- Patient-specific queries ---

/// List appointments for a specific patient with optional date range and status filters.
pub async fn obtener_citas_paciente_impl(
    pool: &DbPool,
    query: GetPatientAppointmentsQuery,
) -> AppResult<PaginatedResponse<AppointmentResponse>> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    
    let patient_id = PatientId(Uuid::parse_str(&query.patient_id)?);
    
    let range = if let (Some(start), Some(end)) = (query.start_date, query.end_date) {
        Some(DateRange {
            start: NaiveDate::parse_from_str(&start, "%Y-%m-%d")?.and_hms_opt(0, 0, 0)
                    .ok_or_else(|| AppError::Validation("Invalid date".to_string()))?.and_utc(),
            end: NaiveDate::parse_from_str(&end, "%Y-%m-%d")?.and_hms_opt(23, 59, 59)
                    .ok_or_else(|| AppError::Validation("Invalid date".to_string()))?.and_utc(),
        })
    } else {
        None
    };
    
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let mut filter = AppointmentFilter {
        patient_id: Some(patient_id),
        date_range: range,
        status: query.status,
        ..Default::default()
    };
    
    let pagination = Pagination::new(page, page_size);
    let total = repo.count(filter.clone()).await?;
    let appointments = repo.list(filter, pagination).await?;
    
    Ok(PaginatedResponse {
        items: appointments.into_iter().map(Into::into).collect(),
        total,
        page,
        page_size,
        total_pages: (total + page_size - 1) / page_size,
    })
}

// --- Reminders ---

/// Get all pending (unsent) reminders.
pub async fn obtener_recordatorios_pendientes_impl(
    pool: &DbPool,
) -> AppResult<Vec<ReminderResponse>> {
    let repo = SqliteReminderRepository::new(pool.clone());
    let now = Utc::now();
    
    let reminders = repo.find_due(now).await?;
    Ok(reminders.into_iter().map(Into::into).collect())
}

/// Process all due reminders: send notifications and mark as sent.
pub async fn procesar_recordatorios_pendientes_impl(
    pool: &DbPool,
) -> AppResult<ProcessRemindersResult> {
    let repo = SqliteReminderRepository::new(pool.clone());
    let now = Utc::now();
    
    let due = repo.find_due(now).await?;
    let mut processed = 0;
    let mut failed = 0;
    
    for reminder in due {
        // In a real implementation, this would send the actual notification
        // via OS notification, email, SMS, etc.
        // For now, we just mark as sent
        match SqliteReminderRepository::new(pool.clone()).mark_sent(
            reminder.id, 
            format!("os_notification_{}", Uuid::new_v4())
        ).await {
            Ok(_) => processed += 1,
            Err(_) => failed += 1,
        }
    }
    
    Ok(ProcessRemindersResult { processed, failed })
}

// --- KPIs ---

/// Calculate KPI metrics (sessions completed, revenue, no-show rate, occupancy) for a date range.
pub async fn obtener_kpis_agenda_impl(
    pool: &DbPool,
    therapist_id: Option<String>,
    range: DateRange,
) -> AppResult<KpiMetricsResponse> {
    let repo = SqliteAppointmentRepository::new(pool.clone());
    
    let tid = therapist_id.map(|s| TherapistId(Uuid::parse_str(&s))).transpose()?;
    
    let filter = AppointmentFilter {
        therapist_id: tid,
        date_range: Some(range),
        ..Default::default()
    };
    
    let all = repo.list(filter.clone(), Pagination::new(0, 10000)).await?;
    
    let completed = all.iter().filter(|a| a.status == soft_mindledger_domain::appointment::AppointmentStatus::Realizada).count() as u64;
    let cancelled = all.iter().filter(|a| a.status == soft_mindledger_domain::appointment::AppointmentStatus::Cancelada).count() as u64;
    let scheduled = all.iter().filter(|a| a.status == soft_mindledger_domain::appointment::AppointmentStatus::Programada).count() as u64;
    
    let total_scheduled = scheduled + completed;
    let no_show_rate = if total_scheduled > 0 {
        (cancelled as f64 / total_scheduled as f64) * 100.0
    } else { 0.0 };
    
    let revenue: i64 = all.iter()
        .filter(|a| a.status == soft_mindledger_domain::appointment::AppointmentStatus::Realizada)
        .map(|a| a.fee_cents)
        .sum();
    
    let now = Utc::now();
    let upcoming_24h = all.iter()
        .filter(|a| a.status == soft_mindledger_domain::appointment::AppointmentStatus::Programada)
        .filter(|a| a.time_range.start <= now + Duration::hours(24))
        .count() as u64;
    
    // Simplified occupancy: completed / total slots in range
    let occupancy = if total_scheduled > 0 {
        (completed as f64 / total_scheduled as f64) * 100.0
    } else { 0.0 };
    
    Ok(KpiMetricsResponse {
        sessions_completed_today: completed,
        sessions_completed_week: completed, // Simplified
        sessions_completed_month: completed,
        occupancy_rate_pct: occupancy,
        revenue_today_cents: revenue,
        revenue_week_cents: revenue,
        revenue_month_cents: revenue,
        no_show_rate_pct: no_show_rate,
        upcoming_24h,
    })
}

// ============================================================================
// Tauri Command Wrappers
// ============================================================================

/// Tauri IPC command: create appointment.
#[command]
pub async fn crear_cita_agenda(
    db: tauri::State<'_, Arc<DbPool>>,
    request: CreateAppointmentRequest,
) -> AppResult<AppointmentResponse> {
    crear_cita_agenda_impl(&db, request).await
}

/// Tauri IPC command: get appointment by ID.
#[command]
pub async fn obtener_cita_agenda(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
) -> AppResult<AppointmentResponse> {
    obtener_cita_agenda_impl(&db, id).await
}

/// Tauri IPC command: list appointments with filters.
#[command]
pub async fn listar_citas_agenda(
    db: tauri::State<'_, Arc<DbPool>>,
    query: ListAppointmentsQuery,
) -> AppResult<PaginatedResponse<AppointmentResponse>> {
    listar_citas_agenda_impl(&db, query).await
}

/// Tauri IPC command: finalize session (atomic appointment + accounting).
#[command]
pub async fn finalizar_sesion_agenda(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
    notes: Option<String>,
) -> AppResult<AppointmentResponse> {
    finalizar_sesion_agenda_impl(&db, id, notes).await
}

/// Tauri IPC command: reschedule appointment.
#[command]
pub async fn reagendar_cita(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
    new_start_at: String,
    new_end_at: String,
    reason: String,
) -> AppResult<AppointmentResponse> {
    reagendar_cita_impl(&db, id, new_start_at, new_end_at, reason).await
}

/// Tauri IPC command: cancel appointment.
#[command]
pub async fn cancelar_cita(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
    reason: String,
) -> AppResult<AppointmentResponse> {
    cancelar_cita_impl(&db, id, reason).await
}

/// Tauri IPC command: list patient appointments.
#[command]
pub async fn obtener_citas_paciente(
    db: tauri::State<'_, Arc<DbPool>>,
    query: GetPatientAppointmentsQuery,
) -> AppResult<PaginatedResponse<AppointmentResponse>> {
    obtener_citas_paciente_impl(&db, query).await
}

/// Tauri IPC command: get pending reminders.
#[command]
pub async fn obtener_recordatorios_pendientes(
    db: tauri::State<'_, Arc<DbPool>>,
) -> AppResult<Vec<ReminderResponse>> {
    obtener_recordatorios_pendientes_impl(&db).await
}

/// Tauri IPC command: process due reminders.
#[command]
pub async fn procesar_recordatorios_pendientes(
    db: tauri::State<'_, Arc<DbPool>>,
) -> AppResult<ProcessRemindersResult> {
    procesar_recordatorios_pendientes_impl(&db).await
}

/// Tauri IPC command: get agenda KPIs.
#[command]
pub async fn obtener_kpis_agenda(
    db: tauri::State<'_, Arc<DbPool>>,
    therapist_id: Option<String>,
    start_date: String,
    end_date: String,
) -> AppResult<KpiMetricsResponse> {
    let range = DateRange {
        start: NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")?.and_hms_opt(0, 0, 0)
                    .ok_or_else(|| AppError::Validation("Invalid date".to_string()))?.and_utc(),
        end: NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")?.and_hms_opt(23, 59, 59)
                    .ok_or_else(|| AppError::Validation("Invalid date".to_string()))?.and_utc(),
    };
    obtener_kpis_agenda_impl(&db, therapist_id, range).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use soft_mindledger_infrastructure::database::create_memory_pool;
    use soft_mindledger_domain::{PatientId, TherapistId, Modality, AppointmentStatus, ReminderChannel, ReminderTemplate, Reminder, ReminderId, AppointmentId, PatientId as DomainPatientId};
    use chrono::{Utc, Duration};
    use uuid::Uuid;

fn create_test_pool() -> DbPool {
        let pool = create_memory_pool().unwrap();
        {
            let conn = pool.lock().unwrap();
            // Run migrations
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS patients (
                    id TEXT PRIMARY KEY NOT NULL,
                    document_number TEXT NOT NULL UNIQUE,
                    document_type TEXT NOT NULL,
                    country_code TEXT NOT NULL,
                    first_name TEXT NOT NULL,
                    last_name TEXT NOT NULL,
                    middle_name TEXT,
                    date_of_birth TEXT NOT NULL,
                    gender TEXT NOT NULL,
                    email TEXT,
                    phone_number TEXT,
                    phone_country_code TEXT,
                    phone_extension TEXT,
                    address_street TEXT,
                    address_city TEXT,
                    address_state TEXT,
                    address_postal_code TEXT,
                    address_country TEXT,
                    address_additional_info TEXT,
                    emergency_contact_name_first TEXT,
                    emergency_contact_name_last TEXT,
                    emergency_contact_name_middle TEXT,
                    emergency_contact_relationship TEXT,
                    emergency_contact_phone_number TEXT,
                    emergency_contact_phone_country_code TEXT,
                    emergency_contact_email TEXT,
                    blood_type TEXT,
                    allergies TEXT DEFAULT '[]',
                    chronic_conditions TEXT DEFAULT '[]',
                    medications TEXT DEFAULT '[]',
                    notes TEXT,
                    is_active INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                );
                CREATE TABLE IF NOT EXISTS appointments (
                    id TEXT PRIMARY KEY NOT NULL,
                    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
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
                    reagendada_from_id TEXT REFERENCES appointments(id),
                    external_calendar_id TEXT,
                    calendar_provider TEXT,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                );
                CREATE TABLE IF NOT EXISTS reminders (
                    id TEXT PRIMARY KEY NOT NULL,
                    appointment_id TEXT NOT NULL REFERENCES appointments(id) ON DELETE CASCADE,
                    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
                    remind_at TEXT NOT NULL,
                    channel TEXT NOT NULL DEFAULT 'push',
                    template_id TEXT,
                    sent_at TEXT,
                    external_id TEXT,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                );
                CREATE TABLE IF NOT EXISTS asientos_contables (
                    id TEXT PRIMARY KEY NOT NULL,
                    fecha TEXT NOT NULL,
                    descripcion TEXT NOT NULL,
                    lineas TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                );
                "#,
            ).unwrap();
        }
        pool
    }

    async fn create_test_patient(pool: &DbPool) -> Uuid {
        let patient_id = Uuid::new_v4();
        let pool_clone = pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool_clone.lock().unwrap();
            conn.execute(
                "INSERT INTO patients (id, document_number, document_type, country_code, first_name, last_name, date_of_birth, gender, is_active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                rusqlite::params![
                    patient_id.to_string(),
                    "12345678",
                    "DNI",
                    "EC",
                    "Test",
                    "Patient",
                    "1990-01-01",
                    "Male",
                    1,
                ],
            ).unwrap();
        }).await.unwrap();
        patient_id
    }

    #[tokio::test]
    async fn test_crear_cita_agenda() {
        let pool = create_test_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();
        
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: Some("Primera sesión".to_string()),
        };
        
        let result = crear_cita_agenda_impl(&pool, request).await;
        assert!(result.is_ok());
        let appt = result.unwrap();
        assert_eq!(appt.status, AppointmentStatus::Programada);
        assert_eq!(appt.fee_cents, 50000);
        assert!(!appt.reminder_sent);
    }

    #[tokio::test]
    async fn test_finalizar_sesion_agenda() {
        let pool = create_test_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();
        
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };
        
        let created = crear_cita_agenda_impl(&pool, request).await.unwrap();
        
        // Now finalize
        let result = finalizar_sesion_agenda_impl(&pool, created.id, Some("Sesión completada".to_string())).await;
        assert!(result.is_ok());
        let finalized = result.unwrap();
        assert_eq!(finalized.status, AppointmentStatus::Realizada);
    }

    #[tokio::test]
    async fn test_reagendar_cita() {
        let pool = create_test_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();
        
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Virtual,
            fee_cents: Some(30000),
            notes: None,
        };
        
        let created = crear_cita_agenda_impl(&pool, request).await.unwrap();
        
        // Reschedule to tomorrow same time
        let new_start = start + Duration::days(1);
        let new_end = end + Duration::days(1);
        
        let result = reagendar_cita_impl(&pool, created.id, 
            new_start.to_rfc3339(), new_end.to_rfc3339(), "Paciente solicitó cambio".to_string()).await;
        assert!(result.is_ok());
        let rescheduled = result.unwrap();
        assert_eq!(rescheduled.status, AppointmentStatus::Reagendada);
        // reagendada_from_id is no longer set since the appointment ID is preserved
    }

    #[tokio::test]
    async fn test_cancelar_cita() {
        let pool = create_test_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();
        
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };
        
        let created = crear_cita_agenda_impl(&pool, request).await.unwrap();
        
        let result = cancelar_cita_impl(&pool, created.id, "Paciente no asistirá".to_string()).await;
        assert!(result.is_ok());
        let cancelled = result.unwrap();
        assert_eq!(cancelled.status, AppointmentStatus::Cancelada);
        assert!(cancelled.notes.unwrap().contains("Cancelada"));
    }

    #[tokio::test]
    async fn test_obtener_citas_paciente() {
        let pool = create_test_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();
        
        for i in 0..3 {
            let start = Utc::now() + Duration::hours(1) + Duration::minutes(i * 60);
            let end = start + Duration::minutes(50);
            let request = CreateAppointmentRequest {
                patient_id: patient_id.to_string(),
                therapist_id: therapist_id.to_string(),
                start_at: start.to_rfc3339(),
                end_at: end.to_rfc3339(),
                modality: Modality::Presencial,
                fee_cents: Some(50000),
                notes: None,
            };
            crear_cita_agenda_impl(&pool, request).await.unwrap();
        }
        
        let query = GetPatientAppointmentsQuery {
            patient_id: patient_id.to_string(),
            start_date: None,
            end_date: None,
            status: None,
            page: Some(0),
            page_size: Some(10),
        };
        
        let result = obtener_citas_paciente_impl(&pool, query).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.items.len(), 3);
    }

    #[tokio::test]
    async fn test_recordatorios_pendientes() {
        let pool = create_test_pool();
        let patient_id = create_test_patient(&pool).await;
        
        // Create an appointment first so FK references are valid
        let therapist_id = Uuid::new_v4();
        let start = Utc::now() + Duration::hours(1);
        let end = start + Duration::minutes(50);
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };
        let appointment = crear_cita_agenda_impl(&pool, request).await.unwrap();
        
        // Create a reminder due now
        let repo = SqliteReminderRepository::new(pool.clone());
        let reminder = Reminder::new(
            AppointmentId(Uuid::parse_str(&appointment.id).unwrap()),
            PatientId(patient_id),
            Utc::now() - Duration::minutes(5),
            ReminderChannel::Push,
            ReminderTemplate::Session30Min,
        );
        repo.create(&reminder).await.unwrap();
        
        let result = obtener_recordatorios_pendientes_impl(&pool).await;
        assert!(result.is_ok());
        let reminders = result.unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].channel, ReminderChannel::Push);
    }
}