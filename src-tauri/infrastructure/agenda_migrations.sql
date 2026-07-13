-- Agenda Module Migrations
-- Tables for appointments, reminders, and calendar sync
-- Extends the existing appointments table from migrations.sql

PRAGMA foreign_keys = ON;

-- ============================================================================
-- STEP 1: Add new columns to existing appointments table
-- ============================================================================
-- Note: SQLite doesn't support IF NOT EXISTS for ADD COLUMN.
-- On fresh databases (tests), these will succeed.
-- On existing databases, you'd need a proper migration tool or manual execution.

-- Add status column with new enum values (if not already present with correct CHECK)
-- The existing status column has different values ('Scheduled', etc.), we need to migrate it
-- For now, we add a new column with the correct constraints
-- ALTER TABLE appointments ADD COLUMN status TEXT NOT NULL DEFAULT 'Programada' CHECK (status IN ('Programada','Realizada','Reagendada','Cancelada'));

-- Add fee_cents for session pricing
ALTER TABLE appointments ADD COLUMN fee_cents INTEGER NOT NULL DEFAULT 0;

-- Add reminder tracking columns
ALTER TABLE appointments ADD COLUMN reminder_sent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE appointments ADD COLUMN reminder_external_id TEXT;
ALTER TABLE appointments ADD COLUMN reagendada_from_id TEXT REFERENCES appointments(id);
ALTER TABLE appointments ADD COLUMN external_calendar_id TEXT;
ALTER TABLE appointments ADD COLUMN calendar_provider TEXT;

-- Add professional_id for multi-therapist support
ALTER TABLE appointments ADD COLUMN professional_id TEXT NOT NULL DEFAULT '' REFERENCES patients(id);

-- ============================================================================
-- STEP 2: Create new tables
-- ============================================================================

-- Reminders table
CREATE TABLE IF NOT EXISTS reminders (
    id TEXT PRIMARY KEY NOT NULL,
    appointment_id TEXT NOT NULL REFERENCES appointments(id) ON DELETE CASCADE,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
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

-- Calendar sync tokens table
CREATE TABLE IF NOT EXISTS calendar_sync_tokens (
    provider TEXT NOT NULL,
    calendar_id TEXT NOT NULL,
    token TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (provider, calendar_id)
);

-- ============================================================================
-- STEP 3: Create indexes on appointments table (after columns exist)
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_appointments_status ON appointments(status);
CREATE INDEX IF NOT EXISTS idx_appointments_reminder_due ON appointments(scheduled_date) WHERE reminder_sent = 0;
CREATE INDEX IF NOT EXISTS idx_appointments_professional_date ON appointments(professional_id, scheduled_date);
CREATE INDEX IF NOT EXISTS idx_appointments_reagendada_from ON appointments(reagendada_from_id);
CREATE INDEX IF NOT EXISTS idx_appointments_calendar_provider ON appointments(calendar_provider);

-- ============================================================================
-- STEP 4: Create triggers
-- ============================================================================

-- Trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS trigger_reminders_updated_at
AFTER UPDATE ON reminders
BEGIN
    UPDATE reminders SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Appointment status transition validation trigger
CREATE TRIGGER IF NOT EXISTS trigger_appointment_status_transition
BEFORE UPDATE OF status ON appointments
BEGIN
    SELECT RAISE(ABORT, 'Invalid status transition')
    WHERE NOT (
        (OLD.status = 'Programada' AND NEW.status IN ('Realizada', 'Reagendada', 'Cancelada'))
        OR (OLD.status = 'Reagendada' AND NEW.status IN ('Realizada', 'Cancelada'))
        -- Terminal states cannot transition
    )
    AND NEW.status != OLD.status;
END;

-- Trigger to mark reminder as sent when appointment is finalized
CREATE TRIGGER IF NOT EXISTS trigger_appointment_finalized_cancel_reminder
AFTER UPDATE OF status ON appointments
BEGIN
    UPDATE reminders 
    SET sent_at = datetime('now'), external_id = 'cancelled'
    WHERE appointment_id = NEW.id 
      AND sent_at IS NULL
      AND NEW.status IN ('Realizada', 'Cancelada');
END;

-- Trigger for reagendada: cancel old reminder, schedule new
CREATE TRIGGER IF NOT EXISTS trigger_appointment_rescheduled
AFTER UPDATE OF scheduled_date, scheduled_time ON appointments
BEGIN
    UPDATE reminders 
    SET sent_at = datetime('now'), external_id = 'rescheduled'
    WHERE appointment_id = OLD.id 
      AND sent_at IS NULL;
END;

-- Trigger to update appointments updated_at
CREATE TRIGGER IF NOT EXISTS trigger_appointments_updated_at
AFTER UPDATE ON appointments
BEGIN
    UPDATE appointments SET updated_at = datetime('now') WHERE id = NEW.id;
END;