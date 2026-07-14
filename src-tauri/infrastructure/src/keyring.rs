use anyhow::{Context, Result};
use keyring::Entry;
use rand::Rng;
use std::path::Path;
use zeroize::Zeroizing;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const KEY_LENGTH: usize = 32;
const FALLBACK_KEY_FILE: &str = "mind-ledger.key";

pub struct SqlCipherKeyManager {
    entry: Option<Entry>,
    fallback_path: Option<std::path::PathBuf>,
}

impl SqlCipherKeyManager {
    pub fn new(service_name: &str, account_name: &str) -> Result<Self> {
        let entry = Entry::new(service_name, account_name)
            .context("Failed to create keyring entry")?;
        Ok(Self { entry: Some(entry), fallback_path: None })
    }

    /// Create a key manager with file-based fallback if keyring is unavailable.
    /// `data_dir` is the app data directory where the fallback key file is stored.
    pub fn new_with_fallback(
        service_name: &str,
        account_name: &str,
        data_dir: &Path,
    ) -> Self {
        let entry = Entry::new(service_name, account_name).ok();
        let fallback_path = Some(data_dir.join(FALLBACK_KEY_FILE));
        if entry.is_none() {
            eprintln!(
                "[MindLedger] WARNING: keyring unavailable (service={}), using file-based key fallback",
                service_name
            );
        }
        Self { entry, fallback_path }
    }

    pub fn get_or_create_key(&self) -> Result<String> {
        // 1. Try keyring first
        if let Some(ref entry) = self.entry {
            if let Ok(key) = entry.get_password() {
                return Ok(key);
            }
            // Keyring has no key yet — try to create one
            let key = Self::generate_hex_key();
            match entry.set_password(&key) {
                Ok(()) => return Ok(key),
                Err(e) => {
                    eprintln!(
                        "[MindLedger] WARNING: keyring set_password failed: {}. Falling back to file key.",
                        e
                    );
                }
            }
        }

        // 2. Fallback: read or create key from local file
        if let Some(ref fallback_path) = self.fallback_path {
            if let Ok(content) = std::fs::read_to_string(fallback_path) {
                let key = content.trim().to_string();
                if key.len() == KEY_LENGTH * 2 && key.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Ok(key);
                }
                eprintln!(
                    "[MindLedger] WARNING: fallback key file is invalid, regenerating"
                );
            }
            let key = Self::generate_hex_key();
            if let Some(parent) = fallback_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::write(fallback_path, &key)
                .context("Failed to write fallback key file")?;
            // Set file permissions to owner-only read/write (0o600) on Unix
            #[cfg(unix)]
            {
                std::fs::set_permissions(
                    fallback_path,
                    std::fs::Permissions::from_mode(0o600),
                ).context("Failed to set key file permissions")?;
            }
            eprintln!(
                "[MindLedger] INFO: wrote fallback key to {}",
                fallback_path.display()
            );
            return Ok(key);
        }

        anyhow::bail!("No keyring and no fallback path — cannot obtain encryption key")
    }

    pub fn delete_key(&self) -> Result<()> {
        if let Some(ref entry) = self.entry {
            let _ = entry.delete_credential();
        }
        if let Some(ref fallback_path) = self.fallback_path {
            let _ = std::fs::remove_file(fallback_path);
        }
        Ok(())
    }

    fn generate_hex_key() -> String {
        use rand::Rng;
        let mut rng = rand::rngs::OsRng;
        let key_bytes: Zeroizing<Vec<u8>> = Zeroizing::new((0..KEY_LENGTH).map(|_| rng.gen()).collect());
        let hex: Zeroizing<String> = Zeroizing::new(
            key_bytes.iter().map(|b| format!("{:02x}", b)).collect(),
        );
        // NOTE: The returned String is NOT zeroized on drop. Callers requiring
        // zeroization should wrap the result in Zeroizing<String>. The intermediate
        // key material (key_bytes, hex) IS zeroized via the Zeroizing wrapper.
        hex.to_string()
    }

    /// Test-visible wrapper for `generate_hex_key`.
    /// Allows security audit tests to verify key generation properties
    /// without exposing the private method in production builds.
    #[cfg(test)]
    pub fn generate_hex_key_for_test() -> String {
        Self::generate_hex_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hex_key_length() {
        let key = SqlCipherKeyManager::generate_hex_key();
        assert_eq!(key.len(), KEY_LENGTH * 2);
    }

    #[test]
    fn test_generate_hex_key_is_valid_hex() {
        let key = SqlCipherKeyManager::generate_hex_key();
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_keyring_manager_new() {
        let result = SqlCipherKeyManager::new("test-service", "test-account");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_or_create_key_creates_new() {
        let manager = SqlCipherKeyManager::new("test-soft-gloria", "test-create").unwrap();
        let _ = manager.delete_key();
        
        let key = manager.get_or_create_key().unwrap();
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
        
        let _ = manager.delete_key();
    }

    #[test]
    fn test_get_or_create_key_retrieves_existing() {
        let manager = SqlCipherKeyManager::new("test-soft-gloria", "test-retrieve").unwrap();
        let _ = manager.delete_key();
        
        let key1 = manager.get_or_create_key().unwrap();
        let key2 = manager.get_or_create_key().unwrap();
        assert_eq!(key1, key2);
        
        let _ = manager.delete_key();
    }
}
