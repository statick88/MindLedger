#Requires -Version 5.1
<#
.SYNOPSIS
    MindLedger v1.0.0 — Windows 11 VM Build Script (MSVC Native)
.DESCRIPTION
    Prepares the workspace on a Windows 11 VM using fnm + pnpm,
    then builds the Tauri release binary with SQLCipher 4.5.3.
.NOTES
    Developer: Diego Medardo Saavedra García <Statick>
    Tenant:    Psic. Gloria Once | Neuropsicóloga Clínica
    Date:      2026-07-13
#>

param(
    [string]$Fnmdir = "$env:USERPROFILE\.fnm",
    [string]$NodeVersion = "v24.13.1",
    [string]$WorkspacePath = "$env:USERPROFILE\dev\MindLedger"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host " MindLedger v1.0.0 — Windows Build Pipeline" -ForegroundColor Cyan
Write-Host " Tenant: Psic. Gloria Once" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# ── Step 1: Verify prerequisites ──────────────────────────────────────────────
Write-Host "[1/6] Checking prerequisites..." -ForegroundColor Yellow

# Git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Error "Git is not installed. Install from https://git-scm.com/download/win"
    exit 1
}
Write-Host "  ✓ Git $(git --version)" -ForegroundColor Green

# Rust / MSVC
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Error "Rust is not installed. Run: winget install Rustlang.Rustup"
    exit 1
}
$rustcVersion = rustc --version
Write-Host "  ✓ $rustcVersion" -ForegroundColor Green

# fnm
if (-not (Get-Command fnm -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing fnm..." -ForegroundColor Yellow
    winget install Schniz.fnm
    $env:PATH = "$Fnmdir;$env:PATH"
}
Write-Host "  ✓ fnm $(fnm --version)" -ForegroundColor Green

# ── Step 2: Setup Node.js via fnm ─────────────────────────────────────────────
Write-Host "[2/6] Setting up Node.js $NodeVersion via fnm..." -ForegroundColor Yellow

fnm install $NodeVersion
fnm use $NodeVersion
$nodeVersion = node --version
Write-Host "  ✓ Node.js $nodeVersion" -ForegroundColor Green

# ── Step 3: Install pnpm dependencies ─────────────────────────────────────────
Write-Host "[3/6] Installing pnpm dependencies..." -ForegroundColor Yellow

Set-Location $WorkspacePath

if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    corepack enable
    corepack prepare pnpm@latest --activate
}
pnpm install --frozen-lockfile
Write-Host "  ✓ Dependencies installed" -ForegroundColor Green

# ── Step 4: Verify tenant config ──────────────────────────────────────────────
Write-Host "[4/6] Verifying tenant configuration..." -ForegroundColor Yellow

$tenantConfig = Get-Content "tenant-configs\gloria-once.json" | ConvertFrom-Json
Write-Host "  Tenant ID:      $($tenantConfig.tenant.id)" -ForegroundColor Gray
Write-Host "  Commercial:     $($tenantConfig.tenant.commercialName)" -ForegroundColor Gray
Write-Host "  Keyring Account: $($tenantConfig.crypto.keyringAccount)" -ForegroundColor Gray
Write-Host "  DB Filename:    $($tenantConfig.crypto.dbFileName)" -ForegroundColor Gray

if ($tenantConfig.tenant.id -ne "gloria-once") {
    Write-Error "Tenant config mismatch: expected 'gloria-once', got '$($tenantConfig.tenant.id)'"
    exit 1
}
Write-Host "  ✓ Tenant config verified" -ForegroundColor Green

# ── Step 5: Build release binary ──────────────────────────────────────────────
Write-Host "[5/6] Building Tauri release binary (this may take 5-10 minutes)..." -ForegroundColor Yellow

# Ensure SQLCipher is available for Windows build
# The build.rs will template tauri.conf.json with tenant branding
pnpm tauri build

if ($LASTEXITCODE -ne 0) {
    Write-Error "Tauri build failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}
Write-Host "  ✓ Build completed successfully" -ForegroundColor Green

# ── Step 6: Validate artifacts ────────────────────────────────────────────────
Write-Host "[6/6] Validating build artifacts..." -ForegroundColor Yellow

$bundleDir = "src-tauri\target\release\bundle"
$nsisDir = "$bundleDir\nsis"
$msiDir = "$bundleDir\msi"

$artifacts = @()

# Check NSIS installer
$nsisExe = Get-ChildItem "$nsisDir\*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nsisExe) {
    $hash = (Get-FileHash $nsisExe.FullName -Algorithm SHA256).Hash
    $sizeMB = [math]::Round($nsisExe.Length / 1MB, 2)
    Write-Host "  ✓ NSIS Installer: $($nsisExe.Name) ($sizeMB MB)" -ForegroundColor Green
    Write-Host "    SHA-256: $hash" -ForegroundColor Gray
    $artifacts += @{
        Name = $nsisExe.Name
        Path = $nsisExe.FullName
        SizeMB = $sizeMB
        SHA256 = $hash
        Type = "NSIS"
    }
}

# Check MSI installer
$msiFile = Get-ChildItem "$msiDir\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($msiFile) {
    $hash = (Get-FileHash $msiFile.FullName -Algorithm SHA256).Hash
    $sizeMB = [math]::Round($msiFile.Length / 1MB, 2)
    Write-Host "  ✓ MSI Installer: $($msiFile.Name) ($sizeMB MB)" -ForegroundColor Green
    Write-Host "    SHA-256: $hash" -ForegroundColor Gray
    $artifacts += @{
        Name = $msiFile.Name
        Path = $msiFile.FullName
        SizeMB = $sizeMB
        SHA256 = $hash
        Type = "MSI"
    }
}

# Check binary
$binary = "src-tauri\target\release\mind-ledger.exe"
if (Test-Path $binary) {
    $binaryHash = (Get-FileHash $binary -Algorithm SHA256).Hash
    $binarySizeMB = [math]::Round((Get-Item $binary).Length / 1MB, 2)
    Write-Host "  ✓ Binary: mind-ledger.exe ($binarySizeMB MB)" -ForegroundColor Green
    Write-Host "    SHA-256: $binaryHash" -ForegroundColor Gray
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host " Build Complete — MindLedger v1.0.0" -ForegroundColor Green
Write-Host " Tenant: Psic. Gloria Once" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Cyan

# Export artifact info for manifest generation
$artifacts | ConvertTo-Json -Depth 3 | Set-Content "scripts\windows-build-artifacts.json"
Write-Host "Artifact metadata saved to scripts\windows-build-artifacts.json" -ForegroundColor Gray
