# MindLedger

Sistema de Gestión Clínica y Contable para Psicólogos Clínicos en Ecuador. Cifrado total, cumplimiento LOPD.

## Quick Start

1. Descargar `MindLedger_1.0.0_x64-setup.exe`
2. Ejecutar el instalador (next → next → install)
3. Abrir MindLedger — la BD se crea automáticamente en la primera ejecución

## What is this?

MindLedger es una aplicación de escritorio que centraliza:

| Módulo | Capacidades |
|--------|-------------|
| **Pacientes** | CRUD completo, cédula/RUC ecuatoriano, contactos de emergencia |
| **Contabilidad** | Plan Contable Ecuatoriano, Balance General, Estado de Resultados |
| **Agenda** | Citas con recordatorios, reagendamiento, KPIs de productividad |
| **Diagnósticos** | Catálogos CIE-10 y DSM-5, mapeo automático |
| **Notas Clínicas** | Registro seguro de sesiones vinculadas a pacientes |

## Architecture

```
Presentation (React/TypeScript)
     ↓
Application (Tauri Commands)
     ↓
Domain (Pure Rust)
     ↓
Infrastructure (SQLCipher + Keyring)
```

- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust + Tauri v2
- **Database**: SQLCipher (AES-256 encrypted SQLite)
- **State**: Zustand (global) + React Query (server)

## Security

- SQLCipher AES-256 — base de datos cifrada en disco
- Keyring nativo — claves en Windows Credential Manager / macOS Keychain
- Zeroize — claves limpiadas de memoria automáticamente
- CSP estricto — sin `unsafe-inline` ni `unsafe-eval`

## Building from Source

```powershell
# Prerequisites
# - Rust 1.80+ (rustup)
# - Node.js 20+ with pnpm
# - WebView2 Runtime (Windows)

# Install dependencies
pnpm install

# Build frontend
pnpm build

# Build Tauri app (NSIS installer)
cd src-tauri
pnpm tauri build --bundles nsis
```

See [WINDOWS_BUILD.md](WINDOWS_BUILD.md) for detailed Windows setup instructions.

## Documentation

- [Architecture Manual](ARCHITECTURE.md) — Clean Architecture layers, data flow, workspace structure
- [Windows Build Guide](WINDOWS_BUILD.md) — Prerequisites, toolchain setup, build commands
- [White-Label Guide](WHITE_LABEL_GUIDE.md) — Multi-tenant deployment, branding injection
- [Contributing](CONTRIBUTING.md) — SDD workflow, coding conventions, git practices

## License

Proprietary — Diego Medardo Saavedra García (Statick)
