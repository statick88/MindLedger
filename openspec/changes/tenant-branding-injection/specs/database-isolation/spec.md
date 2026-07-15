# Database Isolation — Full Spec (New Capability)

## Purpose

Defines per-tenant database isolation: unique keyring accounts, database filenames, and data directories to ensure zero data crossover between tenant builds.

## Requirements

### REQ-DI-001: Per-Tenant Keyring Account
The `SqlCipherKeyManager` SHALL use a tenant-specific keyring account name derived from `tenant.config.crypto.keyringAccount`.

**Keyring Configuration:**
- **Service:** `tenant.config.crypto.keyringService` (default: "mind-ledger")
- **Account:** `tenant.config.crypto.keyringAccount` (format: `sqlcipher-key-{tenant.id}`)

**Function Signature (existing):**
```rust
pub fn new_with_fallback(
    service_name: &str,
    account_name: &str,
    data_dir: &Path,
) -> Self
```

**Given** tenant config: `crypto.keyringAccount = "sqlcipher-key-gloria-once"`  
**When** `SqlCipherKeyManager::new_with_fallback("mind-ledger", "sqlcipher-key-gloria-once", data_dir)` called  
**Then** keyring operations target `service="mind-ledger", account="sqlcipher-key-gloria-once"`

**Isolation Guarantee:** Two tenant builds on same machine use different keyring accounts → different encryption keys → cannot decrypt each other's databases.

---

### REQ-DI-002: Per-Tenant Database Filename
The database filename SHALL be derived from `tenant.config.crypto.dbFileName`.

**Format:** `mind_ledger_{tenant.id}.db` (e.g., `mind_ledger_gloria_once.db`)

**Given** tenant config: `crypto.dbFileName = "mind_ledger_gloria_once.db"`  
**When** database pool created  
**Then** SQLite opens `$DATA_DIR/mind_ledger_gloria_once.db`

**Default Build:** `mind_ledger.db` (backward compatible)

---

### REQ-DI-003: Per-Tenant Data Directory
The application data directory SHALL be tenant-specific:

**Directory Pattern:** `$APPDATA/mind-ledger-{tenant.id}/`

**Windows:** `%APPDATA%\mind-ledger-{tenant.id}\`
**macOS:** `~/Library/Application Support/mind-ledger-{tenant.id}/`
**Linux:** `~/.local/share/mind-ledger-{tenant.id}/`

**Derivation in main.rs:**
```rust
let tenant_id = get_tenant_id(); // from embedded config
let data_dir = app_data_dir.join(format!("mind-ledger-{}", tenant_id));
```

**Given** `tenant_id = "gloria-once"` on Windows  
**When** app initializes  
**Then** data dir = `%APPDATA%\mind-ledger-gloria-once\`

**Given** `tenant_id = "default"` (no config)  
**When** app initializes  
**Then** data dir = `%APPDATA%\mind-ledger\` (backward compatible)

---

### REQ-DI-004: Main.rs Initializes with Tenant Paths
The `main.rs` setup hook SHALL derive all paths from embedded tenant config.

**Implementation:**
```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let data_dir = app.path().app_data_dir()?;
            
            // Derive tenant-specific paths
            let tenant_id = soft_gloria_commands::tenant::get_tenant_id();
            let tenant_data_dir = data_dir.join(format!("mind-ledger-{}", tenant_id));
            
            let db_filename = soft_gloria_commands::tenant::get_tenant_db_filename();
            let db_path = tenant_data_dir.join(db_filename);
            
            // Create tenant data directory
            std::fs::create_dir_all(&tenant_data_dir)?;
            
            // Initialize DB with tenant-specific keyring account
            let keyring_account = soft_gloria_commands::tenant::get_tenant_keyring_account();
            let keyring_service = "mind-ledger"; // from config or default
            
            let key_manager = SqlCipherKeyManager::new_with_fallback(
                keyring_service,
                &keyring_account,
                &tenant_data_dir,
            );
            let key = key_manager.get_or_create_key()?;
            let db = create_pool_with_key(&db_path, &key)?;
            
            // Run migrations...
            app_handle.manage(Arc::new(db));
            Ok(())
        })
        // ... invoke_handler
}
```

**Given** Gloria Once build  
**When** app starts  
**Then** uses `mind-ledger-gloria-once/` dir, `mind_ledger_gloria_once.db`, `sqlcipher-key-gloria-once` keyring

**Given** Default build  
**When** app starts  
**Then** uses `mind-ledger/` dir, `mind_ledger.db`, `sqlcipher-key` keyring

---

### REQ-DI-005: create_pool_with_key Used for Tenant DB
The infrastructure layer SHALL use `create_pool_with_key` (which accepts explicit key) instead of `create_pool` (which derives key internally) to enable tenant-specific key derivation.

**Given** tenant-specific key obtained from keyring  
**When** `create_pool_with_key(&db_path, &key)` called  
**Then** database encrypted with that specific key

---

### REQ-DI-006: Zero Data Crossover Guarantee
Two different tenant builds running on the same machine SHALL have zero data visibility into each other.

**Verification Checklist:**
- [ ] Different keyring accounts → different encryption keys
- [ ] Different data directories → different filesystem locations
- [ ] Different database filenames → different files
- [ ] Running both apps simultaneously → each sees only its own data

**Given** Gloria Once app and Default app both installed  
**When** user creates patient in Gloria Once app  
**Then** patient NOT visible in Default app  
**And** database files are separate on disk  
**And** keyring entries are separate

---

### REQ-DI-007: Backward Compatibility — Default Build Works
A build without `TENANT_CONFIG` MUST continue to work with hardcoded defaults.

**Defaults:**
- Service: "mind-ledger"
- Account: "sqlcipher-key"
- DB filename: "mind_ledger.db"
- Data dir: `$APPDATA/mind-ledger/`

**Implementation:** Helper functions in `tenant.rs` return defaults when config not embedded.

```rust
pub fn get_tenant_keyring_account() -> String {
    let config_str = include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"));
    serde_json::from_str::<TenantConfig>(config_str)
        .map(|c| c.crypto.keyringAccount)
        .unwrap_or_else(|_| "sqlcipher-key".to_string())
}
```

**Given** `cargo tauri build` (no TENANT_CONFIG)  
**When** app runs  
**Then** uses all hardcoded defaults  
**And** existing user data in `mind-ledger/` accessible

---

## Scenarios

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Gloria Once paths | Config has gloria-once IDs | App starts | Uses `mind-ledger-gloria-once/`, `mind_ledger_gloria_once.db`, `sqlcipher-key-gloria-once` |
| Default build | No TENANT_CONFIG | App starts | Uses `mind-ledger/`, `mind_ledger.db`, `sqlcipher-key` |
| Keyring isolation | Both apps installed | Both run | Different keyring entries, different keys |
| FS isolation | Both apps installed | Both run | Separate data directories, no crossover |
| Migration path | Existing default user | Installs Gloria Once | Default data unchanged, Gloria Once starts fresh |