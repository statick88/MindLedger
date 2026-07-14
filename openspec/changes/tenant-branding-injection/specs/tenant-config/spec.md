# Tenant Config — Full Spec (New Capability)

## Purpose

Defines the tenant configuration schema, build-time embedding mechanism, and runtime access API for white-label tenant configurations.

## Requirements

### REQ-TC-001: Tenant Configuration JSON Schema
The system SHALL define a JSON schema for tenant configuration with the exact structure shown in `tenant-configs/gloria-once.json`.

**Schema Fields:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| tenant.id | string | YES | Kebab-case unique identifier |
| tenant.commercialName | string | YES | App title, window title, sidebar brand |
| tenant.clinicalRole | string | YES | Subtitle in sidebar |
| tenant.ownerName | string | YES | Owner display name |
| tenant.ownerTitle | string | YES | Owner professional title |
| brand.* | string (hex) | YES | 16 light mode color tokens |
| brandDark.* | string (hex) | YES | 16 dark mode color tokens |
| typography.fontFamily | string | YES | CSS font-family string |
| typography.headingWeight | string | YES | CSS font-weight for headings |
| typography.bodyWeight | string | YES | CSS font-weight for body |
| crypto.keyringService | string | YES | Keyring service name (e.g., "mind-ledger") |
| crypto.keyringAccount | string | YES | Keyring account (e.g., "sqlcipher-key-gloria-once") |
| crypto.dbFileName | string | YES | Database filename (e.g., "mind_ledger_gloria_once.db") |
| features.* | boolean | YES | Feature flags (clinicalNotes, accounting, agenda, diagnostics) |

**Given** valid tenant config JSON  
**When** parsed by build.rs  
**Then** validates as valid JSON matching schema

**Given** invalid JSON or missing required field  
**When** parsed by build.rs  
**Then** build fails with descriptive error

---

### REQ-TC-002: Build-Time Config Embedding
The `build.rs` SHALL read `TENANT_CONFIG` environment variable (path to tenant JSON), copy it to `OUT_DIR/tenant-config.json`, and emit `cargo:rustc-env=TENANT_CONFIG_PATH=...`.

**Process:**
1. Read `TENANT_CONFIG` env var (default: `tenant-configs/gloria-once.json`)
2. Validate file exists and is valid JSON
3. Copy to `$OUT_DIR/tenant-config.json`
4. Emit `cargo:rustc-env=TENANT_CONFIG_PATH=$OUT_DIR/tenant-config.json`

**Given** `TENANT_CONFIG=../tenant-configs/gloria-once.json cargo build`  
**When** build.rs runs  
**Then** `$OUT_DIR/tenant-config.json` exists with tenant config  
**And** `TENANT_CONFIG_PATH` env var available at compile time

**Given** `TENANT_CONFIG` not set (default build)  
**When** build.rs runs  
**Then** uses default path or skips embedding

---

### REQ-TC-003: Runtime Tauri Command get_tenant_config
The system SHALL expose a Tauri command `get_tenant_config` that returns the embedded tenant configuration to the frontend.

**Command Signature:**
```rust
#[tauri::command]
pub fn get_tenant_config() -> Result<TenantConfig, String>
```

**Implementation:**
```rust
pub fn get_tenant_config() -> Result<TenantConfig, String> {
    let config_str = include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"));
    serde_json::from_str(config_str).map_err(|e| format!("Parse error: {}", e))
}
```

**Given** tenant config embedded at build time  
**When** frontend invokes `invoke('get_tenant_config')`  
**Then** returns full `TenantConfig` object

**Given** no tenant config embedded (default build)  
**When** frontend invokes command  
**Then** returns error or default config

---

### REQ-TC-004: Frontend useTenantConfig Hook
The frontend SHALL provide a `useTenantConfig` hook using TanStack Query that fetches tenant config on mount.

**Hook:**
```typescript
export function useTenantConfig() {
  return useQuery({
    queryKey: ['tenantConfig'],
    queryFn: () => invoke<TenantConfig>('get_tenant_config'),
    staleTime: Infinity,
    retry: false,
  });
}
```

**Given** app mounts  
**When** `useTenantConfig()` called  
**Then** fetches config once, caches forever

**Given** config fetch fails  
**When** hook executes  
**Then** returns error state, UI shows fallback

---

### REQ-TC-005: Helper Functions for Rust Infrastructure
The `tenant.rs` module SHALL provide helper functions for infrastructure layer to access crypto settings without full deserialization.

**Functions:**
```rust
pub fn get_tenant_keyring_account() -> String
pub fn get_tenant_db_filename() -> String
pub fn get_tenant_id() -> String
```

**Given** embedded tenant config  
**When** `get_tenant_keyring_account()` called  
**Then** returns `crypto.keyringAccount` from config

**Given** embedded tenant config  
**When** `get_tenant_db_filename()` called  
**Then** returns `crypto.dbFileName` from config

---

## Scenarios

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Valid config embeds | TENANT_CONFIG points to valid JSON | cargo build | Config embedded, build succeeds |
| Invalid config fails | TENANT_CONFIG points to invalid JSON | cargo build | Build fails with parse error |
| Command returns config | Config embedded | invoke('get_tenant_config') | Returns TenantConfig |
| Hook caches result | useTenantConfig called twice | Component re-renders | Second call uses cache |
| Helpers extract crypto | Config embedded | get_tenant_keyring_account() | Returns keyringAccount |