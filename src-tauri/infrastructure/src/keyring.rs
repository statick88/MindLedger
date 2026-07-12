use anyhow::{Context, Result};
use keyring::Entry;
use rand::Rng;
use zeroize::Zeroizing;

const KEY_LENGTH: usize = 32;

pub struct SqlCipherKeyManager {
    entry: Entry,
}

impl SqlCipherKeyManager {
    pub fn new(service_name: &str, account_name: &str) -> Result<Self> {
        let entry = Entry::new(service_name, account_name)
            .context("Failed to create keyring entry")?;
        Ok(Self { entry })
    }

    pub fn get_or_create_key(&self) -> Result<String> {
        if let Ok(key) = self.entry.get_password() {
            return Ok(key);
        }

        let key = Self::generate_hex_key();
        self.entry
            .set_password(&key)
            .context("Failed to store key in keyring")?;
        Ok(key)
    }

    pub fn delete_key(&self) -> Result<()> {
        self.entry
            .delete_credential()
            .context("Failed to delete key from keyring")?;
        Ok(())
    }

    fn generate_hex_key() -> String {
        let mut rng = rand::thread_rng();
        let key_bytes: Zeroizing<Vec<u8>> = Zeroizing::new((0..KEY_LENGTH).map(|_| rng.gen()).collect());
        let hex: Zeroizing<String> = Zeroizing::new(
            key_bytes.iter().map(|b| format!("{:02x}", b)).collect(),
        );
        hex.to_string()
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
