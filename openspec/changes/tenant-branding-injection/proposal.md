# Proposal: Tenant Branding Injection for Gloria Once

## Intent

Implement white-label branding injection for the first tenant **Psic. Gloria Once** (Neuropsicóloga Clínica). Transform MindLedger from single-tenant hardcoded branding to a tenant-aware architecture where branding tokens, database isolation, and app identity are injected at build time via `TENANT_CONFIG` environment variable.

## Scope

### In Scope
- **Tenant config system**: Validate/integrate `tenant-configs/gloria-once.json` schema; add build.rs copy to OUT_DIR
- **Brand injection**: Runtime `get_tenant_config` Tauri command → `useTenantConfig` hook → CSS variable override in `:root`
- **Database isolation**: Per-tenant keyring account (`sqlcipher-key-gloria-once`), DB filename (`mind_ledger_gloria_once.db`), data directory (`$APPDATA/mind-ledger-gloria-once/`)
- **Build pipeline**: `TENANT_CONFIG` env var → build.rs embeds config → Tauri bundle ID + app name updated for Gloria Once
- **Frontend integration**: Dynamic Layout sidebar brand, login screen, window title from tenant config

### Out of Scope
- Multi-tenant runtime switching (single binary = single tenant per build)
- Tenant asset pipeline (logos, favicons) — deferred to WHITE_LABEL_GUIDE §6.1
- CI/CD automation for multi-tenant builds — deferred
- Settings persistence per tenant (uses existing `settings` table in isolated DB)

## Capabilities

### New Capabilities
- `tenant-config`: Tenant configuration schema, build-time embedding, runtime access via Tauri command
- `brand-injection`: CSS variable override from tenant config → dynamic UI theming (colors, typography, app title)
- `database-isolation`: Per-tenant SQLCipher keyring entry, DB file, and data directory isolation

### Modified Capabilities
- `tauri-app-setup`: Update `tauri.conf.json` bundle identifier → `com.mindledger.gloriaonce.desktop`, productName → `MindLedger - Psic. Gloria Once`
- `database-core`: Modify `SqlCipherKeyManager` to accept tenant-specific keyring account + DB filename from config
- `frontend-layout`: Layout.tsx sidebar brand text from `tenant.commercialName`; window title from `tenant.commercialName`
- `crypto-keyring`: Keyring service/account derived from `tenant-config` crypto section

## Approach

### Build-Time (Rust)
1. `build.rs` reads `TENANT_CONFIG` env var (path to `tenant-configs/gloria-once.json`)
2. Copies config to `OUT_DIR/app.config.json`; emits `cargo:rustc-env=TENANT_CONFIG_PATH=...`
3. `tauri.conf.json` templated via build script: `identifier`, `productName` from config

### Runtime (Rust → Frontend)
1. New `get_tenant_config` command in `commands/src/tenant.rs` reads embedded config
2. Frontend `useTenantConfig` hook (TanStack Query) invokes command on mount
3. `useEffect` injects `branding.colors.*` as HSL CSS variables on `document.documentElement`
4. Tailwind `tailwind.config.js` uses CSS variables — components auto-theme

### Database Isolation
- `SqlCipherKeyManager::new_with_fallback(service, account, data_dir)` already supports per-tenant account
- `main.rs` derives `tenant_id` from config → constructs data dir + keyring account + DB path

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `tenant-configs/gloria-once.json` | Validated | Tenant config schema (already exists) |
| `src-tauri/build.rs` | Modified | Copy tenant config to OUT_DIR, emit env var |
| `src-tauri/commands/src/tenant.rs` | New | `get_tenant_config` Tauri command + helpers |
| `src-tauri/commands/src/lib.rs` | Modified | Export tenant module |
| `src-tauri/Cargo.toml` | Modified | Add `serde_json` build dep (already added) |
| `src-tauri/tauri.conf.json` | Templated | `identifier`, `productName` from tenant config |
| `src-tauri/infrastructure/src/database.rs` | Modified | Per-tenant keyring account, DB filename, data dir |
| `src-tauri/app/src/main.rs` | Modified | Initialize DB pool with tenant-specific paths |
| `src/hooks/useTenantConfig.ts` | New | TanStack Query hook for tenant config |
| `src/index.css` | Modified | CSS variable defaults + dynamic injection point |
| `tailwind.config.js` | Modified | Brand color scales reference CSS variables |
| `src/components/layout/Layout.tsx` | Modified | Dynamic sidebar brand, window title |
| `src/pages/LoginPage.tsx` | Modified | Dynamic login branding (deferred if not exists) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| CSS variable specificity conflicts with Tailwind | Medium | Use `@layer base` for `:root` vars; test all components |
| Breaking existing single-tenant default behavior | High | Keep `default.json` fallback; `TENANT_CONFIG` optional |
| Keyring entry collision across tenants | Low | Unique account per tenant (`sqlcipher-key-{tenant_id}`) |
| Tauri bundle ID mismatch on macOS codesign | Medium | Template `tauri.conf.json` in build.rs before `tauri-build` |
| Frontend flash of default colors before config loads | Low | SSR not applicable; use `staleTime: Infinity`, show skeleton |

## Rollback Plan

1. **Git revert**: `git revert <merge-commit>` on main branch
2. **Build.rs**: Remove tenant config copy logic; restore original `tauri.conf.json` values
3. **Database.rs**: Revert `SqlCipherKeyManager` to hardcoded `mind-ledger` service + `sqlcipher-key` account
4. **Commands**: Remove `tenant.rs` module and `get_tenant_config` command registration
5. **Frontend**: Remove `useTenantConfig` hook, CSS variable injection, dynamic Layout branding
6. **Config**: Delete `tenant-configs/gloria-once.json` (or keep as reference)

## Dependencies

- Existing: `tauri-build`, `serde_json` (build dep), `keyring` crate, `dirs` crate
- No new external dependencies required

## Success Criteria

- [ ] `TENANT_CONFIG=../tenant-configs/gloria-once.json cargo tauri build` produces app with:
  - Bundle ID: `com.mindledger.gloriaonce.desktop`
  - App name: `MindLedger - Psic. Gloria Once`
  - Window title: `MindLedger - Psic. Gloria Once`
- [ ] Sidebar shows "MindLedger - Psic. Gloria Once" with "Neuropsicóloga Clínica" subtitle
- [ ] Primary color `#1A5F60` (Teal) applied to buttons, sidebar, focus rings
- [ ] Secondary `#E5F1EE` (Sage) on metric cards, hover states
- [ ] Accent `#E3645F` (Coral) on destructive actions only
- [ ] Dark mode colors from `brandDark` section work correctly
- [ ] Database created at `$APPDATA/mind-ledger-gloria-once/mind_ledger_gloria_once.db`
- [ ] SQLCipher key stored in keyring under `mind-ledger` / `sqlcipher-key-gloria-once`
- [ ] Running Gloria Once build alongside default build shows zero data crossover
- [ ] All existing tests pass (74+ tests)
- [ ] No TypeScript errors, no Rust clippy warnings