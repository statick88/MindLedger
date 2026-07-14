# MindLdger — Windows 11 Build Guide

> Complete instructions for building MindLdger on Windows 11 (native or VM).
> Target: `.msi` installer via WiX or `.exe` installer via NSIS.

---

## Prerequisites

### 1. Rust Toolchain

```powershell
# Install rustup (Rust installer for Windows)
winget install Rustlang.Rustup

# Verify installation
rustc --version    # Should show 1.96+ 
cargo --version    # Should show 1.96+

# Add the MSVC target (required for Windows builds)
rustup target add x86_64-pc-windows-msvc
```

### 2. Microsoft C++ Build Tools (MSVC)

MindLdger requires the MSVC linker and C++ runtime for compiling native dependencies (SQLCipher, OpenSSL).

```powershell
# Option A: Install via Visual Studio Installer
winget install Microsoft.VisualStudio.2022.BuildTools

# During installation, select:
#   ✅ "Desktop development with C++"
#   ✅ MSVC v143 build tools (latest)
#   ✅ Windows 11 SDK (10.0.22621.0 or later)
#   ✅ C++ CMake tools for Windows

# Option B: If you already have Visual Studio 2022
# Open Visual Studio Installer → Modify → ensure C++ workload is installed
```

### 3. LLVM (Required for SQLCipher/bundled-sqlcipher)

The `bundled-sqlcipher` feature compiles SQLCipher from source using Clang/LLVM.

```powershell
# Install LLVM
winget install LLVM.LLVM

# Add LLVM to PATH (if not auto-added)
# Default location: C:\Program Files\LLVM\bin
$env:PATH += ";C:\Program Files\LLVM\bin"

# Verify
clang --version
```

### 4. Perl (Required for OpenSSL/SQLCipher build)

SQLCipher's build process requires Perl for OpenSSL configuration.

```powershell
# Install Strawberry Perl (includes all needed modules)
winget install StrawberryPerl.StrawberryPerl

# Verify
perl --version
```

### 5. NASM (Optional, for optimized crypto)

```powershell
winget install NASM.NASM
# Add to PATH: C:\Program Files\NASM
```

### 6. Node.js, fnm & pnpm

```powershell
# Install fnm (Fast Node Manager)
winget install Schniz.fnm

# Install Node.js LTS via fnm
fnm install 24
fnm use 24
node --version   # Should show v24+

# Install pnpm
corepack enable
corepack prepare pnpm@latest --activate
pnpm --version
```

### 7. Tauri CLI

```powershell
pnpm add -g @tauri-apps/cli@2
tauri --version   # Should show 2.x
```

---

## Build Steps

### 1. Clone & Install Dependencies

```powershell
git clone https://github.com/Statick/MindLdger.git
cd MindLdger

# Install frontend dependencies
pnpm install

# Verify frontend builds
pnpm build
```

### 2. Configure Environment Variables (if needed)

```powershell
# Ensure LLVM and Perl are in PATH
$env:PATH += ";C:\Program Files\LLVM\bin;C:\Strawberry\perl\bin"

# For SQLCipher compilation, ensure CL.exe (MSVC) is accessible
# This is typically handled by running from a "Developer Command Prompt":
# Start Menu → "x64 Native Tools Command Prompt for VS 2022"
```

### 3. Build the Application

```powershell
# Development build (faster, larger binary)
cd src-tauri
cargo tauri dev

# Release build (optimized, smaller binary)
cargo tauri build

# The installer will be generated at:
#   src-tauri/target/release/bundle/msi/MindLdger_0.1.0_x64.msi
#   src-tauri/target/release/bundle/nsis/MindLdger_0.1.0_x64-setup.exe
```

### 4. Build with WiX MSI Only

```powershell
cargo tauri build --bundles msi
```

### 5. Build with NSIS EXE Only

```powershell
cargo tauri build --bundles nsis
```

---

## Known Issues & Troubleshooting

### SQLCipher compilation fails on Windows

**Symptom**: `error: linker 'link.exe' not found` or `clang: error: unknown argument`

**Fix**:
```powershell
# Ensure you're in a Developer Command Prompt, NOT PowerShell
# Start Menu → "x64 Native Tools Command Prompt for VS 2022"

# Or set the environment manually:
$env:CC = "clang"
$env:CXX = "clang++"
```

### `bundled-sqlcipher` feature requires CMake

```powershell
winget install Kitware.CMake
cmake --version
```

### Keyring unavailable on Windows

The `keyring` crate uses Windows Credential Manager. If running in a sandboxed environment:
- The app falls back to file-based key storage (same as macOS)
- Key stored at: `%APPDATA%\com.mindledger.desktop\mind-ledger.key`
- **This file contains the database encryption key** — protect it accordingly

### WebView2 not found

Tauri v2 requires WebView2 Runtime on Windows 11. It's pre-installed on Windows 11, but if missing:
```powershell
winget install Microsoft.EdgeWebView2Runtime
```

---

## Architecture Notes

| Component | Windows Build |
|-----------|--------------|
| WebView | WebView2 (Edge Chromium) |
| Database | SQLCipher 4.5.3 (bundled, AES-256-CBC) |
| Key Storage | Windows Credential Manager → file fallback |
| Installer | WiX (.msi) or NSIS (.exe) |
| Target | x86_64-pc-windows-msvc |
| LTO | `lto = "thin"` (**DO NOT use `lto = true`** — corrupts SQLCipher FFI) |

---

## Critical: Release Profile

```toml
[profile.release]
opt-level = 3
lto = "thin"        # ⚠️ MUST be "thin", NOT true — fat LTO corrupts SQLCipher
codegen-units = 1
strip = true
panic = "abort"
```

---

## Bundle Identifier

- Package: `mind-ledger`
- Binary: `MindLdger.exe`
- Bundle ID: `com.mindledger.desktop`
- Display Name: MindLdger

---

*Last updated: 2026-07-13 | MindLdger v0.1.0*
