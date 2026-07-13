use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::keyring::SqlCipherKeyManager;

pub type DbPool = Arc<Mutex<Connection>>;

const DEFAULT_SERVICE_NAME: &str = "mind-ledger";
const DEFAULT_ACCOUNT_NAME: &str = "sqlcipher-key";

/// Create a connection pool using the keyring (or file fallback) for the encryption key.
/// `data_dir` is the app data directory — used for the file-based key fallback.
pub fn create_pool(database_path: &Path, data_dir: &Path) -> Result<DbPool> {
    let key_manager = SqlCipherKeyManager::new_with_fallback(
        DEFAULT_SERVICE_NAME,
        DEFAULT_ACCOUNT_NAME,
        data_dir,
    );
    let key = key_manager.get_or_create_key()?;
    create_pool_with_key(database_path, &key)
}

pub fn create_pool_with_key(database_path: &Path, key: &str) -> Result<DbPool> {
    let conn = if database_path.to_string_lossy() == ":memory:" {
        Connection::open_in_memory()
            .context("Failed to open in-memory database")?
    } else {
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }
        Connection::open(database_path)
            .context("Failed to open database")?
    };

    let pragma_key = format!("PRAGMA key = '{}';", key);
    conn.execute_batch(&pragma_key)
        .context("Failed to set encryption key")?;

    // Set journal mode — try WAL first, fall back to DELETE (SQLCipher may not support WAL).
    if let Err(e) = conn.execute_batch("PRAGMA journal_mode=WAL;") {
        eprintln!("[MindLedger] WAL mode unavailable ({}), using DELETE", e);
        conn.execute_batch("PRAGMA journal_mode=DELETE;")
            .context("Failed to set journal mode")?;
    }

    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .context("Failed to set foreign_keys pragma")?;

    Ok(Arc::new(Mutex::new(conn)))
}

pub fn create_memory_pool() -> Result<DbPool> {
    // Each call creates a fresh private in-memory database.
    // Tests must NOT share pools — each test gets its own via create_memory_pool().
    let conn = Connection::open_in_memory()
        .context("Failed to open in-memory database")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .context("Failed to set PRAGMAs")?;
    Ok(Arc::new(Mutex::new(conn)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_memory_pool() {
        let pool = create_memory_pool();
        assert!(pool.is_ok());
    }

    #[test]
    fn test_create_pool_with_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        
        let pool = create_pool_with_key(&db_path, key);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_create_pool_uses_keyring() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_keyring.db");
        
        let pool = create_pool(&db_path, dir.path());
        assert!(pool.is_ok());
    }
}
