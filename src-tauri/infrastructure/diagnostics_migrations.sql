-- Diagnostics Module Migrations
-- Tables for CIE-10, DSM-5, and diagnostic mappings

PRAGMA foreign_keys = ON;

-- CIE-10 Diagnoses
CREATE TABLE IF NOT EXISTS cie10 (
    codigo TEXT PRIMARY KEY NOT NULL,
    descripcion TEXT NOT NULL,
    categoria TEXT NOT NULL,
    subcategoria TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_cie10_categoria ON cie10(categoria);
CREATE INDEX IF NOT EXISTS idx_cie10_descripcion ON cie10(descripcion);

-- DSM-5 Diagnoses
CREATE TABLE IF NOT EXISTS dsm5 (
    codigo TEXT PRIMARY KEY NOT NULL,
    descripcion TEXT NOT NULL,
    categoria TEXT NOT NULL,
    criterios_diagnosticos TEXT, -- JSON array of strings
    especificadores TEXT, -- JSON array of strings
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_dsm5_categoria ON dsm5(categoria);
CREATE INDEX IF NOT EXISTS idx_dsm5_descripcion ON dsm5(descripcion);

-- Mapeo de Diagnósticos (Patient-Diagnosis mapping)
CREATE TABLE IF NOT EXISTS mapeos_diagnosticos (
    id TEXT PRIMARY KEY NOT NULL,
    paciente_id TEXT NOT NULL,
    diagnostico_id TEXT NOT NULL,
    fuente TEXT NOT NULL, -- "CIE-10" or "DSM-5"
    notas TEXT,
    fecha TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (paciente_id) REFERENCES patients(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_mapeos_paciente ON mapeos_diagnosticos(paciente_id);
CREATE INDEX IF NOT EXISTS idx_mapeos_diagnostico ON mapeos_diagnosticos(diagnostico_id);
CREATE INDEX IF NOT EXISTS idx_mapeos_fuente ON mapeos_diagnosticos(fuente);
CREATE INDEX IF NOT EXISTS idx_mapeos_fecha ON mapeos_diagnosticos(fecha);

-- Trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS trigger_mapeos_updated_at
AFTER UPDATE ON mapeos_diagnosticos
BEGIN
    UPDATE mapeos_diagnosticos SET updated_at = datetime('now') WHERE id = NEW.id;
END;