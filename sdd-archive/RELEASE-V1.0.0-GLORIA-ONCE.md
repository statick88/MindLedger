# MindLedger v1.0.0-gloria-once - Release Notes

## Build Information
- **Date**: <!-- FILL: YYYY-MM-DD HH:MM:SS -->
- **Commit**: <!-- FILL: short hash from git rev-parse --short HEAD -->
- **Branch**: release/v1.0.0-gloria-once
- **Tenant**: gloria_once (Psic. Gloria Once)
- **Platform**: Windows 11 (MSVC native)

## Installer
- **Type**: MSI or EXE
- **File**: <!-- FILL: filename from src-tauri/target/release/bundle/ -->
- **Size**: <!-- FILL: MB -->
- **SHA-256**: <!-- FILL: hash from Get-FileHash -->

## Main Binary
- **File**: mindledger.exe
- **Size**: <!-- FILL: MB -->
- **SHA-256**: <!-- FILL: hash -->

## Tenant Configuration
- **Identifier**: com.mindldger.gloriaonce.desktop
- **ProductName**: Psic. Gloria Once
- **WindowTitle**: MindLedger - Psic. Gloria Once
- **CSP**: Intact (not modified by bundler)

## Build Profile
- **Rust**: <!-- FILL: rustc --version -->
- **Cargo**: <!-- FILL: cargo --version -->
- **Node**: v24.13.1
- **pnpm**: <!-- FILL: pnpm --version -->
- **LTO**: thin
- **Strip**: true

## Smoke Test Results
- [ ] Installer runs without errors
- [ ] Application launches with correct title
- [ ] SQLCipher initializes without pointer exceptions
- [ ] Teal/Sage color palette renders correctly in WebView2
- [ ] Window metadata shows "MindLedger - Psic. Gloria Once"
- [ ] Role displays "Neuropsicóloga Clínica"

## Build Commands Used
```powershell
# Sync repo
git pull origin release/v1.0.0-gloria-once

# Activate Node
fnm use 24.13.1

# Install dependencies
pnpm install

# Inject branding
python scripts/bundle-tenant.py tenants/gloria_once.json

# Build
pnpm tauri build

# Verify
Get-FileHash src-tauri\target\release\bundle\*.msi -Algorithm SHA256
```

## Notes
Automated build from CI/CD pipeline. All branding injected via
scripts/bundle-tenant.py — no core source code modifications required.
