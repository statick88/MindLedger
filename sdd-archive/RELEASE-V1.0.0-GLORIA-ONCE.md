# MindLedger v1.0.0-gloria-once — Release Certification

## Build Information
- **Date**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Commit**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Branch**: release/v1.0.0-gloria-once
- **Tenant**: gloria_once (Psic. Gloria Once)
- **Platform**: Windows 11 (MSVC native)

## Installer Artifact
- **Type**: MSI or EXE
- **File**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Size**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **SHA-256**: <!-- FILLED BY: scripts/windows-audit.ps1 -->

## Main Binary
- **File**: mindledger.exe
- **Size**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **SHA-256**: <!-- FILLED BY: scripts/windows-audit.ps1 -->

## Tenant Configuration
- **Identifier**: com.mindldger.gloriaonce.desktop
- **ProductName**: Psic. Gloria Once
- **WindowTitle**: MindLedger - Psic. Gloria Once
- **CSP**: Intact (not modified by bundler)

## Build Environment
- **Rust**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Cargo**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Node**: v24.13.1
- **pnpm**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **LTO**: thin
- **Strip**: true

## Security Audit Results

### PE Hardening (Static Binary Analysis)
<!-- FILLED BY: scripts/windows-audit.ps1 — ASLR, DEP/NX, HIGH_ENTROPY_VA, CFG checks -->

### String Scanning (Sensitive Data Exposure)
<!-- FILLED BY: scripts/windows-audit.ps1 — hardcoded key, hex key, plaintext checks -->

### Runtime Verification (Smoke Test)
<!-- FILLED BY: scripts/windows-audit.ps1 — app data dir, SQLCipher, window title, WAL files -->

### Configuration & Metadata
<!-- FILLED BY: scripts/windows-audit.ps1 — identifier, productName, CSP, frontend config, .env -->

## Audit Summary
- **Total Checks**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Passed**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Failed**: <!-- FILLED BY: scripts/windows-audit.ps1 -->
- **Warnings**: <!-- FILLED BY: scripts/windows-audit.ps1 -->

## Smoke Test Checklist
- [ ] Installer runs without errors
- [ ] Application launches with correct title
- [ ] SQLCipher initializes without pointer exceptions
- [ ] Teal/Sage color palette renders correctly in WebView2
- [ ] Window metadata shows "MindLedger - Psic. Gloria Once"
- [ ] Role displays "Neuropsicóloga Clínica"
- [ ] Medical appointment CRUD works (create, update status)
- [ ] Revenue recalculation triggers on status change
- [ ] Double-entry bookkeeping posts correctly
- [ ] No UI thread blocking during financial operations

## Certification
<!-- FILLED BY: scripts/windows-audit.ps1 after all checks pass -->

## Build Commands Used
```powershell
# Step 1: Build
git pull origin release/v1.0.0-gloria-once
fnm use 24.13.1
pnpm install
python scripts/bundle-tenant.py tenants/gloria_once.json
pnpm tauri build

# Step 2: Audit & Certify
powershell -ExecutionPolicy Bypass -File scripts/windows-audit.ps1

# Step 3: Commit results
git add sdd-archive/RELEASE-V1.0.0-GLORIA-ONCE.md
git commit -m "audit: certify secure windows runtime build and freeze sha256 artifact hashes for v1.0.0-gloria-once"
git push origin release/v1.0.0-gloria-once
```

## Notes
Automated build + audit pipeline. All branding injected via
scripts/bundle-tenant.py. Security audit via scripts/windows-audit.ps1.
No core source code modifications required.
