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

/// Load tenant config from embedded file (set by build.rs via TENANT_CONFIG_PATH)
/// Falls back to default.json for tests and backward compatibility.
fn load_tenant_config() -> Result<TenantConfig, String> {
    // At compile time, build.rs (mind-ledger crate) copies the active tenant config
    // to OUT_DIR/tenant-config.json and sets TENANT_CONFIG_PATH via cargo:rustc-env.
    // But cargo:rustc-env is crate-scoped — commands crate can't see it.
    // So we also set the process env var from main.rs before this is called.
    let config = if let Ok(config_path) = std::env::var("TENANT_CONFIG_PATH") {
        if let Ok(config_str) = std::fs::read_to_string(&config_path) {
            serde_json::from_str(&config_str)
                .map_err(|e| format!("Failed to parse tenant config: {}", e))?
        } else {
            return Err(format!("Failed to read tenant config at: {}", config_path));
        }
    } else {
        // Fallback: compile-time default config (for tests and backward compatibility)
        const DEFAULT_CONFIG: &str = include_str!("../../../tenant-configs/default.json");
        serde_json::from_str(DEFAULT_CONFIG)
            .map_err(|e| format!("Default config parse error: {}", e))?
    };

    // Runtime validation — fail fast on invalid crypto config
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
