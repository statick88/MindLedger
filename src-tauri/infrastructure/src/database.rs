use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::keyring::SqlCipherKeyManager;

pub type DbPool = Arc<Mutex<Connection>>;

const DEFAULT_SERVICE_NAME: &str = "soft-gloria";
const DEFAULT_ACCOUNT_NAME: &str = "sqlcipher-key";

pub fn create_pool(database_path: &Path) -> Result<DbPool> {
    let key_manager = SqlCipherKeyManager::new(DEFAULT_SERVICE_NAME, DEFAULT_ACCOUNT_NAME)?;
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
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .context("Failed to set PRAGMAs")?;
    
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn create_memory_pool() -> Result<DbPool> {
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
        
        let pool = create_pool(&db_path);
        assert!(pool.is_ok());
    }
}
