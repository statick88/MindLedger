use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn create_pool(database_path: &Path) -> Result<DbPool, rusqlite::Error> {
    let conn = Connection::open(database_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn create_memory_pool() -> Result<DbPool, rusqlite::Error> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    Ok(Arc::new(Mutex::new(conn)))
}
