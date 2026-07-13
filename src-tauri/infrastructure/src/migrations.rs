use crate::database::DbPool;
use anyhow::Result;

pub const MIGRATIONS: &str = include_str!("../migrations.sql");

pub const ACCOUNTING_MIGRATIONS: &str = include_str!("../accounting_migrations.sql");

pub const DIAGNOSTICS_MIGRATIONS: &str = include_str!("../diagnostics_migrations.sql");

pub const AGENDA_MIGRATIONS: &str = include_str!("../agenda_migrations.sql");

pub fn run_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    // Use execute_batch directly — SQLite handles multi-statement SQL natively.
    // The old split(';') approach broke trigger creation (BEGIN...END; blocks).
    conn.execute_batch(MIGRATIONS)?;
    Ok(())
}

pub fn run_accounting_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(ACCOUNTING_MIGRATIONS)?;
    Ok(())
}

pub fn run_diagnostics_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(DIAGNOSTICS_MIGRATIONS)?;
    Ok(())
}

pub fn run_agenda_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(AGENDA_MIGRATIONS)?;
    Ok(())
}

pub fn run_all_migrations(pool: &DbPool) -> Result<()> {
    run_migrations(pool)?;
    run_accounting_migrations(pool)?;
    run_diagnostics_migrations(pool)?;
    run_agenda_migrations(pool)?;
    Ok(())
}
