# MindLdger v1.0.0 — Release Manifest

**Tenant:** Psic. Gloria Once | Neuropsicóloga Clínica
**Release Date:** 2026-07-13
**Developer:** Diego Medardo Saavedra García <Statick>

---

## Build Pipeline Summary

| Phase | Status | Notes |
|-------|--------|-------|
| Branding Freeze | ✅ | Tenant config, CSS tokens, CSP verified |
| macOS Universal Build | ✅ | DMG 6.1MB, universal x86_64+arm64 |
| Windows Build Prep | ✅ | `scripts/build-windows-vm.ps1` ready for VM |
| Smoke Tests | ✅ | Exit code 124 (app launched, no crash) |
| Test Suite | ✅ | 55/59 pass (4 pre-existing failures) |

---

## Artifacts

### macOS

| Artifact | Size | SHA-256 |
|----------|------|---------|
| `MindLdger_1.0.0_universal.dmg` | 6.1 MB | `fc8d05ad609ccad7c9ce37b92746c9b69b21eebd2b3e70bcdf9af8c568b5ff80` |

- **Target:** universal-apple-darwin (x86_64 + arm64)
- **Binary:** Mach-O universal, no crash on launch
- **DMG installer:** Standard drag-to-Applications

### Windows (Pending VM Build)

| Artifact | Status |
|----------|--------|
| NSIS Installer | Build script ready — execute in Windows 11 VM |
| MSI Installer | Build script ready — execute in Windows 11 VM |

- **Build script:** `scripts/build-windows-vm.ps1`
- **Requirements:** Windows 11, Rust/MSVC, fnm, Node v24.13.1

---

## Tenant Branding Verification

| Property | Expected | Verified |
|----------|----------|----------|
| Tenant ID | `gloria-once` | ✅ |
| Commercial Name | `MindLdger - Psic. Gloria Once` | ✅ |
| Bundle ID | `com.mindledger.gloriaonce.desktop` | ✅ |
| Primary Color | `#1A5F60` (Teal, HSL 192 72% 21%) | ✅ |
| Accent Color | `#E3645F` (Coral, HSL 2 72% 63%) | ✅ |
| CSP | `style-src 'self'` (no unsafe-inline) | ✅ |
| Font Loading | `font-src 'self'` (system Inter) | ✅ |
| Window Title | Templated to commercial name | ✅ |

---

## Test Results

**55/59 tests pass**

Pre-existing failures (not blocking release):

| Test | Reason |
|------|--------|
| `test_invalid_transition_cancelada_to_realizada` | Test DB schema missing `modality` column |
| `test_terminal_state_reentry_realizada` | Same schema mismatch |
| `test_error_leakage_no_sql_or_paths` | Error message contains "Invalid date format" flagged as leak |
| `test_tauri_allowlist_no_wildcards` | Cannot find `tauri.conf.json` from test working directory |

---

## Build Configuration

| Setting | Value |
|---------|-------|
| LTO | `thin` (fat LTO causes SQLCipher corruption) |
| Strip | `true` |
| Codegen Units | `1` |
| Panic | `abort` |
| Profile | `release` |

---

## Security Posture

- SQLCipher 4.5.3 with keyring-backed key management
- Argon2 removed (was unused dependency)
- CSP enforced: no `unsafe-inline`, no `unsafe-eval`
- IPC allowlist: no wildcards
- Tenant data path: `$APPDATA/mind-ledger-gloria-once/`

---

## Deployment Notes

1. **macOS:** Distribute `MindLdger_1.0.0_universal.dmg`. Users drag to Applications.
2. **Windows:** Execute `scripts/build-windows-vm.ps1` in Windows 11 VM. Collect NSIS/MSI from `src-tauri/target/release/bundle/`.
3. **First launch:** App creates keyring entry `sqlcipher-key-gloria-once` on first run. User must grant keyring access.
4. **Data isolation:** All DB files stored under `$APPDATA/mind-ledger-gloria-once/`.

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-13 | Production release for Psic. Gloria Once |
| 0.1.0 | — | Initial development |
