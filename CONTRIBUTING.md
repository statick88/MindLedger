# CONTRIBUTING.md

# Contributing to Soft Gloria (MindLedger)

> **Brand**: MindLedger (white-label: Soft Gloria)  
> **Purpose**: Local-first desktop app for clinical psychology practice management in Ecuador  
> **Stack**: Rust + Tauri v2 + SQLCipher + React/TypeScript  
> **Architecture**: Clean Architecture (Domain → Use Cases → Infrastructure → Presentation)  
> **Methodology**: Spec-Driven Development (SDD) with Engram + OpenSpec hybrid artifact stores

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture Overview](#architecture-overview)
3. [SDD Workflow](#sdd-workflow)
4. [Development Setup](#development-setup)
5. [Coding Conventions](#coding-conventions)
6. [Testing Strategy](#testing-strategy)
7. [Starting a New SDD Change](#starting-a-new-sdd-change)
8. [Git Workflow](#git-workflow)
9. [Key Design Decisions](#key-design-decisions)
10. [Legal & Compliance](#legal--compliance)
11. [Useful Commands](#useful-commands)

---

## Project Overview

**Soft Gloria** (brand: **MindLedger**) is a local-first desktop application for clinical psychology practice management, built specifically for the Ecuadorian market. It handles:

- **Patient Registry** — Demographics, document validation (CI/RUC), clinical history
- **Appointments** — Scheduling, status tracking, doctor/room assignment
- **Clinical Notes** — SOAP notes with diagnosis mapping (CIE-10 / DSM-5)
- **Accounting** — Libro Diario, Asientos Contables, Balance General (Ecuadorian GAAP)
- **Diagnostics Engine** — Auto-mapping CIE-10 ↔ DSM-5 with clinical case detection
- **DOCX Import** — Parse clinical notes from Word documents
- **Encrypted Persistence** — SQLCipher AES-256 + OS Keyring (Windows DPAPI / macOS Keychain / Linux Secret Service)
- **Audit Trail** — Immutable append-only logs for LODP & Ecuador Cybersecurity Law compliance

### Target Environment

| Platform | Status |
|----------|--------|
| macOS (Apple Silicon) | ✅ Primary dev host (M5) |
| Windows 11 | ✅ Cross-test VM |
| Linux | 🔜 Planned |

---

## Architecture Overview

### Clean Architecture Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                        PRESENTATION LAYER                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │
│  │  React/TS   │  │   Tauri     │  │   Commands  │               │
│  │   Frontend  │◄─┤   Commands  │◄─┤   (Rust)    │               │
│  └─────────────┘  └─────────────┘  └──────┬──────┘               │
└────────────────────────────────────────────┼──────────────────────┘
                                             │ invoke / events
                                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                       APPLICATION LAYER                           │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Use Cases / Services  (application/src/)                    │ │
│  │  • docx_parser.rs — ClinicalNoteParser                       │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                             │ depends on (traits)
                                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                        DOMAIN LAYER                               │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Pure Rust — Zero external deps (domain/src/)                │ │
│  │  • patient.rs        — Paciente, DocumentoIdentidad, Value Objects │ │
│  │  • age.rs            — Edad, AgeOfMajority                   │ │
│  │  • value_objects.rs  — Email, Phone, Address, etc.           │ │
│  │  • identifiers.rs    — Ulid, Uuid, typed IDs                 │ │
│  │  • repositories.rs   — Repository TRAITS (port interfaces)   │ │
│  │  • accounting.rs     — LibroDiario, AsientoContable,         │ │
│  │                        BalanceGeneral, ContabilidadError     │ │
│  │  • diagnostics.rs    — CIE10, DSM5, MapeoDiagnostico,        │ │
│  │                        CatalogoCIE10, detectar_caso_clinico  │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                             ▲ implements (traits)
                                             │
┌─────────────────────────────────────────────────────────────────┐
│                     INFRASTRUCTURE LAYER                          │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  Concrete implementations (infrastructure/src/)              │ │
│  │  • database.rs     — SQLCipher pool (rusqlite)               │ │
│  │  • keyring.rs      — SqlCipherKeyManager + OS keyring        │ │
│  │  • repositories.rs — PatientRepositorySqlite, etc.           │ │
│  │  • migrations.rs   — Embedded SQL + runner                   │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Crate Dependency Graph

```
domain (no deps)          ←── Pure domain logic, zero infrastructure
    ▲
    │ implements Repository traits
    │
application               ←── Use cases, depends on domain only
    ▲
    │ uses
    │
commands                  ←── Tauri command handlers, depends on application + domain
    ▲
    │ uses
    │
infrastructure            ←── DB, keyring, repos; depends on domain (traits)
    ▲
    │
app                       ←── Tauri entry point, wires everything together
```

### Frontend Architecture (React/TypeScript)

```
src/
├── api/              # Tauri invoke wrappers, typed via shared-types
├── components/       # Atomic design: atoms → molecules → organisms
├── hooks/            # Custom React hooks (TanStack Query, Zustand)
├── pages/            # Route-level components
├── services/         # Business logic, state management
├── types/            # Shared TypeScript types (synced with Rust via shared-types/)
├── utils/            # Helpers, formatters, validators
└── lib/              # Utilities (clsx, tailwind-merge, etc.)
```

**State Management**: TanStack Query (server state) + Zustand (client state)  
**UI**: Radix UI primitives + Tailwind CSS + class-variance-authority  
**Design Tokens**: See `tailwind.config.js` — Primary `#0F4C5C`, Secondary `#E5F1EE`, Accent `#E3645F`

---

## SDD Workflow

This project follows **Spec-Driven Development (SDD)**. Every non-trivial change goes through a structured lifecycle:

### SDD Phases

| Phase | Agent | Artifact | Engram Key |
|-------|-------|----------|------------|
| **Explore** | `sdd-explore` | Clarification questions, feasibility | `sdd/soft-gloria/explore` |
| **Propose** | `sdd-propose` | Change proposal with intent/scope/approach | `sdd/soft-gloria/proposal` |
| **Spec** | `sdd-spec` | Delta specs: requirements, scenarios, acceptance criteria | `sdd/soft-gloria/spec` |
| **Design** | `sdd-design` | Technical design: architecture, data models, APIs, tasks | `sdd/soft-gloria/design` |
| **Tasks** | `sdd-tasks` | Granular implementation tasks (vertical slices) | `sdd/soft-gloria/tasks` |
| **Apply** | `sdd-apply` | Implementation (TDD: red → green → refactor) | `sdd/soft-gloria/apply-progress` |
| **Verify** | `sdd-verify` | Test execution, coverage, lint, type-check | `sdd/soft-gloria/verify-report` |
| **Archive** | `sdd-archive` | Sync delta specs to canonical specs, close loop | `sdd/soft-gloria/archive-report` |

### How to Start a Change

```bash
# 1. Trigger the orchestrator (gentle-orchestrator skill)
# It will spin up sub-agents for each phase automatically

# Or manually start exploration:
# "I want to add [feature]. Let's explore first."
```

### Engram Memory

All SDD artifacts are persisted in **Engram** (persistent memory across sessions):

- **Topic key**: `sdd/soft-gloria/*`
- **Search**: `mem_search "sdd/soft-gloria" "accounting"`
- **Context**: `mem_context` at session start to recover state

---

## Development Setup

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.80+ | `rustup update stable` |
| Node.js | 20+ | `nvm install 20` |
| pnpm | 9+ | `corepack enable pnpm` |
| Tauri CLI | 2.0+ | `cargo install tauri-cli --version 2.0.0` |
| SQLite3 (dev) | 3.45+ | `brew install sqlite3` / `apt install sqlite3` |
| SQLCipher | 4.5+ | Bundled via `rusqlite` feature `bundled-sqlcipher` |

### Initial Setup

```bash
# 1. Clone & enter
cd /Users/statick/dev/soft-gloria

# 2. Install frontend deps
pnpm install

# 3. Build Rust workspace (downloads SQLCipher)
cd src-tauri
cargo build

# 4. Run tests to verify setup
cargo test --workspace

# 5. Start dev servers (two terminals)
# Terminal 1: Frontend
pnpm dev

# Terminal 2: Tauri
pnpm tauri dev
```

### Windows Cross-Test (VM)

```bash
# On Windows 11 VM:
# 1. Install Rust (msvc toolchain), Node, Visual Studio Build Tools
# 2. Clone repo
# 3. cargo build --target x86_64-pc-windows-msvc
# 4. pnpm tauri build --target x86_64-pc-windows-msvc
```

### IDE Setup (Recommended)

- **VS Code** with extensions: `rust-analyzer`, `tauri`, `tailwindcss`, `eslint`, `prettier`
- **Neovim/LazyVim**: `nvim-frontend` skill has keymaps for React/TS development

---

## Coding Conventions

### Rust (Domain & Infrastructure)

| Aspect | Convention |
|--------|------------|
| **Edition** | 2021 (workspace) |
| **Error Handling** | `thiserror` for domain errors; `anyhow` in infrastructure/commands |
| **Async** | `tokio` (infrastructure only); **domain is sync** |
| **Naming** | `snake_case` (functions, vars), `PascalCase` (types), `SCREAMING_SNAKE_CASE` (consts) |
| **Modules** | One concept per file; re-export in `lib.rs` |
| **Dependencies** | Domain: **zero external deps** (only `std`, `thiserror`, `serde`) |
| **Database** | `rusqlite` with `bundled-sqlcipher` — **NOT sqlx** for runtime |
| **Keyring** | `keyring` crate (OS keyring: DPAPI/Keychain/Secret Service) |
| **Zeroize** | All secrets implement `ZeroizeOnDrop` (CRITICAL-2 fix) |

#### Error Handling Pattern

```rust
// domain/src/accounting.rs
#[derive(Debug, thiserror::Error)]
pub enum ContabilidadError {
    #[error("Asiento desbalanceado: debe={debe} haber={haber}")]
    AsientoDesbalanceado { debe: Decimal, haber: Decimal },
    #[error("Cuenta no encontrada: {codigo}")]
    CuentaNoEncontrada { codigo: String },
    // ...
}

// infrastructure/src/repositories.rs
impl PacienteRepository for PatientRepositorySqlite {
    fn save(&self, paciente: &Paciente) -> Result<(), RepositorioError> {
        // Map domain errors → infrastructure errors
    }
}
```

#### Domain Purity Rules

- **NO** `rusqlite`, `sqlx`, `tokio`, `tauri`, `keyring` in `domain/`
- **NO** `async` functions in domain
- Repository **traits** live in `domain/src/repositories.rs`
- Implementations live in `infrastructure/src/repositories.rs`

### TypeScript / React

| Aspect | Convention |
|--------|------------|
| **Language** | TypeScript strict mode |
| **Components** | Function components + hooks |
| **Styling** | Tailwind + `clsx` + `tailwind-merge` |
| **UI Primitives** | Radix UI (`@radix-ui/*`) |
| **State** | TanStack Query (server) + Zustand (client) |
| **Forms** | React Hook Form + Zod validation |
| **Types** | Shared with Rust via `shared-types/` (codegen planned) |

#### Component Structure

```tsx
// components/atoms/Button.tsx
import { Slot } from '@radix-ui/react-slot';
import { cva, VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';

const buttonVariants = cva('inline-flex items-center justify-center...', {
  variants: {
    variant: { default: 'bg-primary text-primary-foreground', ... },
    size: { default: 'h-10 px-4 py-2', ... },
  },
  defaultVariants: { variant: 'default', size: 'default' },
});

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button';
    return (
      <Comp
        ref={ref}
        className={cn(buttonVariants({ variant, size }), className)}
        {...props}
      />
    );
  }
);
Button.displayName = 'Button';
```

---

## Testing Strategy

### Test Pyramid

```
                    ┌─────────────────┐
                    │   E2E (Playwright)   │  ← Tauri app, critical flows
                    └────────┬────────┘
                     ┌───────┴────────┐
                     │ Integration     │  ← Repository impls, command handlers
                     └───────┬────────┘
              ┌──────────────┴──────────────┐
              │      Unit (cargo test)      │  ← Domain logic, parsers, value objects
              └─────────────────────────────┘
```

### Running Tests

```bash
# All Rust tests (workspace)
cd src-tauri && cargo test --workspace

# Specific crate
cargo test -p domain          # 28 tests (accounting + diagnostics)
cargo test -p application     # 5 tests (docx parser)
cargo test -p infrastructure  # 8 tests (db, keyring, repos)
cargo test -p commands        # Command handler tests

# With output
cargo test --workspace -- --nocapture

# Frontend tests (when added)
pnpm test
pnpm test:e2e
```

### TDD Enforcement

- **Strict TDD mode available**: `strict_tdd` flag in SDD config
- Write failing test → Make it pass → Refactor
- Domain tests **must** exist before implementation
- Coverage target: **≥ 90% on domain crate**

### Test Naming

```rust
// Unit: function_scenario_expected
#[test]
fn asiento_contable_balanced_returns_ok() { ... }
#[test]
fn asiento_contable_unbalanced_returns_err() { ... }

// Integration: module_operation_condition
#[test]
fn patient_repository_save_then_find_by_id_returns_patient() { ... }
```

---

## Starting a New SDD Change

### 1. Trigger Exploration

Tell the orchestrator (or use `sdd-explore` skill directly):

> "I want to add **invoice generation** for patient appointments. Let's explore."

The explore phase will:
- Check existing domain models
- Identify affected crates
- Ask clarifying questions (legal requirements, UI/UX, integration points)
- Produce feasibility assessment

### 2. Review Proposal

The `sdd-propose` phase produces a **Change Proposal** with:
- **Intent**: What problem are we solving?
- **Scope**: Which crates, modules, tables?
- **Approach**: High-level design (new types, traits, migrations)
- **Risks**: Compliance, migration, breaking changes
- **Effort**: T-shirt size (S/M/L/XL)

### 3. Spec Review

The `sdd-spec` phase writes **delta specs** (not full rewrite):
- Requirements (functional + non-functional)
- Scenarios (Given/When/Then)
- Acceptance criteria (testable)
- Open questions

### 4. Design Review

The `sdd-design` phase produces:
- Architecture diagram updates
- Data model changes (SQL + Rust types)
- API contracts (Tauri commands + React hooks)
- Task breakdown (vertical slices)

### 5. Implementation

Tasks are executed via `sdd-apply` with TDD:
1. Write failing domain test
2. Implement domain logic
3. Write failing repo test
4. Implement infrastructure
5. Wire command handler
6. Add frontend hook/component
7. E2E test

---

## Git Workflow

### Branch Naming

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feat/<domain>-<short-desc>` | `feat/accounting-asientos-balance` |
| Bugfix | `fix/<domain>-<short-desc>` | `fix/diagnostics-cie10-mapping` |
| Refactor | `refactor/<domain>-<short-desc>` | `refactor/domain-value-objects` |
| Docs | `docs/<topic>` | `docs/architecture-decisions` |
| Chore | `chore/<topic>` | `chore/update-dependencies` |

### Commit Conventions (Conventional Commits)

```
<type>(<scope>): <subject>

<body>

<footer>
```

| Type | Scope | Example |
|------|-------|---------|
| `feat` | `domain`, `app`, `infra`, `cmd`, `ui` | `feat(domain): add BalanceGeneral computation` |
| `fix` | | `fix(infra): keyring zeroize on drop` |
| `refactor` | | `refactor(domain): extract Edad value object` |
| `test` | | `test(domain): add diagnostics edge cases` |
| `docs` | | `docs: update CONTRIBUTING with SDD flow` |
| `chore` | | `chore: upgrade tauri to 2.1` |

**Examples from current branch** (`feat/accounting-diagnostics-domain`):
```
feat(domain): add LibroDiario, AsientoContable, BalanceGeneral
feat(domain): add CIE10, DSM5, MapeoDiagnostico, CatalogoCIE10
feat(application): add ClinicalNoteParser for DOCX import
feat(infra): add SqlCipherKeyManager with OS keyring integration
feat(infra): add SQLCipher pool with PRAGMA key
fix(domain): auto-map diagnostics in detectar_caso_clinico (CRITICAL-1)
fix(infra): zeroize key material on drop (CRITICAL-2)
fix(domain): add missing ContabilidadError variants (WARNING-1)
test(domain): 28 tests for accounting + diagnostics
test(infra): 8 tests for database + keyring
test(application): 5 tests for docx parser
```

### PR Process

1. **Small, focused PRs** (< 400 lines changed)
2. **Chained PRs** for large features (use `chained-pr` skill)
3. **Requirements**: All tests pass, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`
4. **Review**: At least one approval (Gentleman Guardian Angel auto-review runs on push)
5. **Merge**: Squash merge with conventional commit message

---

## Key Design Decisions

| Decision | Rationale | Trade-off |
|----------|-----------|-----------|
| **Clean Architecture** | Testability, swap infra, domain purity | More boilerplate (traits, wiring) |
| **rusqlite + bundled-sqlcipher** | Zero-config, static linking, no system dep | No async; sync-only DB calls |
| **Domain = zero deps** | Pure logic, fast tests, no supply chain risk | Manual mapping to/from DTOs |
| **SQLCipher AES-256 + OS Keyring** | LODP compliance, keys never in code/config | Platform-specific keyring APIs |
| **Immutable audit_log** | Ecuador Cybersecurity Law Art. 15 | No UPDATE/DELETE on audit table |
| **CIE-10 + DSM-5 dual coding** | Ecuador MINSAL requires CIE-10; clinicians use DSM-5 | Mapping maintenance burden |
| **DOCX parsing (not PDF)** | Structured XML, easier extraction | Limited to .docx format |
| **Tauri v2 (Rust + WebView)** | Native performance, small bundle, OS integration | WebView2 on Windows, WebKitGTK on Linux |
| **TanStack Query + Zustand** | Separation of server/client state | Two state libs to learn |
| **Radix UI + Tailwind** | Accessible primitives, design token flexibility | More setup than component library |

### Non-Negotiables

1. **Domain crate never depends on infrastructure**
2. **All secrets implement `ZeroizeOnDrop`**
3. **Audit log is append-only (DB triggers enforce)**
4. **Patient document validation follows Ecuadorian rules (CI/RUC)**
5. **Tests must pass before commit (CI enforces)**

---

## Legal & Compliance (Ecuador)

| Regulation | Requirement | Implementation |
|------------|-------------|----------------|
| **LODP** (Ley Orgánica de Protección de Datos Personales) | Encryption at rest, access control, audit trail | SQLCipher AES-256, OS keyring, immutable audit_log |
| **Ley de Ciberseguridad** | Incident logging, data integrity | Append-only audit_log, triggers prevent tampering |
| **MINSAL (Ministerio de Salud)** | CIE-10 coding mandatory | `CatalogoCIE10` with official codes |
| **SRI (Servicio de Rentas Internas)** | Contabilidad GAAP Ecuador | `LibroDiario`, `BalanceGeneral` per SRI norms |
| **Edad de mayoría** | 18 años (Código Civil) | `AgeOfMajority::ECUADOR = 18` in domain |

---

## Useful Commands

### Rust Workspace

```bash
# Build all
cd src-tauri && cargo build --workspace

# Check (fast, no codegen)
cargo check --workspace

# Format
cargo fmt --all --check

# Lint
cargo clippy --workspace -- -D warnings

# Test with coverage (requires cargo-llvm-cov)
cargo llvm-cov --workspace --html

# Generate docs
cargo doc --workspace --no-deps --open

# Update dependencies
cargo update -w
```

### Frontend

```bash
# Dev server
pnpm dev

# Type check
pnpm tsc --noEmit

# Lint
pnpm lint

# Build
pnpm build

# Preview build
pnpm preview
```

### Tauri

```bash
# Dev (frontend + backend with hot reload)
pnpm tauri dev

# Build production
pnpm tauri build

# Build for specific target
pnpm tauri build --target x86_64-pc-windows-msvc
pnpm tauri build --target aarch64-apple-darwin
```

### Database / Migrations

```bash
# Migrations are embedded in infrastructure crate
# To add a migration:
# 1. Edit src-tauri/infrastructure/migrations.sql
# 2. Bump version in migrations.rs if needed
# 3. Tests will run migrations on in-memory DB
```

### SDD Artifacts (Engram)

```bash
# Search past decisions
mem_search "sdd/soft-gloria" "accounting"

# Get full context
mem_context

# View specific artifact
mem_get_observation <id>
```

---

## Quick Reference: Crate Responsibilities

| Crate | Responsibility | Key Files |
|-------|---------------|-----------|
| `domain` | Pure business logic, value objects, entities, repository traits | `patient.rs`, `accounting.rs`, `diagnostics.rs`, `value_objects.rs`, `repositories.rs` |
| `application` | Use cases, orchestrators, document parsing | `docx_parser.rs` |
| `infrastructure` | SQLCipher, keyring, repository impls, migrations | `database.rs`, `keyring.rs`, `repositories.rs`, `migrations.sql` |
| `commands` | Tauri command handlers, error mapping | `patient_commands.rs`, `error.rs` |
| `app` | Tauri setup, DI wiring, plugin registration | `src/main.rs` |

---

## Getting Help

- **Architecture questions**: Check `docs/adr/` (Architecture Decision Records) — create one if missing
- **SDD process**: Read `sdd-init` skill or ask orchestrator
- **Rust patterns**: `typescript-best-practices` skill has cross-lang guidance
- **UI patterns**: `impeccable` skill for design review, `shadcn-layouts` for layout issues
- **Testing**: `e2e-testing-patterns` for Playwright, `dart-add-unit-test` patterns apply to Rust too

---

## License

MIT License — see `LICENSE` file.

---

*Generated by SDD Onboard Agent — Soft Gloria v0.1.0 — July 2026*