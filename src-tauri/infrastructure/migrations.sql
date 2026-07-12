-- Soft Gloria Database Schema
-- Version: 1.0.0
-- Description: Initial schema for patient registry with SQLCipher support

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Patients table
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
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_patients_document ON patients(document_number);
CREATE INDEX IF NOT EXISTS idx_patients_name ON patients(last_name, first_name);
CREATE INDEX IF NOT EXISTS idx_patients_dob ON patients(date_of_birth);
CREATE INDEX IF NOT EXISTS idx_patients_active ON patients(is_active);

-- Appointments table
CREATE TABLE IF NOT EXISTS appointments (
    id TEXT PRIMARY KEY NOT NULL,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
    appointment_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Scheduled',
    scheduled_date TEXT NOT NULL,
    scheduled_time TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL DEFAULT 30,
    reason TEXT NOT NULL,
    notes TEXT,
    doctor_name TEXT NOT NULL,
    room TEXT,
    completed_at TEXT,
    cancelled_at TEXT,
    cancellation_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_appointments_patient ON appointments(patient_id);
CREATE INDEX IF NOT EXISTS idx_appointments_date ON appointments(scheduled_date);
CREATE INDEX IF NOT EXISTS idx_appointments_status ON appointments(status);
CREATE INDEX IF NOT EXISTS idx_appointments_doctor_date ON appointments(doctor_name, scheduled_date);

-- Clinical Notes table
CREATE TABLE IF NOT EXISTS clinical_notes (
    id TEXT PRIMARY KEY NOT NULL,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
    appointment_id TEXT REFERENCES appointments(id) ON DELETE SET NULL,
    note_type TEXT NOT NULL,
    chief_complaint TEXT NOT NULL,
    history_of_present_illness TEXT NOT NULL,
    physical_examination TEXT NOT NULL,
    assessment TEXT NOT NULL,
    plan TEXT NOT NULL,
    diagnoses TEXT DEFAULT '[]',
    vital_signs TEXT,
    attachments TEXT DEFAULT '[]',
    is_signed INTEGER NOT NULL DEFAULT 0,
    signed_at TEXT,
    signed_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_clinical_notes_patient ON clinical_notes(patient_id);
CREATE INDEX IF NOT EXISTS idx_clinical_notes_appointment ON clinical_notes(appointment_id);
CREATE INDEX IF NOT EXISTS idx_clinical_notes_type ON clinical_notes(note_type);
CREATE INDEX IF NOT EXISTS idx_clinical_notes_signed ON clinical_notes(is_signed);

-- Audit Log table (append-only)
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    old_values TEXT,
    new_values TEXT,
    user_id TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default settings
INSERT OR IGNORE INTO settings (key, value, description) VALUES 
    ('clinic_name', 'Soft Gloria Medical Center', 'Name of the clinic'),
    ('clinic_address', '', 'Clinic physical address'),
    ('clinic_phone', '', 'Clinic phone number'),
    ('clinic_email', '', 'Clinic email address'),
    ('timezone', 'America/Argentina/Buenos_Aires', 'Clinic timezone'),
    ('appointment_duration_default', '30', 'Default appointment duration in minutes'),
    ('age_of_majority', '18', 'Age of majority for consent'),
    ('currency', 'ARS', 'Default currency code'),
    ('language', 'es-AR', 'Default language');

-- Trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS trigger_patients_updated_at
AFTER UPDATE ON patients
BEGIN
    UPDATE patients SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trigger_appointments_updated_at
AFTER UPDATE ON appointments
BEGIN
    UPDATE appointments SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trigger_clinical_notes_updated_at
AFTER UPDATE ON clinical_notes
BEGIN
    UPDATE clinical_notes SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Prevent updates/deletes on audit_log
CREATE TRIGGER IF NOT EXISTS trigger_audit_log_prevent_update
BEFORE UPDATE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log is append-only');
END;

CREATE TRIGGER IF NOT EXISTS trigger_audit_log_prevent_delete
BEFORE DELETE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log is append-only');
END;

-- Views for common queries
CREATE VIEW IF NOT EXISTS v_active_patients AS
SELECT 
    id,
    document_number,
    document_type,
    country_code,
    first_name,
    last_name,
    middle_name,
    date_of_birth,
    gender,
    email,
    phone_number,
    is_active,
    created_at
FROM patients
WHERE is_active = 1;

CREATE VIEW IF NOT EXISTS v_upcoming_appointments AS
SELECT 
    a.id,
    a.patient_id,
    p.first_name || ' ' || p.last_name AS patient_name,
    p.document_number,
    a.appointment_type,
    a.status,
    a.scheduled_date,
    a.scheduled_time,
    a.duration_minutes,
    a.reason,
    a.doctor_name,
    a.room
FROM appointments a
JOIN patients p ON a.patient_id = p.id
WHERE a.scheduled_date >= date('now')
    AND a.status IN ('Scheduled', 'Confirmed')
ORDER BY a.scheduled_date, a.scheduled_time;

CREATE VIEW IF NOT EXISTS v_patient_age AS
SELECT 
    id,
    date_of_birth,
    CAST((julianday('now') - julianday(date_of_birth)) / 365.25 AS INTEGER) AS age_years
FROM patients;