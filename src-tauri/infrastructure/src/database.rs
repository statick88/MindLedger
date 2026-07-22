use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::warn;

use crate::keyring::SqlCipherKeyManager;

pub type DbPool = Arc<Mutex<Connection>>;

const DEFAULT_SERVICE_NAME: &str = "mind-ledger";
const DEFAULT_ACCOUNT_NAME: &str = "sqlcipher-key";
const DEFAULT_DB_FILENAME: &str = "mind_ledger.db";

/// Key entropy validation error
#[derive(Debug, thiserror::Error)]
pub enum KeyEntropyError {
    #[error("Key entropy too low: {0:.2} bits/char (minimum 3.5)")]
    LowEntropy(f64),
    #[error("Key contains repeating pattern: {0}")]
    RepeatingPattern(String),
    #[error("Key is all zeros or all same character")]
    AllSameChar,
}

/// Validate SQLCipher key entropy to reject weak keys.
/// Returns Ok(()) if key passes entropy checks, Err(KeyEntropyError) otherwise.
pub fn validate_key_entropy(key: &str) -> Result<(), KeyEntropyError> {
    // Check for all same character (e.g., all zeros, all 'a's)
    if key.chars().all(|c| c == key.chars().next().unwrap()) {
        return Err(KeyEntropyError::AllSameChar);
    }

    // Check for repeating pattern (e.g., "0123456789abcdef" repeated)
    // Find smallest repeating unit
    for pattern_len in 1..=key.len() / 2 {
        if key.len() % pattern_len == 0 {
            let pattern = &key[..pattern_len];
            let mut is_repeating = true;
            for chunk in key.as_bytes().chunks(pattern_len) {
                if chunk != pattern.as_bytes() {
                    is_repeating = false;
                    break;
                }
            }
            if is_repeating {
                return Err(KeyEntropyError::RepeatingPattern(pattern.to_string()));
            }
        }
    }

    // Calculate Shannon entropy
    let mut freq = [0u32; 16]; // hex digits 0-9, a-f
    for c in key.chars() {
        if let Some(digit) = c.to_digit(16) {
            freq[digit as usize] += 1;
        }
    }
    let len = key.len() as f64;
    let mut entropy = 0.0;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    // Minimum 3.5 bits/char for 64-char hex key (max is 4.0 for uniform)
    if entropy < 3.5 {
        return Err(KeyEntropyError::LowEntropy(entropy));
    }

    Ok(())
}

/// Create a connection pool with tenant-specific configuration.
/// Uses tenant-specific keyring account and database filename for isolation.
pub fn create_pool_for_tenant(
    data_dir: &Path,
    keyring_account: &str,
    db_filename: &str,
) -> Result<DbPool> {
    let db_path = data_dir.join(db_filename);
    let service_name = DEFAULT_SERVICE_NAME; // Shared service, unique account per tenant
    
    let key_manager = SqlCipherKeyManager::new_with_fallback(
        service_name,
        keyring_account,
        data_dir,
    );
    
    let key = key_manager.get_or_create_key()?;
    create_pool_with_key(&db_path, &key)
}

/// Backward-compatible create_pool (for tests, default tenant).
/// Delegates to create_pool_for_tenant with default values.
pub fn create_pool(_database_path: &Path, data_dir: &Path) -> Result<DbPool> {
    create_pool_for_tenant(
        data_dir,
        DEFAULT_ACCOUNT_NAME,
        DEFAULT_DB_FILENAME,
    )
}

/// Open (or create) an encrypted SQLCipher database with the given hex key.
/// Validates key format (64 hex chars) and entropy before executing PRAGMA to prevent injection and weak keys.
pub fn create_pool_with_key(database_path: &Path, key: &str) -> Result<DbPool> {
    // Security: validate key format before passing to SQLCipher.
    // Keys must be exactly KEY_LENGTH*2 hex characters (64 chars for 32-byte key).
    // This prevents SQL injection via malformed keys and ensures proper key derivation.
    const EXPECTED_KEY_LEN: usize = 64; // 32 bytes * 2 hex chars
    if key.len() != EXPECTED_KEY_LEN {
        anyhow::bail!(
            "Invalid encryption key length: expected {} hex chars, got {}",
            EXPECTED_KEY_LEN,
            key.len()
        );
    }
    if !key.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("Invalid encryption key: must contain only hex characters (0-9, a-f)");
    }

    // Security: validate key entropy (reject weak keys)
    validate_key_entropy(key)?;

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

    // Use hex-literal form: PRAGMA key = "x'HEX_KEY'"
    // This avoids SQL injection via single-quote escaping and is the
    // recommended SQLCipher format for programmatic key injection.
    let pragma_key = format!("PRAGMA key = \"x'{}'\";", key);
    conn.execute_batch(&pragma_key)
        .context("Failed to set encryption key")?;

    // SQLCipher hardening: strengthen encryption parameters
    conn.execute_batch(
        "PRAGMA cipher_page_size = 4096;
         PRAGMA kdf_iter = 256000;
         PRAGMA cipher_hmac_algorithm = HMAC_SHA512;
         PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA512;"
    ).context("Failed to set SQLCipher hardening parameters")?;

    // Set journal mode — try WAL first, fall back to DELETE (SQLCipher may not support WAL).
    if let Err(e) = conn.execute_batch("PRAGMA journal_mode=WAL;") {
        warn!("[MindLedger] WAL mode unavailable ({}), using DELETE", e);
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

    #[test]
    fn test_create_pool_for_tenant_isolation() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("mind-ledger-default");
        
        let pool = create_pool_for_tenant(
            &data_dir,
            "sqlcipher-key-default",
            "mind_ledger_default.db"
        );
        
        assert!(pool.is_ok());
        
        // Verify DB file created in tenant-specific directory
        let db_path = data_dir.join("mind_ledger_default.db");
        assert!(db_path.exists());
    }
}
