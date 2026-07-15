# Tauri App Setup — Delta Spec

## MODIFIED Requirements

### REQ-TAURI-001: Dynamic Bundle Identifier (MODIFIED)
**Previously:** `tauri.conf.json` had hardcoded `"identifier": "com.mindledger.desktop"`.

**Updated:** The bundle identifier SHALL be templated from tenant config `tenant.id` at build time via `build.rs`.

**Template:** `com.mindledger.{tenant.id}.desktop`

**Build.rs Implementation:**
```rust
// In build.rs after copying tenant config
if let Ok(config_str) = std::fs::read_to_string(&config_path) {
    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
        if let Some(tenant_id) = config.get("tenant").and_then(|t| t.get("id")).and_then(|v| v.as_str()) {
            // Update tauri.conf.json
            let tauri_conf_path = Path::new("tauri.conf.json");
            if let Ok(mut conf) = std::fs::read_to_string(tauri_conf_path) {
                let mut json: serde_json::Value = serde_json::from_str(&conf).unwrap();
                json["identifier"] = serde_json::Value::String(format!("com.mindledger.{}.desktop", tenant_id));
                std::fs::write(tauri_conf_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
            }
        }
    }
}
```

**Given** tenant config with `tenant.id = "gloria-once"`  
**When** `cargo tauri build` runs  
**Then** `tauri.conf.json` identifier = `com.mindledger.gloria-once.desktop`  
**And** macOS bundle ID matches for codesigning

**Given** default build (no tenant config)  
**When** `cargo tauri build` runs  
**Then** identifier remains `com.mindledger.desktop`

(Previously: hardcoded `com.mindledger.desktop`)

---

### REQ-TAURI-002: Dynamic Product Name (MODIFIED)
**Previously:** `tauri.conf.json` had hardcoded `"productName": "MindLedger"`.

**Updated:** The product name SHALL be templated from `tenant.commercialName` at build time.

**Build.rs Implementation:** (in same block as above)
```rust
json["productName"] = serde_json::Value::String(config["tenant"]["commercialName"].as_str().unwrap().to_string());
```

**Given** `tenant.commercialName = "MindLedger - Psic. Gloria Once"`  
**When** `cargo tauri build` runs  
**Then** `tauri.conf.json` productName = `"MindLedger - Psic. Gloria Once"`  
**And** built app shows this name in Finder/Explorer/Applications

**Given** default build  
**When** build runs  
**Then** productName = `"MindLedger"`

(Previously: hardcoded `"MindLedger"`)

---

### REQ-TAURI-003: Dynamic Window Title Default (MODIFIED)
**Previously:** `tauri.conf.json` had hardcoded window `"title": "MindLedger"`.

**Updated:** The default window title SHALL be templated from `tenant.commercialName`. Note: runtime override via REQ-BI-004 also applies.

**Build.rs Implementation:**
```rust
json["app"]["windows"][0]["title"] = serde_json::Value::String(
    config["tenant"]["commercialName"].as_str().unwrap().to_string()
);
```

**Given** `tenant.commercialName = "MindLedger - Psic. Gloria Once"`  
**When** build runs  
**Then** `tauri.conf.json` window title = `"MindLedger - Psic. Gloria Once"`

(Previously: hardcoded `"MindLedger"`)

---

## REMOVED Requirements

*(None)*

---

## ADDED Requirements

*(None — all changes are MODIFIED to existing requirements)*

---

## Scenarios

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Gloria Once bundle ID | tenant.id = "gloria-once" | cargo tauri build | identifier = com.mindledger.gloria-once.desktop |
| Gloria Once product name | tenant.commercialName set | cargo tauri build | productName = "MindLedger - Psic. Gloria Once" |
| Default build | No TENANT_CONFIG | cargo tauri build | identifier = com.mindledger.desktop, productName = "MindLedger" |
| macOS codesign | Built with Gloria Once config | codesign --verify | Bundle ID matches provisioning profile |