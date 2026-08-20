use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tauri::command;

/// Tenant configuration loaded at compile time from tenant-configs/*.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub tenant: TenantInfo,
    pub brand: BrandTokens,
    pub brandDark: BrandTokens,
    pub typography: TypographyConfig,
    pub crypto: CryptoConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    pub id: String,
    pub commercialName: String,
    pub clinicalRole: String,
    pub ownerName: String,
    pub ownerTitle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandTokens {
    pub primary: String,
    pub primaryForeground: String,
    pub secondary: String,
    pub secondaryForeground: String,
    pub accent: String,
    pub accentForeground: String,
    pub background: String,
    pub foreground: String,
    pub muted: String,
    pub mutedForeground: String,
    pub card: String,
    pub cardForeground: String,
    pub border: String,
    pub input: String,
    pub ring: String,
    pub destructive: String,
    pub destructiveForeground: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyConfig {
    pub fontFamily: String,
    pub headingWeight: String,
    pub bodyWeight: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub keyringService: String,
    pub keyringAccount: String,
    pub dbFileName: String,
}

impl CryptoConfig {
    /// Validate crypto configuration fields at runtime.
    /// Returns Err with descriptive message if any field is empty or whitespace-only.
    pub fn validate(&self) -> Result<(), String> {
        if self.keyringService.trim().is_empty() {
            return Err("CryptoConfig.keyringService must not be empty".to_string());
        }
        if self.keyringAccount.trim().is_empty() {
            return Err("CryptoConfig.keyringAccount must not be empty".to_string());
        }
        if self.dbFileName.trim().is_empty() {
            return Err("CryptoConfig.dbFileName must not be empty".to_string());
        }
        // dbFileName must end with .db extension
        if !self.dbFileName.ends_with(".db") {
            return Err(format!(
                "CryptoConfig.dbFileName must end with .db extension, got: {}",
                self.dbFileName
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub clinicalNotes: bool,
    pub accounting: bool,
    pub agenda: bool,
    pub diagnostics: bool,
}

/// Global cached tenant config (loaded once at startup)
static TENANT_CONFIG: OnceLock<Result<TenantConfig, String>> = OnceLock::new();

impl TenantConfig {
    /// Validate all configuration sections at runtime.
    /// Crypto config is critical — invalid values cause hard failure.
    pub fn validate(&self) -> Result<(), String> {
        self.crypto.validate()
            .map_err(|e| format!("Tenant config validation failed: {}", e))
    }
}

/// Load tenant config with a 3-tier fallback strategy:
///   1. File at TENANT_CONFIG_PATH (works in dev and when file is shipped with installer)
///   2. Embedded JSON content via TENANT_CONFIG_JSON env var (works in release builds
///      where the path points to a non-existent CI build directory)
///   3. Compile-time default.json (backward compat for tests)
fn load_tenant_config() -> Result<TenantConfig, String> {
    // Tier 1: Try reading from TENANT_CONFIG_PATH file
    // This works when the file actually exists (dev machine, or shipped alongside binary)
    if let Ok(config_path) = std::env::var("TENANT_CONFIG_PATH") {
        if let Ok(config_str) = std::fs::read_to_string(&config_path) {
            let config: TenantConfig = serde_json::from_str(&config_str)
                .map_err(|e| format!("Failed to parse tenant config: {}", e))?;
            config.validate()?;
            return Ok(config);
        }
        // File not found or unreadable — fall through to embedded content
    }

    // Tier 2: Embedded config content (propagated from build.rs via main.rs)
    // This is the ACTIVE tenant config JSON embedded at compile time.
    // Works even when TENANT_CONFIG_PATH points to a non-existent file
    // (e.g. release builds where path references CI build machine filesystem).
    if let Ok(config_str) = std::env::var("TENANT_CONFIG_JSON") {
        let config: TenantConfig = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse embedded tenant config: {}", e))?;
        config.validate()?;
        return Ok(config);
    }

    // Tier 3: Compile-time default config (tests, backward compatibility)
    const DEFAULT_CONFIG: &str = include_str!("../../../tenant-configs/default.json");
    let config: TenantConfig = serde_json::from_str(DEFAULT_CONFIG)
        .map_err(|e| format!("Default config parse error: {}", e))?;
    config.validate()?;
    Ok(config)
}

/// Get or initialize the cached tenant config
pub fn get_tenant_config_cached() -> Result<&'static TenantConfig, String> {
    TENANT_CONFIG.get_or_init(load_tenant_config).as_ref().map_err(|e| e.clone())
}

/// Tauri command: returns the tenant configuration compiled into the binary.
/// The frontend uses this to apply branding, typography, and feature flags.
#[command]
pub async fn get_tenant_config() -> AppResult<TenantConfig> {
    get_tenant_config_cached().cloned().map_err(|e| AppError::Validation(e))
}

/// Helper: get the crypto keyring account name for the current tenant.
/// Used by infrastructure layer to isolate encryption keys per tenant.
pub fn get_tenant_keyring_account() -> String {
    get_tenant_config_cached()
        .map(|c| c.crypto.keyringAccount.clone())
        .unwrap_or_else(|_| "sqlcipher-key".to_string())
}

/// Helper: get the tenant database filename.
pub fn get_tenant_db_filename() -> String {
    get_tenant_config_cached()
        .map(|c| c.crypto.dbFileName.clone())
        .unwrap_or_else(|_| "mind_ledger.db".to_string())
}

/// Helper: get the tenant ID.
pub fn get_tenant_id() -> String {
    get_tenant_config_cached()
        .map(|c| c.tenant.id.clone())
        .unwrap_or_else(|_| "default".to_string())
}
