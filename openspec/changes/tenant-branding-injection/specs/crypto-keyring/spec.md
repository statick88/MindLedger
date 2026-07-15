# Crypto Keyring — Delta Spec

## MODIFIED Requirements

### REQ-CRYPTO-001: Keyring Service/Account from Tenant Config (MODIFIED)
**Previously:** `main.rs` used hardcoded constants when creating the key manager:
```rust
let key_manager = SqlCipherKeyManager::new_with_fallback(
    "mind-ledger",      // hardcoded
    "sqlcipher-key",    // hardcoded
    &data_dir,
);
```

**Updated:** `main.rs` derives service/account from embedded tenant config:
```rust
let tenant_id = soft_gloria_commands::tenant::get_tenant_id();
let keyring_service = soft_gloria_commands::tenant::get_tenant_keyring_service();
let keyring_account = soft_gloria_commands::tenant::get_tenant_keyring_account();

let key_manager = SqlCipherKeyManager::new_with_fallback(
    &keyring_service,
    &keyring_account,
    &tenant_data_dir,
);
```

(Previously: hardcoded "mind-ledger" / "sqlcipher-key")

#### Scenario: Gloria Once build
- GIVEN tenant config embedded with `crypto.keyringService = "mind-ledger"`, `crypto.keyringAccount = "sqlcipher-key-gloria-once"`
- WHEN app initializes
- THEN keyring operations use service="mind-ledger", account="sqlcipher-key-gloria-once"

#### Scenario: Default build
- GIVEN no tenant config (helpers return defaults)
- WHEN app initializes
- THEN keyring uses service="mind-ledger", account="sqlcipher-key"

---

### REQ-CRYPTO-002: Database Filename from Tenant Config (MODIFIED)
**Previously:** `main.rs` hardcoded database filename:
```rust
let db_path = data_dir.join("mind_ledger.db");
```

**Updated:** Derives filename from tenant config:
```rust
let db_filename = soft_gloria_commands::tenant::get_tenant_db_filename();
let db_path = tenant_data_dir.join(db_filename);
```

(Previously: hardcoded "mind_ledger.db")

#### Scenario: Gloria Once build
- GIVEN `crypto.dbFileName = "mind_ledger_gloria_once.db"`
- WHEN app initializes
- THEN database file is `mind_ledger_gloria_once.db`

#### Scenario: Default build
- GIVEN fallback returns "mind_ledger.db"
- WHEN app initializes
- THEN database file is `mind_ledger.db`

---

### REQ-CRYPTO-003: Data Directory from Tenant ID (MODIFIED)
**Previously:** `main.rs` used app data dir directly:
```rust
let data_dir = app.path().app_data_dir()?;
let db_path = data_dir.join("mind_ledger.db");
```

**Updated:** Creates tenant-specific subdirectory:
```rust
let data_dir = app.path().app_data_dir()?;
let tenant_id = soft_gloria_commands::tenant::get_tenant_id();
let tenant_data_dir = data_dir.join(format!("mind-ledger-{}", tenant_id));
std::fs::create_dir_all(&tenant_data_dir)?;

let db_filename = soft_gloria_commands::tenant::get_tenant_db_filename();
let db_path = tenant_data_dir.join(db_filename);
```

(Previously: used app data dir directly)

#### Scenario: Gloria Once build
- GIVEN tenant_id = "gloria-once"
- WHEN app initializes
- THEN data dir = `$APPDATA/mind-ledger-gloria-once/`

#### Scenario: Default build
- GIVEN tenant_id = "default"
- WHEN app initializes
- THEN data dir = `$APPDATA/mind-ledger/`

---

### REQ-CRYPTO-004: create_pool_with_key Used (MODIFIED)
**Previously:** `main.rs` used `create_pool` which internally created its own key manager with hardcoded params.

**Updated:** `main.rs` gets key explicitly and uses `create_pool_with_key`:
```rust
let key = key_manager.get_or_create_key()?;
let db = create_pool_with_key(&db_path, &key)?;
```

(Previously: `create_pool(&db_path, &data_dir)`)

#### Scenario: Key obtained from tenant-specific keyring
- GIVEN key_manager configured with tenant service/account
- WHEN get_or_create_key() called
- THEN returns tenant-specific encryption key
- AND create_pool_with_key uses that key

---

## ADDED Requirements

### REQ-CRYPTO-005: Tenant Helper Functions (ADDED)
The `commands/src/tenant.rs` module SHALL provide these helper functions used by `main.rs`:

```rust
/// Returns the keyring service name from tenant config, or default
pub fn get_tenant_keyring_service() -> String

/// Returns the keyring account name from tenant config, or default
pub fn get_tenant_keyring_account() -> String

/// Returns the database filename from tenant config, or default
pub fn get_tenant_db_filename() -> String

/// Returns the tenant ID from tenant config, or default
pub fn get_tenant_id() -> String
```

Each function reads the embedded JSON at compile time via `include_str!`, deserializes, and returns the relevant field with a fallback default.

---

## REMOVED Requirements

*(None)*

---

## Scenarios

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Keyring service from config | Config has keyringService | main.rs init | Uses config value |
| Keyring account from config | Config has keyringAccount | main.rs init | Uses config value (tenant-specific) |
| DB filename from config | Config has dbFileName | main.rs init | Uses config value |
| Data dir from tenant ID | tenant_id = "gloria-once" | main.rs init | Creates mind-ledger-gloria-once/ |
| Default fallback | No config embedded | main.rs init | Uses hardcoded defaults |
| Key isolation | Two tenant builds | Both run | Separate keyring entries, no crossover |