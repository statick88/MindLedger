use crate::database::DbPool;
use anyhow::Result;

pub const MIGRATIONS: &str = include_str!("../migrations.sql");

pub const ACCOUNTING_MIGRATIONS: &str = include_str!("../accounting_migrations.sql");

pub const DIAGNOSTICS_MIGRATIONS: &str = include_str!("../diagnostics_migrations.sql");

pub fn run_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    for statement in MIGRATIONS.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() && !statement.starts_with("--") {
            conn.execute_batch(statement)?;
        }
    }
    Ok(())
}

pub fn run_accounting_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    for statement in ACCOUNTING_MIGRATIONS.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() && !statement.starts_with("--") {
            conn.execute_batch(statement)?;
        }
    }
    Ok(())
}

pub fn run_diagnostics_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    for statement in DIAGNOSTICS_MIGRATIONS.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() && !statement.starts_with("--") {
            conn.execute_batch(statement)?;
        }
    }
    Ok(())
}

pub fn run_all_migrations(pool: &DbPool) -> Result<()> {
    run_migrations(pool)?;
    run_accounting_migrations(pool)?;
    run_diagnostics_migrations(pool)?;
    Ok(())
}
