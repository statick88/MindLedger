use crate::database::DbPool;
use anyhow::Result;

pub const MIGRATIONS: &str = include_str!("../migrations.sql");

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
