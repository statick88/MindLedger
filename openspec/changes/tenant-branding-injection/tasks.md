# Tasks: Tenant Branding Injection for Gloria Once

## Phase Overview

| Phase | Description | Tasks |
|-------|-------------|-------|
| **Phase 0** | Validation of pre-existing inline work | 0.1–0.5 |
| **Phase 1** | Rust Backend — Database Isolation & App Init | 1.1–1.4 |
| **Phase 2** | Tauri Commands — Runtime Config Exposure | 2.1–2.3 |
| **Phase 3** | Build Pipeline — tauri.conf.json Templating | 3.1–3.3 |
| **Phase 4** | Frontend — CSS Variables & Tailwind Integration | 4.1–4.5 |
| **Phase 5** | Frontend — Layout & App Integration | 5.1–5.4 |
| **Phase 6** | Feature Flags & Conditional Routing | 6.1–6.2 |
| **Phase 7** | Verification & Testing | 7.1–7.6 |

---

## Phase 0: Validate Pre-Existing Inline Work

These items were created before SDD. Mark as **DONE** after verification.

### Task 0.1: Validate `tenant-configs/gloria-once.json` schema
- **Status**: ✅ DONE (marked in proposal)
- **Verify**: JSON validates against `TenantConfig` schema in design.md §3.1
- **Files**: `tenant-configs/gloria-once.json`
- **Acceptance**: `serde_json::from_str` succeeds; all required fields present (tenant, brand, brandDark, typography, crypto, features)

### Task 0.2: Validate `src-tauri/build.rs` modifications
- **Status**: ⏳ NEEDS VALIDATION
- **Verify**: 
  - Reads `TENANT_CONFIG` env var (defaults to `tenant-configs/gloria-once.json`)
  - Copies config to `OUT_DIR/tenant-config.json`
  - Emits `cargo:rustc-env=TENANT_CONFIG_PATH=...`
  - Validates JSON via `serde_json`
  - Templates `tauri.conf.json` (identifier, productName, window.title)
- **Files**: `src-tauri/build.rs`
- **Acceptance**: `cargo build` with `TENANT_CONFIG=../tenant-configs/gloria-once.json` produces correct `tauri.conf.json` values

### Task 0.3: Validate `src-tauri/commands/src/tenant.rs` module
- **Status**: ⏳ NEEDS VALIDATION
- **Verify**:
  - `TenantConfig`, `BrandTokens`, `TypographyConfig`, `CryptoConfig`, `FeatureFlags`, `TenantInfo` structs match design.md §3.1
  - `get_tenant_config()` command reads embedded config via `include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"))`
  - Helper functions: `get_tenant_keyring_account()`, `get_tenant_db_filename()`, `get_tenant_id()`
  - Fallback to `include_str!("../../../tenant-configs/default.json")` if embedded missing
- **Files**: `src-tauri/commands/src/tenant.rs`
- **Acceptance**: `cargo test -p soft_gloria_commands` passes; command returns valid JSON

### Task 0.4: Validate `src-tauri/commands/src/lib.rs` export
- **Status**: ⏳ NEEDS VALIDATION
- **Verify**: `pub use tenant::*;` or equivalent exports all public items
- **Files**: `src-tauri/commands/src/lib.rs`
- **Acceptance**: `use soft_gloria_commands::tenant::get_tenant_config;` compiles in `main.rs`

### Task 0.5: Validate `src-tauri/Cargo.toml` build dependency
- **Status**: ⏳ NEEDS VALIDATION
- **Verify**: `serde_json = "1.0"` under `[build-dependencies]`
- **Files**: `src-tauri/Cargo.toml`
- **Acceptance**: `cargo build` succeeds with build.rs using serde_json

---

## Phase 1: Rust Backend — Database Isolation & App Init

### Task 1.1: Modify `database.rs` — Add `create_pool_for_tenant()`
- **Dependencies**: Task 0.3 (tenant helpers exist)
- **Complexity**: Medium
- **Files**: `src-tauri/infrastructure/src/database.rs`
- **Changes**:
  - Add `create_pool_for_tenant(data_dir, keyring_account, db_filename) -> Result<DbPool>`
  - Accept tenant-specific `keyring_account` and `db_filename`
  - Keep `create_pool()` as backward-compatible wrapper calling `create_pool_for_tenant` with defaults
  - Use existing `SqlCipherKeyManager::new_with_fallback(service, account, data_dir)`
- **Acceptance**: 
  - Unit test `test_create_pool_for_tenant_isolation` passes (design.md §9.1)
  - No breaking changes to existing `create_pool()` callers

### Task 1.2: Modify `main.rs` — Tenant-Aware Initialization
- **Dependencies**: Task 1.1, Task 0.3
- **Complexity**: Medium
- **Files**: `src-tauri/app/src/main.rs`
- **Changes**:
  - Import tenant helpers: `get_tenant_config`, `get_tenant_keyring_account`, `get_tenant_db_filename`, `get_tenant_id`
  - In `setup()`: call `get_tenant_config()` synchronously at startup
  - Derive `data_dir = app_data_dir.join("mind-ledger-{tenant_id}")`
  - Create directory if missing
  - Call `create_pool_for_tenant(&data_dir, &keyring_account, &db_filename)`
  - Run migrations on the tenant pool
  - Register `get_tenant_config` in `invoke_handler!`
- **Acceptance**: 
  - App starts without panic with `TENANT_CONFIG=../tenant-configs/gloria-once.json`
  - DB created at `$APPDATA/mind-ledger-gloria-once/mind_ledger_gloria_once.db`
  - Keyring entry at `mind-ledger` / `sqlcipher-key-gloria-once`

### Task 1.3: Add `default.json` fallback config
- **Dependencies**: None
- **Complexity**: Low
- **Files**: `tenant-configs/default.json` (new)
- **Changes**: Create minimal valid tenant config matching current hardcoded MindLedger defaults
- **Acceptance**: Build succeeds without `TENANT_CONFIG` env var; app runs with default branding

### Task 1.4: Verify Database Isolation (Integration)
- **Dependencies**: Task 1.2, Task 1.3
- **Complexity**: Medium
- **Files**: N/A (manual verification)
- **Steps**:
  1. Build with `TENANT_CONFIG=../tenant-configs/gloria-once.json`
  2. Run app → verify DB at `mind-ledger-gloria-once/mind_ledger_gloria_once.db`
  3. Build default (no TENANT_CONFIG)
  4. Run app → verify DB at `mind-ledger/mind_ledger.db`
  5. Confirm zero data crossover (different keyring accounts, different DB files, different dirs)
- **Acceptance**: Both builds run independently with isolated data

---

## Phase 2: Tauri Commands — Runtime Config Exposure

### Task 2.1: Ensure `get_tenant_config` command registration
- **Dependencies**: Task 0.3, Task 0.4
- **Complexity**: Low
- **Files**: `src-tauri/app/src/main.rs`
- **Changes**: Confirm `get_tenant_config` is in `tauri::generate_handler![...]`
- **Acceptance**: `invoke('get_tenant_config')` from frontend returns valid JSON

### Task 2.2: Add `get_tenant_id`, `get_tenant_keyring_account`, `get_tenant_db_filename` commands (optional)
- **Dependencies**: Task 0.3
- **Complexity**: Low
- **Files**: `src-tauri/commands/src/tenant.rs`, `src-tauri/commands/src/lib.rs`, `src-tauri/app/src/main.rs`
- **Changes**: Export helpers as Tauri commands if frontend needs them directly (currently only `get_tenant_config` needed)
- **Acceptance**: Commands invokable from frontend (test via devtools)

### Task 2.3: TypeScript Types for Tenant Config
- **Dependencies**: Task 0.3
- **Complexity**: Low
- **Files**: `src/types/tenant.ts` (new)
- **Changes**: Mirror Rust structs from design.md §3.1 exactly
- **Acceptance**: `useTenantConfig` hook returns fully typed `TenantConfig`

---

## Phase 3: Build Pipeline — tauri.conf.json Templating

### Task 3.1: Verify build.rs tauri.conf.json templating works
- **Dependencies**: Task 0.2
- **Complexity**: Medium
- **Files**: `src-tauri/build.rs`, `src-tauri/tauri.conf.json`
- **Verify**: 
  - `identifier` → `com.mindledger.gloriaonce.desktop`
  - `productName` → `MindLedger - Psic. Gloria Once`
  - `app.windows[0].title` → `MindLedger - Psic. Gloria Once`
- **Acceptance**: Built `.app`/`.exe` has correct bundle ID and window title

### Task 3.2: Ensure tauri.conf.json is tracked but template-safe
- **Dependencies**: Task 3.1
- **Complexity**: Low
- **Files**: `src-tauri/tauri.conf.json`, `.gitignore`
- **Changes**: 
  - Keep `tauri.conf.json` in git (templated at build time)
  - Add `src-tauri/tauri.conf.json.bak` to gitignore if backup created
- **Acceptance**: `git diff src-tauri/tauri.conf.json` shows templated values after build; `git checkout -- src-tauri/tauri.conf.json` restores template

### Task 3.3: Document build commands in README
- **Dependencies**: Task 3.1
- **Complexity**: Low
- **Files**: `README.md` or `BUILD.md`
- **Changes**: Document `TENANT_CONFIG=../tenant-configs/gloria-once.json cargo tauri build`
- **Acceptance**: New developer can build Gloria Once binary from docs

---

## Phase 4: Frontend — CSS Variables & Tailwind Integration

### Task 4.1: Update `src/index.css` — CSS Variable Defaults + Dark Mode Injection Pattern
- **Dependencies**: None
- **Complexity**: Medium
- **Files**: `src/index.css`
- **Changes**:
  - Define `:root` CSS variables with current MindLedger defaults (design.md §4.2)
  - Define `.dark` overrides with current dark mode defaults
  - Add `.dark` selector mapping `--*-dark` variables to base variables (design.md §4.2 lines 475-493)
  - Use `@layer base` for proper Tailwind precedence
- **Acceptance**: 
  - App loads with current branding when no tenant config
  - Dark mode toggle works with defaults

### Task 4.2: Create `src/lib/color-utils.ts` — HEX to HSL Conversion
- **Dependencies**: None
- **Complexity**: Low
- **Files**: `src/lib/color-utils.ts` (new)
- **Changes**: Implement `hexToHsl(hex: string): string` from design.md §3.2
- **Acceptance**: Unit test converts `#1A5F60` → `192 72% 21%`

### Task 4.3: Update `tailwind.config.js` — Reference CSS Variables
- **Dependencies**: Task 4.1
- **Complexity**: Medium
- **Files**: `tailwind.config.js`
- **Changes**: 
  - All `colors.*` values reference `hsl(var(--css-variable))` (design.md §4.3)
  - `fontFamily.sans` → `var(--font-family)`
  - `fontWeight.heading` → `var(--heading-weight)`
  - `fontWeight.body` → `var(--body-weight)`
  - `borderRadius` → `var(--radius)` etc.
- **Acceptance**: 
  - `npm run build` compiles CSS without errors
  - Changing CSS variables at runtime updates all Tailwind components

### Task 4.4: Create `useTenantConfig` Hook
- **Dependencies**: Task 2.3
- **Complexity**: Low
- **Files**: `src/hooks/useTenantConfig.ts` (new)
- **Changes**: Implement per design.md §7.1 with `staleTime: Infinity`, `gcTime: Infinity`
- **Acceptance**: Hook returns typed `TenantConfig`; caches forever

### Task 4.5: Modify `src/App.tsx` — CSS Variable Injection + Title
- **Dependencies**: Task 4.1, Task 4.2, Task 4.4
- **Complexity**: Medium
- **Files**: `src/App.tsx`
- **Changes**:
  - Call `useTenantConfig()` at root
  - `useEffect` on `tenantConfig`: inject all `brand.*` as `--kebab-case-key` HSL values
  - Inject `brandDark.*` as `--kebab-case-key-dark` HSL values
  - Inject typography vars: `--font-family`, `--heading-weight`, `--body-weight`
  - Set `document.title = tenantConfig.tenant.commercialName`
  - Show skeleton while loading (`isLoading`)
  - Pass `tenantConfig` to `<Layout />`
- **Acceptance**: 
  - App loads with Gloria Once colors without flash
  - Window title shows "MindLedger - Psic. Gloria Once"
  - Dark mode uses `brandDark` values

---

## Phase 5: Frontend — Layout & App Integration

### Task 5.1: Modify `src/components/layout/Layout.tsx` — Dynamic Branding
- **Dependencies**: Task 4.5
- **Complexity**: Low
- **Files**: `src/components/layout/Layout.tsx`
- **Changes**:
  - Accept `tenantConfig?: TenantConfig | null` prop
  - Extract `brandName = tenantConfig?.tenant.commercialName ?? 'MindLedger'`
  - Extract `subtitle = tenantConfig?.tenant.clinicalRole ?? 'Clinical Psychology'`
  - Pass to `<Sidebar brandName={brandName} subtitle={subtitle} />`
- **Acceptance**: Sidebar header shows "MindLedger - Psic. Gloria Once" / "Neuropsicóloga Clínica"

### Task 5.2: Modify `src/components/ui/sidebar.tsx` — Accept Brand Props
- **Dependencies**: Task 5.1
- **Complexity**: Low
- **Files**: `src/components/ui/sidebar.tsx`
- **Changes**:
  - Add `brandName: string`, `subtitle: string` props
  - Replace hardcoded "MindLedger" / "Clinical Psychology" with props
- **Acceptance**: Sidebar renders dynamic brand correctly

### Task 5.3: Create `src/pages/LoginPage.tsx` — Dynamic Login Branding (Optional)
- **Dependencies**: Task 4.5
- **Complexity**: Low
- **Files**: `src/pages/LoginPage.tsx` (new or modify existing)
- **Changes**: Use `useTenantConfig()` to render dynamic title/subtitle on login screen
- **Acceptance**: Login page shows Gloria Once branding

### Task 5.4: Add Loading Skeleton Component
- **Dependencies**: Task 4.5
- **Complexity**: Low
- **Files**: `src/components/ui/AppSkeleton.tsx` (new)
- **Changes**: Simple skeleton matching app layout for `isLoading` state
- **Acceptance**: No layout shift during config load

---

## Phase 6: Feature Flags & Conditional Routing

### Task 6.1: Create `useFeatureFlags` Hook
- **Dependencies**: Task 4.4
- **Complexity**: Low
- **Files**: `src/hooks/useFeatureFlags.ts` (new)
- **Changes**: Implement per design.md §8.1 — reads `tenantConfig.features` with defaults
- **Acceptance**: Hook returns typed feature flag object

### Task 6.2: Conditional Route Rendering in `App.tsx`
- **Dependencies**: Task 6.1
- **Complexity**: Low
- **Files**: `src/App.tsx`
- **Changes**: Wrap route definitions with feature flag checks (design.md §8.2)
- **Acceptance**: Disabled features' routes return 404 / not rendered

---

## Phase 7: Verification & Testing

### Task 7.1: Rust Unit Tests
- **Dependencies**: Task 1.1, Task 0.3
- **Complexity**: Medium
- **Files**: `src-tauri/commands/src/tenant.rs`, `src-tauri/infrastructure/src/database.rs`
- **Run**: `cargo test -p soft_gloria_commands -p soft_gloria_infrastructure`
- **Acceptance**: All tests pass including new isolation tests

### Task 7.2: Frontend Unit Tests
- **Dependencies**: Task 4.4, Task 6.1
- **Complexity**: Medium
- **Files**: `src/hooks/__tests__/useTenantConfig.test.tsx`, `src/hooks/__tests__/useFeatureFlags.test.tsx`
- **Run**: `npm test`
- **Acceptance**: Hooks return expected data; color conversion tested

### Task 7.3: TypeScript Type Check
- **Dependencies**: Task 2.3, Task 4.3, Task 4.5
- **Complexity**: Low
- **Run**: `npm run typecheck` (or `tsc --noEmit`)
- **Acceptance**: Zero errors

### Task 7.4: Lint & Format
- **Dependencies**: All code tasks
- **Complexity**: Low
- **Run**: `cargo clippy -- -D warnings` + `npm run lint`
- **Acceptance**: Zero warnings/errors

### Task 7.5: Build Verification — Gloria Once
- **Dependencies**: All prior tasks
- **Complexity**: Medium
- **Command**: `TENANT_CONFIG=../tenant-configs/gloria-once.json cargo tauri build`
- **Acceptance Criteria** (from proposal §Success Criteria):
  - [ ] Bundle ID: `com.mindledger.gloriaonce.desktop`
  - [ ] App name: `MindLedger - Psic. Gloria Once`
  - [ ] Window title: `MindLedger - Psic. Gloria Once`
  - [ ] Sidebar shows "MindLedger - Psic. Gloria Once" + "Neuropsicóloga Clínica"
  - [ ] Primary `#1A5F60` (Teal) on buttons, sidebar, focus rings
  - [ ] Secondary `#E5F1EE` (Sage) on metric cards, hover states
  - [ ] Accent `#E3645F` (Coral) on destructive actions only
  - [ ] Dark mode colors from `brandDark` work correctly
  - [ ] DB at `$APPDATA/mind-ledger-gloria-once/mind_ledger_gloria_once.db`
  - [ ] Keyring: `mind-ledger` / `sqlcipher-key-gloria-once`
  - [ ] Zero data crossover with default build

### Task 7.6: Build Verification — Default (Regression)
- **Dependencies**: Task 1.3, Task 7.5
- **Complexity**: Low
- **Command**: `cargo tauri build` (no TENANT_CONFIG)
- **Acceptance**: 
  - App builds and runs with default MindLedger branding
  - DB at `$APPDATA/mind-ledger/mind_ledger.db`
  - Keyring: `mind-ledger` / `sqlcipher-key`
  - All existing tests pass (74+ tests)

---

## Task Dependency Graph

```
Phase 0 (Validate)
  0.1 ──┐
  0.2 ──┤
  0.3 ──┼──→ Phase 1
  0.4 ──┤
  0.5 ──┘

Phase 1 (Backend)
  1.1 ← 0.3
  1.2 ← 1.1, 0.3
  1.3 (independent)
  1.4 ← 1.2, 1.3

Phase 2 (Commands)
  2.1 ← 0.3, 0.4
  2.2 ← 0.3 (optional)
  2.3 ← 0.3

Phase 3 (Build)
  3.1 ← 0.2
  3.2 ← 3.1
  3.3 ← 3.1

Phase 4 (Frontend CSS)
  4.1 (independent)
  4.2 (independent)
  4.3 ← 4.1
  4.4 ← 2.3
  4.5 ← 4.1, 4.2, 4.4

Phase 5 (Frontend Layout)
  5.1 ← 4.5
  5.2 ← 5.1
  5.3 ← 4.5 (optional)
  5.4 ← 4.5

Phase 6 (Feature Flags)
  6.1 ← 4.4
  6.2 ← 6.1

Phase 7 (Verify)
  7.1 ← 1.1, 0.3
  7.2 ← 4.4, 6.1
  7.3 ← 2.3, 4.3, 4.5
  7.4 ← all code tasks
  7.5 ← all prior
  7.6 ← 1.3, 7.5
```

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Done (validated) | 1 |
| ⏳ Needs Validation | 4 |
| 📝 New Tasks | 27 |
| **Total** | **32** |

**Critical Path**: 0.2 → 0.3 → 1.1 → 1.2 → 4.1 → 4.3 → 4.5 → 5.1 → 7.5

**Estimated Effort**: ~2-3 days for full implementation and verification