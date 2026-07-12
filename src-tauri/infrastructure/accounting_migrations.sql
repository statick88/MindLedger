-- Accounting Module Migrations
-- Tables for double-entry bookkeeping

PRAGMA foreign_keys = ON;

-- Asientos Contables (Journal Entries)
CREATE TABLE IF NOT EXISTS asientos_contables (
    id TEXT PRIMARY KEY NOT NULL,
    fecha TEXT NOT NULL,
    descripcion TEXT NOT NULL,
    lineas TEXT NOT NULL, -- JSON array of LineaAsiento
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_asientos_fecha ON asientos_contables(fecha);
CREATE INDEX IF NOT EXISTS idx_asientos_created ON asientos_contables(created_at);

-- Trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS trigger_asientos_updated_at
AFTER UPDATE ON asientos_contables
BEGIN
    UPDATE asientos_contables SET updated_at = datetime('now') WHERE id = NEW.id;
END;