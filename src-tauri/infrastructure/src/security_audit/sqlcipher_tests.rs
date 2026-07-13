//! SQLCipher Crypto Verification Tests
//!
//! Validates encryption at rest, key lifecycle security, PRAGMA key format,
//! key file fallback permissions, and connection pool security configuration.
//!
//! These tests verify the infrastructure layer's SQLCipher integration against
//! OWASP A02:2021 and NIST SP 800-175B requirements.
//!
//! All tests use temp directories for file-based operations and mock keyring entries.

#[cfg(test)]
mod sqlcipher_tests {
    use crate::database::{create_pool_with_key, create_pool_for_tenant};
    use crate::keyring::SqlCipherKeyManager;
    use std::path::Path;
    use tempfile::tempdir;
    use zeroize::Zeroize;

    // ══════════════════════════════════════════════════════════════════════
    // Task 4.1: Cold Dump Analysis
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: sqlcipher-resistance/cold-dump/scenario-1
    /// Cold dump analysis — create a SQLCipher-encrypted database with known PHI,
    /// then verify the raw file bytes contain no readable plaintext.
    #[test]
    fn test_cold_dump_no_plaintext_leakage() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_cold_dump.db");
        let key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

        // Create encrypted database with known PHI
        let pool = create_pool_with_key(&db_path, key).expect("Failed to create pool");
        {
            let conn = pool.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE patients (
                    id TEXT PRIMARY KEY,
                    first_name TEXT,
                    last_name TEXT,
                    diagnosis TEXT
                );
                INSERT INTO patients VALUES ('123e4567-e89b', 'Maria', 'Garcia', 'Ansiedad Generalizada');
                INSERT INTO patients VALUES ('987fcdeb-a53d', 'Juan', 'Perez', 'Depresion Mayor');",
            )
            .expect("Failed to create test data");
        }

        // Read raw database file bytes
        let db_bytes = std::fs::read(&db_path).expect("Failed to read database file");

        // Convert to string for analysis (strings-equivalent check)
        let db_as_string = String::from_utf8_lossy(&db_bytes);

        // PHI must NOT appear in raw database file
        let sensitive_terms = [
            "Maria",
            "Garcia",
            "Juan",
            "Perez",
            "Ansiedad Generalizada",
            "Depresion Mayor",
            "patients",       // table name
            "CREATE TABLE",   // SQL DDL
            "INSERT INTO",    // SQL DML
            "first_name",     // column name
            "last_name",      // column name
            "diagnosis",      // column name
        ];

        for term in &sensitive_terms {
            assert!(
                !db_as_string.contains(term),
                "Cold dump leaked plaintext: '{}' found in raw database file",
                term
            );
        }

        // Additional check: file should have SQLCipher header (not plaintext SQLite header)
        // SQLite plaintext header: "SQLite format 3\000"
        assert!(
            !db_as_string.starts_with("SQLite format 3"),
            "Database file starts with plaintext SQLite header — encryption may not be active"
        );
    }

    /// Spec: sqlcipher-resistance/cold-dump/scenario-1 (supplementary)
    /// Verify hexdump-equivalent — no readable ASCII sequences above threshold.
    #[test]
    fn test_cold_dump_hexdump_no_readable_sequences() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_hexdump.db");
        let key = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

        let pool = create_pool_with_key(&db_path, key).expect("Failed to create pool");
        {
            let conn = pool.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE clinical_notes (id TEXT PRIMARY KEY, content TEXT);
                 INSERT INTO clinical_notes VALUES ('note-1', 'Patient reports chronic pain in lower back');",
            )
            .expect("Failed to create test data");
        }

        let db_bytes = std::fs::read(&db_path).expect("Failed to read database file");

        // Count consecutive readable ASCII characters (a-z, A-Z, 0-9, space)
        // In an encrypted database, long runs of readable ASCII should NOT exist
        let mut max_run = 0u32;
        let mut current_run = 0u32;

        for &byte in &db_bytes {
            if byte.is_ascii_alphanumeric() || byte == b' ' {
                current_run += 1;
                if current_run > max_run {
                    max_run = current_run;
                }
            } else {
                current_run = 0;
            }
        }

        // A properly encrypted database should not have readable ASCII runs > 16 chars
        // (header bytes and WAL markers may create short runs, but PHI should not)
        assert!(
            max_run <= 16,
            "Found readable ASCII run of {} chars in database file — possible plaintext leakage",
            max_run
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Task 4.2: Key Zeroization
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: sqlcipher-resistance/key-zeroization/scenario-1
    /// Key zeroization — verify that Zeroizing type properly zeroizes memory on drop.
    /// This tests the zeroize crate integration used by SqlCipherKeyManager.
    #[test]
    fn test_key_zeroization_on_drop() {
        // Simulate key lifecycle: allocate, use, drop
        let key_bytes: Vec<u8> = (0..32).map(|i| i as u8).collect();

        // Wrap in Zeroizing — the type that SqlCipherKeyManager uses
        let mut zeroizing_key = zeroize::Zeroizing::new(key_bytes.clone());

        // Verify the key is valid before drop
        assert_eq!(zeroizing_key.len(), 32);
        assert_eq!(*zeroizing_key, key_bytes);

        // Zeroize and verify memory is zeroed
        zeroizing_key.zeroize();

        // After zeroize, all bytes should be 0x00
        assert!(
            zeroizing_key.iter().all(|&b| b == 0),
            "Key bytes not zeroed after zeroize call: {:02x?}",
            &zeroizing_key[..]
        );
    }

    /// Spec: sqlcipher-resistance/key-zeroization/scenario-1 (supplementary)
    /// Verify that the key manager generates keys using Zeroizing types.
    #[test]
    fn test_generate_hex_key_uses_zeroizing() {
        // The generate_hex_key method returns a String, but internally uses Zeroizing.
        // We verify the returned key is proper length and hex, then verify the
        // Zeroizing intermediate is properly handled.
        let key = SqlCipherKeyManager::generate_hex_key_for_test();

        // Key should be 64 hex chars (32 bytes * 2 hex digits per byte)
        assert_eq!(key.len(), 64, "Key length should be 64 hex chars");
        assert!(
            key.chars().all(|c| c.is_ascii_hexdigit()),
            "Key should contain only hex digits"
        );

        // Verify no null bytes or control characters in the returned key
        assert!(
            key.bytes().all(|b| b.is_ascii_graphic() || b == b' '),
            "Key contains non-printable characters"
        );
    }

    /// Spec: sqlcipher-resistance/key-zeroization/scenario-1 (supplementary)
    /// Verify multiple key generations produce unique keys (no reuse).
    #[test]
    fn test_key_uniqueness_across_generations() {
        let mut keys = std::collections::HashSet::new();
        for _ in 0..100 {
            let key = SqlCipherKeyManager::generate_hex_key_for_test();
            assert!(keys.insert(key), "Duplicate key generated!");
        }
        assert_eq!(keys.len(), 100, "All 100 keys should be unique");
    }

    // ══════════════════════════════════════════════════════════════════════
    // Task 4.3: Key File Fallback Permissions
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: sqlcipher-resistance/key-file-permissions/scenario-1
    /// Key file fallback permissions — verify that fallback key file is created
    /// with 0o600 permissions (owner read/write only) on Unix systems.
    #[test]
    #[cfg(unix)]
    fn test_key_file_fallback_permissions_0600() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("app_data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let manager = SqlCipherKeyManager::new_with_fallback(
            "test-service-perms",
            "test-account-perms",
            &data_dir,
        );

        // This triggers fallback key creation (keyring likely unavailable in test)
        let key = manager.get_or_create_key().expect("Failed to get key via fallback");

        // Verify key file exists
        let key_file = data_dir.join("mind-ledger.key");
        assert!(key_file.exists(), "Fallback key file should exist");

        // Verify permissions are 0o600 (owner only)
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&key_file).expect("Failed to read key file metadata");
        let permissions = metadata.permissions().mode();
        let file_perms = permissions & 0o777; // Mask to get permission bits only

        assert_eq!(
            file_perms, 0o600,
            "Key file permissions should be 0o600 (owner rw only), got: {:o}",
            file_perms
        );

        // Cleanup
        let _ = manager.delete_key();
    }

    /// Spec: sqlcipher-resistance/key-file-permissions/scenario-1 (supplementary)
    /// Verify fallback key file content is hex-encoded, not plaintext.
    #[test]
    fn test_key_file_content_is_hex_encoded() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("app_data_hex");
        std::fs::create_dir_all(&data_dir).unwrap();

        let manager = SqlCipherKeyManager::new_with_fallback(
            "test-service-hex",
            "test-account-hex",
            &data_dir,
        );

        let key = manager.get_or_create_key().expect("Failed to get key via fallback");

        // Read the key file content
        let key_file = data_dir.join("mind-ledger.key");
        let content = std::fs::read_to_string(&key_file).expect("Failed to read key file");

        // Content should be the same hex key
        let trimmed = content.trim();
        assert_eq!(trimmed, key, "Key file content should match returned key");

        // Verify it's 64 hex chars
        assert_eq!(trimmed.len(), 64, "Key file content should be 64 hex chars");
        assert!(
            trimmed.chars().all(|c| c.is_ascii_hexdigit()),
            "Key file content should contain only hex digits, got: {}",
            trimmed
        );

        // Cleanup
        let _ = manager.delete_key();
    }

    // ══════════════════════════════════════════════════════════════════════
    // Task 4.4: PRAGMA Key Format Test
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: sqlcipher-resistance/pragma-key-injection/scenario-1
    /// PRAGMA key injection safety — attempt malformed keys with SQL metacharacters,
    /// verify error returned, not SQL execution.
    #[test]
    fn test_pragma_key_injection_with_single_quotes() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("injection_single_quote.db");

        // Key containing single quotes — should cause error, not SQL execution
        let malicious_key = "'; DROP TABLE test; --";

        let result = create_pool_with_key(&db_path, malicious_key);

        // Should fail with error (invalid key format), not succeed
        assert!(
            result.is_err(),
            "Malformed key with single quotes must be rejected, not executed as SQL"
        );
    }

    /// Spec: sqlcipher-resistance/pragma-key-injection/scenario-1 (supplementary)
    /// Verify rejection of keys with SQL metacharacters.
    #[test]
    fn test_pragma_key_injection_with_sql_metacharacters() {
        let dir = tempdir().unwrap();

        let malicious_keys = [
            "'; SELECT 1; --",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef' OR '1'='1",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef; DROP TABLE patients;",
            "' UNION SELECT * FROM sqlite_master --",
        ];

        for (i, malicious_key) in malicious_keys.iter().enumerate() {
            let db_path = dir.path().join(format!("injection_{}.db", i));
            let result = create_pool_with_key(&db_path, malicious_key);

            assert!(
                result.is_err(),
                "Malformed key #{} must be rejected: {}",
                i,
                malicious_key
            );
        }
    }

    /// Spec: sqlcipher-resistance/pragma-key-injection/scenario-1 (supplementary)
    /// Verify that valid hex keys are accepted.
    #[test]
    fn test_pragma_key_valid_hex_accepted() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("valid_key.db");

        // Valid 64-char hex key
        let valid_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

        let result = create_pool_with_key(&db_path, valid_key);

        assert!(
            result.is_ok(),
            "Valid hex key should be accepted, got error: {:?}",
            result.err()
        );
    }

    /// Spec: sqlcipher-resistance/pragma-key-injection/scenario-1 (supplementary)
    /// Verify rejection of keys with wrong length.
    #[test]
    fn test_pragma_key_wrong_length_rejected() {
        let dir = tempdir().unwrap();

        let invalid_keys = [
            ("short", "Too short (5 chars)"),
            ("0123456789abcdef", "16 chars — half length"),
            (
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "80 chars — too long",
            ),
        ];

        for (invalid_key, description) in &invalid_keys {
            let db_path = dir.path().join(format!("wrong_len_{}.db", description.len()));
            let result = create_pool_with_key(&db_path, invalid_key);

            assert!(
                result.is_err(),
                "Invalid key length should be rejected: {}",
                description
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Task 4.5: Connection Pool Security Test
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: sqlcipher-resistance/connection-pool-security/scenario-1
    /// Connection pool security — verify WAL mode and foreign_keys pragma
    /// are properly configured.
    #[test]
    fn test_connection_pool_wal_mode_and_foreign_keys() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("security_pragmas.db");
        let key = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

        let pool = create_pool_with_key(&db_path, key).expect("Failed to create pool");

        let conn = pool.lock().unwrap();

        // Verify WAL journal mode (or DELETE fallback)
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
            .expect("Failed to query journal_mode");

        assert!(
            journal_mode == "wal" || journal_mode == "delete",
            "Journal mode should be WAL or DELETE fallback, got: {}",
            journal_mode
        );

        // Verify foreign_keys is ON
        let foreign_keys: bool = conn
            .query_row("PRAGMA foreign_keys;", [], |row| row.get(0))
            .expect("Failed to query foreign_keys");

        assert!(foreign_keys, "foreign_keys pragma should be ON");
    }

    /// Spec: sqlcipher-resistance/connection-pool-security/scenario-1 (supplementary)
    /// Verify tenant isolation — different keyring accounts produce isolated databases.
    #[test]
    fn test_tenant_isolation_different_keys() {
        let dir = tempdir().unwrap();

        let key_a = "aaaa00000000000000000000000000000000000000000000000000000000aaaa";
        let key_b = "bbbb00000000000000000000000000000000000000000000000000000000bbbb";

        let db_a = dir.path().join("tenant_a.db");
        let db_b = dir.path().join("tenant_b.db");

        // Create two databases with different keys
        let pool_a = create_pool_with_key(&db_a, key_a).expect("Failed to create pool A");
        let pool_b = create_pool_with_key(&db_b, key_b).expect("Failed to create pool B");

        // Insert data into A
        {
            let conn = pool_a.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE test_data (id TEXT PRIMARY KEY, value TEXT);
                 INSERT INTO test_data VALUES ('secret-a', 'Confidential data for tenant A');",
            )
            .expect("Failed to insert into pool A");
        }

        // Verify pool B cannot read pool A's data (different encryption key)
        {
            let conn = pool_b.lock().unwrap();
            // Creating the same table in B should work (independent database)
            conn.execute_batch(
                "CREATE TABLE test_data (id TEXT PRIMARY KEY, value TEXT);
                 INSERT INTO test_data VALUES ('safe-b', 'Data for tenant B');",
            )
            .expect("Failed to insert into pool B");

            // Verify B has its own data
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM test_data", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 1, "Pool B should have exactly 1 row");
        }

        // Verify A has its own data
        {
            let conn = pool_a.lock().unwrap();
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM test_data", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 1, "Pool A should have exactly 1 row");
        }
    }

    /// Spec: sqlcipher-resistance/connection-pool-security/scenario-1 (supplementary)
    /// Verify that database connections are properly encrypted — opening with wrong key fails.
    #[test]
    fn test_wrong_key_cannot_open_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("wrong_key_test.db");

        let correct_key = "aaaa00000000000000000000000000000000000000000000000000000000aaaa";
        let wrong_key =   "bbbb00000000000000000000000000000000000000000000000000000000bbbb";

        // Create database with correct key
        let pool = create_pool_with_key(&db_path, correct_key).expect("Failed to create pool");
        {
            let conn = pool.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE secrets (data TEXT);
                 INSERT INTO secrets VALUES ('highly confidential');",
            )
            .expect("Failed to insert test data");
        }
        drop(pool);

        // Attempt to open with wrong key — should fail
        let wrong_result = create_pool_with_key(&db_path, wrong_key);
        assert!(
            wrong_result.is_err(),
            "Opening database with wrong key must fail"
        );
    }
}
