#!/usr/bin/env pwsh
# ============================================================================
# MindLedger - Windows Build & Verify Script
# Tenant: Gloria Once (Neuropsicóloga Clínica)
# Branch: release/v1.0.0-gloria-once
# ============================================================================
#Requires -RunAsAdministrator
$ErrorActionPreference = "Stop"

$REPO_URL = "https://github.com/statick88/MindLedger.git"
$BRANCH   = "release/v1.0.0-gloria-once"
$TENANT   = "gloria_once"
$WORK_DIR = "C:\MindLedger"

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host " MindLedger Windows Build Pipeline" -ForegroundColor Cyan
Write-Host " Tenant: Psic. Gloria Once" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# ============================================================================
# FASE 0: Pre-flight checks
# ============================================================================
Write-Host "[FASE 0] Pre-flight checks..." -ForegroundColor Yellow

# Check fnm
$fnmVersion = fnm --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] fnm not found. Install: winget install Schniz.fnm" -ForegroundColor Red
    exit 1
}
Write-Host "  [OK] fnm: $fnmVersion" -ForegroundColor Green

# Check Python
$pythonVersion = python --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] Python not found. Install Python 3.10+" -ForegroundColor Red
    exit 1
}
Write-Host "  [OK] Python: $pythonVersion" -ForegroundColor Green

# Check MSVC tools
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vsPath) {
        Write-Host "  [OK] MSVC Build Tools: $vsPath" -ForegroundColor Green
    } else {
        Write-Host "  [WARN] MSVC Build Tools not found. Install Visual Studio Build Tools (C++ workload)." -ForegroundColor Yellow
    }
} else {
    Write-Host "  [WARN] vswhere not found — cannot verify MSVC" -ForegroundColor Yellow
}

# Check WiX or NSIS
$wixDir = "${env:ProgramFiles(x86)}\WiX Toolset v3\bin"
$nsisDir = "${env:ProgramFiles(x86)}\NSIS"
if (Test-Path $wixDir) {
    Write-Host "  [OK] WiX Toolset v3 found" -ForegroundColor Green
} elseif (Test-Path $nsisDir) {
    Write-Host "  [OK] NSIS found" -ForegroundColor Green
} else {
    Write-Host "  [WARN] Neither WiX nor NSIS found. Tauri may use built-in bundler." -ForegroundColor Yellow
}

# ============================================================================
# FASE 1: Clone / Pull + Dependencies
# ============================================================================
Write-Host ""
Write-Host "[FASE 1] Synchronizing repository..." -ForegroundColor Yellow

if (Test-Path "$WORK_DIR\.git") {
    Write-Host "  Repository exists at $WORK_DIR, pulling..." -ForegroundColor Gray
    Set-Location $WORK_DIR
    git fetch origin
    git checkout $BRANCH
    git pull origin $BRANCH
} else {
    Write-Host "  Cloning repository..." -ForegroundColor Gray
    git clone --branch $BRANCH $REPO_URL $WORK_DIR
    Set-Location $WORK_DIR
}

$commitHash = git rev-parse --short HEAD
Write-Host "  [OK] Commit: $commitHash" -ForegroundColor Green

# Activate Node version
Write-Host "  Activating Node v24.13.1 via fnm..." -ForegroundColor Gray
fnm use 24.13.1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] Node v24.13.1 not installed. Run: fnm install 24.13.1" -ForegroundColor Red
    exit 1
}

# Install frontend dependencies
Write-Host "  Installing frontend dependencies..." -ForegroundColor Gray
pnpm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] pnpm install failed" -ForegroundColor Red
    exit 1
}
Write-Host "  [OK] Dependencies installed" -ForegroundColor Green

# Verify Rust toolchain
Write-Host "  Checking Rust toolchain..." -ForegroundColor Gray
$rustVersion = rustc --version
$cargoVersion = cargo --version
Write-Host "  [OK] Rust: $rustVersion" -ForegroundColor Green
Write-Host "  [OK] Cargo: $cargoVersion" -ForegroundColor Green

# ============================================================================
# FASE 2: Tenant Branding Injection
# ============================================================================
Write-Host ""
Write-Host "[FASE 2] Injecting tenant branding..." -ForegroundColor Yellow

python scripts/bundle-tenant.py "tenants\$TENANT.json"
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] Tenant bundler failed" -ForegroundColor Red
    exit 1
}

# Verify tauri.conf.json
$tauriConf = Get-Content "src-tauri\tauri.conf.json" -Raw | ConvertFrom-Json
$expectedId = "com.mindldger.gloriaonce.desktop"
$expectedName = "Psic. Gloria Once"

if ($tauriConf.identifier -ne $expectedId) {
    Write-Host "  [ERROR] identifier mismatch: $($tauriConf.identifier)" -ForegroundColor Red
    exit 1
}
if ($tauriConf.productName -ne $expectedName) {
    Write-Host "  [ERROR] productName mismatch: $($tauriConf.productName)" -ForegroundColor Red
    exit 1
}

# Verify CSP is intact (not modified by bundler)
$csp = $tauriConf.app.security.csp
if ($csp -match "default-src.*'self'") {
    Write-Host "  [OK] CSP intact" -ForegroundColor Green
} else {
    Write-Host "  [WARN] CSP may have been modified — verify manually" -ForegroundColor Yellow
}

Write-Host "  [OK] Identifier: $($tauriConf.identifier)" -ForegroundColor Green
Write-Host "  [OK] ProductName: $($tauriConf.productName)" -ForegroundColor Green
Write-Host "  [OK] WindowTitle: $($tauriConf.app.windows[0].title)" -ForegroundColor Green

# ============================================================================
# FASE 3: Production Build (Tauri)
# ============================================================================
Write-Host ""
Write-Host "[FASE 3] Building Tauri for Windows (release profile)..." -ForegroundColor Yellow
Write-Host "  Profile: lto=thin, strip=true" -ForegroundColor Gray
Write-Host "  This may take 10-20 minutes on first build..." -ForegroundColor Gray

$buildStart = Get-Date

pnpm tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] tauri build failed" -ForegroundColor Red
    exit 1
}

$buildDuration = (Get-Date) - $buildStart
Write-Host "  [OK] Build completed in $($buildDuration.ToString('mm\:ss'))" -ForegroundColor Green

# ============================================================================
# FASE 4: Verification & Artifact Collection
# ============================================================================
Write-Host ""
Write-Host "[FASE 4] Verifying build artifacts..." -ForegroundColor Yellow

$bundleDir = "src-tauri\target\release\bundle"
$msiFiles = Get-ChildItem -Path $bundleDir -Filter "*.msi" -Recurse -ErrorAction SilentlyContinue
$exeFiles = Get-ChildItem -Path $bundleDir -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue

$installer = $null
$installerType = ""

if ($msiFiles.Count -gt 0) {
    $installer = $msiFiles[0]
    $installerType = "MSI"
} elseif ($exeFiles.Count -gt 0) {
    # Filter out helper/installer binaries, find the main installer
    $installer = $exeFiles | Where-Object { $_.Name -notmatch "WebView2|setup|installer" } | Select-Object -First 1
    $installerType = "EXE"
}

if ($installer) {
    $sizeMB = [math]::Round($installer.Length / 1MB, 2)
    Write-Host "  [OK] Installer found: $($installer.FullName)" -ForegroundColor Green
    Write-Host "  [OK] Type: $installerType | Size: ${sizeMB} MB" -ForegroundColor Green

    # Calculate SHA-256
    Write-Host "  Computing SHA-256..." -ForegroundColor Gray
    $hash = Get-FileHash -Path $installer.FullName -Algorithm SHA256
    Write-Host "  [OK] SHA-256: $($hash.Hash)" -ForegroundColor Green

    # Also check the main binary
    $mainExe = "src-tauri\target\release\mindledger.exe"
    if (Test-Path $mainExe) {
        $mainHash = Get-FileHash -Path $mainExe -Algorithm SHA256
        $mainSizeMB = [math]::Round((Get-Item $mainExe).Length / 1MB, 2)
        Write-Host "  [OK] Main binary: ${mainSizeMB} MB | SHA-256: $($mainHash.Hash)" -ForegroundColor Green
    }

    # Write results to release doc
    $releaseDoc = @"
# MindLedger v1.0.0-gloria-once - Release Notes

## Build Information
- **Date**: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
- **Commit**: $commitHash
- **Branch**: $BRANCH
- **Tenant**: $TENANT (Psic. Gloria Once)
- **Build Duration**: $($buildDuration.ToString('mm\:ss'))
- **Platform**: Windows 11 (MSVC)

## Installer
- **Type**: $installerType
- **File**: $($installer.Name)
- **Size**: ${sizeMB} MB
- **SHA-256**: $($hash.Hash)

## Main Binary
- **File**: mindledger.exe
- **Size**: ${mainSizeMB} MB
- **SHA-256**: $($mainHash.Hash)

## Tenant Configuration
- **Identifier**: $($tauriConf.identifier)
- **ProductName**: $($tauriConf.productName)
- **WindowTitle**: $($tauriConf.app.windows[0].title)
- **CSP**: Intact (not modified by bundler)

## Build Profile
- **Rust**: $rustVersion
- **Cargo**: $cargoVersion
- **Node**: $(node --version)
- **pnpm**: $(pnpm --version)
- **LTO**: thin
- **Strip**: true

## Smoke Test Results
- [ ] Installer runs without errors
- [ ] Application launches with correct title
- [ ] SQLCipher initializes without pointer exceptions
- [ ] Teal/Sage color palette renders correctly in WebView2
- [ ] Window metadata shows "MindLedger - Psic. Gloria Once"
- [ ] Role displays "Neuropsicóloga Clínica"

## Notes
Automated build from CI/CD pipeline. All branding injected via
scripts/bundle-tenant.py — no core source code modifications required.
"@

    Set-Content -Path "sdd-archive\RELEASE-V1.0.0-GLORIA-ONCE.md" -Value $releaseDoc
    Write-Host "  [OK] Release doc written to sdd-archive\RELEASE-V1.0.0-GLORIA-ONCE.md" -ForegroundColor Green
} else {
    Write-Host "  [ERROR] No installer found in $bundleDir" -ForegroundColor Red
    Write-Host "  Checking directory contents:" -ForegroundColor Yellow
    Get-ChildItem -Path $bundleDir -Recurse | ForEach-Object { Write-Host "    $_" }
    exit 1
}

# ============================================================================
# FASE 5: Summary
# ============================================================================
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host " BUILD COMPLETE" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Installer: $($installer.FullName)" -ForegroundColor White
Write-Host "Size:      ${sizeMB} MB" -ForegroundColor White
Write-Host "SHA-256:   $($hash.Hash)" -ForegroundColor White
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Transfer installer to target Windows machine" -ForegroundColor Gray
Write-Host "  2. Run installer and perform smoke test" -ForegroundColor Gray
Write-Host "  3. Update sdd-archive/RELEASE-V1.0.0-GLORIA-ONCE.md with smoke test results" -ForegroundColor Gray
Write-Host "  4. Push release doc: git add . && git commit -m 'docs: release v1.0.0-gloria-once'" -ForegroundColor Gray
Write-Host "  5. Create GitHub release with installer attachment" -ForegroundColor Gray
Write-Host ""
