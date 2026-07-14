# Database Core — Delta Spec

## MODIFIED Requirements

### REQ-DB-001: create_pool Signature Extended (MODIFIED)
**Previously:**
```rust
pub fn create_pool(database_path: &Path, data_dir: &Path) -> Result<DbPool>
```
Used hardcoded constants internally:
```rust
const DEFAULT_SERVICE_NAME: &str = "mind-ledger";
const DEFAULT_ACCOUNT_NAME: &str = "sqlcipher-key";

let key_manager = SqlCipherKeyManager::new_with_fallback(
    DEFAULT_SERVICE_NAME,
    DEFAULT_ACCOUNT_NAME,
    data_dir,
);
```

**Updated:** `create_pool` accepts optional tenant-specific service/account parameters:
```rust
pub fn create_pool(
    database_path: &Path,
    data_dir: &Path,
    service_name: Option<&str>,
    account_name: Option<&str>,
) -> Result<DbPool> {
    let service = service_name.unwrap_or(DEFAULT_SERVICE_NAME);
    let account = account_name.unwrap_or(DEFAULT_ACCOUNT_NAME);
    
    let key_manager = SqlCipherKeyManager::new_with_fallback(service, account, data_dir);
    let key = key_manager.get_or_create_key()?;
    create_pool_with_key(database_path, &key)
}
```

(Previously: no parameters, hardcoded constants used directly)

#### Scenario: Default build (backward compatible)
- GIVEN `create_pool(&db_path, &data_dir, None, None)` called
- WHEN function executes
- THEN uses "mind-ledger" / "sqlcipher-key" constants

#### Scenario: Tenant build
- GIVEN `create_pool(&db_path, &data_dir, Some("mind-ledger"), Some("sqlcipher-key-gloria-once"))` called
- WHEN function executes
- THEN SqlCipherKeyManager uses tenant-specific account

---

### REQ-DB-002: main.rs Calls create_pool with Tenant Params (MODIFIED)
**Previously:**
```rust
let db_path = data_dir.join("mind_ledger.db");
let db = create_pool(&db_path, &data_dir)?;
```

**Updated:**
```rust
// Get tenant config from embedded build-time config
let tenant_id = get_tenant_id(); // "gloria-once" or "default"
let keyring_service = get_tenant_keyring_service(); // from crypto section
let keyring_account = get_tenant_keyring_account(); // from crypto section
let db_filename = get_tenant_db_filename(); // from crypto section

let data_dir = app_data_dir.join(format!("mind-ledger-{}", tenant_id));
let db_path = data_dir.join(db_filename);

let db = create_pool(
    &db_path,
    &data_dir,
    Some(&keyring_service),
    Some(&keyring_account),
)?;
```

(Previously: hardcoded db path, hardcoded keyring params)

#### Scenario: Gloria Once tenant
- GIVEN embedded config has gloria-once values
- WHEN app initializes
- THEN data_dir = `$APPDATA/mind-ledger-gloria-once/`
- AND db_path = `$APPDATA/mind-ledger-gloria-once/mind_ledger_gloria_once.db`
- AND keyring account = `sqlcipher-key-gloria-once`

#### Scenario: Default build
- GIVEN no embedded config (fallback helpers return defaults)
- WHEN app initializes
- THEN data_dir = `$APPDATA/mind-ledger/`
- AND db_path = `$APPDATA/mind-ledger/mind_ledger.db`
- AND keyring account = `sqlcipher-key`

---

## ADDED Requirements

### REQ-DB-003: Helper Functions for Tenant Params (ADDED)
The `commands/src/tenant.rs` module SHALL provide helper functions to extract tenant-specific values for use in `main.rs`:

```rust
pub fn get_tenant_keyring_service() -> String {
    let config_str = include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"));
    serde_json::from_str::<TenantConfig>(config_str)
        .map(|c| c.crypto.keyringService)
        .unwrap_or_else(|_| "mind-ledger".to_string())
}

pub fn get_tenant_keyring_account() -> String {
    let config_str = include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"));
    serde_json::from_str::<TenantConfig>(config_str)
        .map(|c| c.crypto.keyringAccount)
        .unwrap_or_else(|_| "sqlcipher-key".to_string())
}

pub fn get_tenant_db_filename() -> String {
    let config_str = include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"));
    serde_json::from_str::<TenantConfig>(config_str)
        .map(|c| c.crypto.dbFileName)
        .unwrap_or_else(|_| "mind_ledger.db".to_string())
}

pub fn get_tenant_id() -> String {
    let config_str = include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"));
    serde_json::from_str::<TenantConfig>(config_str)
        .map(|c| c.tenant.id)
        .unwrap_or_else(|_| "default".to_string())
}
```

---

## REMOVED Requirements

*(None)*

---

## Scenarios

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Default backward compat | No tenant config | create_pool(path, dir, None, None) | Uses "mind-ledger"/"sqlcipher-key" |
| Tenant-specific | Tenant config embedded | create_pool(path, dir, Some(s), Some(a)) | Uses tenant service/account |
| Gloria Once paths | gloria-once config | main.rs init | Creates mind-ledger-gloria-once/ dir |
| Keyring isolation | Two tenant builds | Both run | Different keyring entries, no collision |