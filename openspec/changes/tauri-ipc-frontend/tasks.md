# SDD Task Breakdown: tauri-ipc-frontend

**Change ID**: `tauri-ipc-frontend`
**Project**: MindLedger (Soft Gloria) — Clinical Psychology Practice Management for Ecuador
**Phase**: FASE 2 — Tauri IPC Commands + Frontend SPA
**Status**: Ready for Implementation

---

## Task Organization

Tasks are organized by **capability** (matching the spec) and **dependency order**. Each task includes:
- **Task ID**: Unique identifier (T-XXX)
- **Capability**: Which spec capability it belongs to
- **Dependencies**: Tasks that must complete first
- **Effort**: Small (S) / Medium (M) / Large (L)
- **Priority**: High / Medium / Low

---

## Phase 1: Backend Infrastructure (Rust) — Foundation

### T-001: Migrate Database Pool to rusqlite + bundled-sqlcipher
- **Capability**: `tauri-ipc-commands`
- **Description**: Replace sqlx `DbPool` with `rusqlite::Connection` + `bundled-sqlcipher` feature. Implement connection pooling with `Arc<Mutex<Connection>>` and WAL mode.
- **Files**:
  - `src-tauri/infrastructure/src/database.rs` — Modify `create_pool`, add WAL mode, in-memory support for tests
  - `src-tauri/infrastructure/src/lib.rs` — Re-export new pool type
  - `src-tauri/Cargo.toml` — Ensure `rusqlite` with `bundled-sqlcipher` feature
- **Dependencies**: None
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `create_pool` returns `Arc<Mutex<Connection>>` with WAL mode enabled
  - [ ] In-memory SQLite works for tests (`:memory:`)
  - [ ] `bundled-sqlcipher` compiles on macOS (Apple Silicon) and Windows x64
  - [ ] Existing 74 tests still pass

### T-002: Run Database Migrations for Accounting & Diagnostics Tables
- **Capability**: `tauri-ipc-commands`
- **Description**: Add migration SQL for new tables: `asientos`, `asiento_lineas`, `diagnosticos_cie10`, `diagnosticos_dsm5`, `mapeos_diagnostico`. Seed CIE-10 (~22k codes) and DSM-5 (~300 codes) from FASE 1 ETL.
- **Files**:
  - `src-tauri/infrastructure/src/migrations.rs` — Add new migration functions
  - `src-tauri/infrastructure/src/database.rs` — Call migrations in `run_migrations`
- **Dependencies**: T-001
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] All 5 tables created with correct schema (FK, indexes, constraints)
  - [ ] CIE-10/DSM-5 seed data loads without errors
  - [ ] Migration is idempotent (safe to run multiple times)
  - [ ] Down-migration SQL provided for rollback

### T-003: Implement SqliteAccountingRepository
- **Capability**: `tauri-ipc-commands`
- **Description**: Create repository implementing CRUD for `AsientoContable` and `LineaAsiento`. Include date range queries, cuenta filtering, and financial report generation (BalanceGeneral, EstadoResultados).
- **Files**:
  - `src-tauri/infrastructure/src/repositories.rs` — Add `SqliteAccountingRepository` struct + impl
  - `src-tauri/domain/src/accounting.rs` — Ensure domain types support repository operations
- **Dependencies**: T-001, T-002
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `add_asiento` / `remove_asiento` / `list_asientos` / `get_asiento_by_id` work
  - [ ] `get_libro_diario(date_range)` returns ordered asientos with lineas
  - [ ] `get_balance_general(fecha_corte)` computes Activos/Pasivos/Patrimonio correctly
  - [ ] `get_estado_resultados(fecha_inicio, fecha_fin)` computes Ingresos/Costos/Gastos/UtilidadNeta
  - [ ] `validate_balance(fecha_corte)` returns ValidationResult with `esta_balanceado: bool`
  - [ ] All methods use parameterized queries (no SQL injection)
  - [ ] Unit tests with in-memory SQLite (>90% coverage)

### T-004: Implement SqliteDiagnosticsRepository
- **Capability**: `tauri-ipc-commands`
- **Description**: Create repository for CIE-10/DSM-5 search and MapeoDiagnostico CRUD. Support text search, category filtering, and patient mapping management.
- **Files**:
  - `src-tauri/infrastructure/src/repositories.rs` — Add `SqliteDiagnosticsRepository` struct + impl
  - `src-tauri/domain/src/diagnostics.rs` — Ensure domain types support repository operations
- **Dependencies**: T-001, T-002
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `search_cie10(query, categoria?)` returns paginated results (LIMIT 20)
  - [ ] `search_dsm5(query, categoria?)` returns paginated results (LIMIT 20)
  - [ ] `get_cie10_by_category(categoria)` returns all codes in category
  - [ ] `get_dsm5_by_category(categoria)` returns all codes in category
  - [ ] `create_mapeo(patient_id, diagnostico_id, tipo, notas)` inserts mapping
  - [ ] `get_mapeos(patient_id)` returns patient's diagnostic history
  - [ ] Full-text search uses SQLite FTS5 or LIKE with indexes
  - [ ] Unit tests with in-memory SQLite (>90% coverage)

---

## Phase 2: Backend IPC Commands (Rust)

### T-005: Extend AppError with AccountingError & DiagnosticsError
- **Capability**: `tauri-ipc-commands`
- **Description**: Add new error variants to `AppError` enum for accounting and diagnostics domains. Implement `From` conversions from repository/domain errors.
- **Files**:
  - `src-tauri/commands/src/error.rs` — Add `AccountingError`, `DiagnosticsError` variants
- **Dependencies**: T-001
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `AppError::AccountingError { code, message, details }` variant exists
  - [ ] `AppError::DiagnosticsError { code, message, details }` variant exists
  - [ ] Error codes follow pattern: `ACCOUNTING_*`, `DIAGNOSTICS_*`
  - [ ] Spanish user-facing messages for all variants
  - [ ] `From<RepositoryError>` and `From<DomainError>` implemented
  - [ ] Serializes correctly via `#[serde(tag="type", content="message")]`

### T-006: Implement Accounting Commands (8 commands)
- **Capability**: `tauri-ipc-commands`
- **Description**: Create `accounting_commands.rs` with 8 Tauri v2 commands. Each command validates input, delegates to repository, inserts audit log, returns serialized response.
- **Files**:
  - `src-tauri/commands/src/accounting_commands.rs` — New file with 8 commands
- **Dependencies**: T-003, T-005
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `add_asiento(date, descripcion, detalles)` — validates debit=credit, non-empty detalles
  - [ ] `remove_asiento(id)` — validates not in closed period
  - [ ] `list_asientos(date_range?, cuenta?)` — max 365 day range
  - [ ] `get_libro_diario(date_range)` — required range, returns LibroDiario
  - [ ] `get_balance_general(fecha_corte)` — not future date, returns BalanceGeneral
  - [ ] `get_estado_resultados(fecha_inicio, fecha_fin)` — inicio≤fin, max 365 days
  - [ ] `validate_balance(fecha_corte)` — returns ValidationResult
  - [ ] `filter_asientos_by_cuenta(cuenta, date_range?)` — additional filter command
  - [ ] All commands have integration tests with in-memory SQLite

### T-007: Implement Diagnostics Commands (6 commands)
- **Capability**: `tauri-ipc-commands`
- **Description**: Create `diagnostics_commands.rs` with 6 Tauri v2 commands for CIE-10/DSM-5 search and mapping management.
- **Files**:
  - `src-tauri/commands/src/diagnostics_commands.rs` — New file with 6 commands
- **Dependencies**: T-004, T-005
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `search_cie10(query, categoria?)` — min 2 chars, returns Vec<DiagnosticoCIE10>
  - [ ] `search_dsm5(query, categoria?)` — min 2 chars, returns Vec<DiagnosticoDSM5>
  - [ ] `get_cie10_by_category(categoria)` — valid enum, returns all in category
  - [ ] `get_dsm5_by_category(categoria)` — valid enum, returns all in category
  - [ ] `create_mapeo(patient_id, diagnostico_id, tipo, notas?)` — validates patient & diagnostico exist
  - [ ] `get_mapeos(patient_id)` — returns Vec<MapeoDiagnostico>
  - [ ] All commands have integration tests with in-memory SQLite

### T-008: Implement Age Calculation Command (1 command)
- **Capability**: `tauri-ipc-commands`
- **Description**: Create `age_commands.rs` with formatted age calculation returning Spanish string "45 años, 2 meses y 3 días".
- **Files**:
  - `src-tauri/commands/src/age_commands.rs` — New file with 1 command
- **Dependencies**: T-001, T-005
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `calculate_age_formatted(date_of_birth)` — returns FormattedAge { texto, anos, meses, dias }
  - [ ] Validates DOB not future, not > 150 years ago
  - [ ] Handles edge cases: leap years, month boundaries
  - [ ] Integration tests with various DOB inputs

### T-009: Audit/Refine Existing Patient Commands (7 commands)
- **Capability**: `tauri-ipc-commands`
- **Description**: Review existing 7 patient commands for consistency with new patterns: validation, error handling, audit logging, serialization.
- **Files**:
  - `src-tauri/commands/src/patient_commands.rs` — Modify existing commands
- **Dependencies**: T-005
- **Effort**: Medium
- **Priority**: Medium
- **Acceptance Criteria**:
  - [ ] All 7 commands use `validator` crate for input validation
  - [ ] Error responses use `AppError` variants consistently
  - [ ] Audit log entries inserted for all write operations
  - [ ] Response types implement `Serialize` correctly
  - [ ] Existing tests still pass (100% compatibility)

### T-010: Register All Commands in Main.rs
- **Capability**: `tauri-ipc-commands`
- **Description**: Update `commands/src/lib.rs` to re-export new modules. Update `app/src/main.rs` to register all commands in `invoke_handler::generate!`.
- **Files**:
  - `src-tauri/commands/src/lib.rs` — Add `pub mod accounting_commands`, `diagnostics_commands`, `age_commands`
  - `src-tauri/app/src/main.rs` — Add all command functions to `invoke_handler`
- **Dependencies**: T-006, T-007, T-008, T-009
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] All 22 commands (7 patient + 8 accounting + 6 diagnostics + 1 age) registered
  - [ ] `cargo check` passes with zero warnings
  - [ ] Tauri dev mode starts without IPC registration errors

---

## Phase 3: Type Synchronization & Frontend API Layer

### T-011: Create Accounting TypeScript Types
- **Capability**: `frontend-pages`, `frontend-components`
- **Description**: Define TypeScript interfaces matching Rust `AsientoContable`, `LineaAsiento`, `BalanceGeneral`, `EstadoResultados`, `ValidationResult`, `LibroDiario`.
- **Files**:
  - `src/types/accounting.ts` — New file with all accounting interfaces
- **Dependencies**: T-006 (Rust types finalized)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `LineaAsiento { cuenta, debito: string, credito: string }` — decimals as strings
  - [ ] `AsientoContable { id, fecha, descripcion, lineas }`
  - [ ] `BalanceGeneral { fechaCorte, activos, pasivos, patrimonio, totalActivos, totalPasivosPatrimonio, estaBalanceado }`
  - [ ] `EstadoResultados { fechaInicio, fechaFin, ingresos, costos, gastos, utilidadBruta, utilidadNeta }`
  - [ ] `ValidationResult { estaBalanceado, diferencia, detalles }`
  - [ ] `LibroDiario { asientos, totalDebitos, totalCreditos }`
  - [ ] Zero TypeScript errors (`tsc --noEmit`)

### T-012: Create Diagnostics TypeScript Types
- **Capability**: `frontend-pages`, `frontend-components`
- **Description**: Define TypeScript interfaces for CIE-10, DSM-5, categories, and MapeoDiagnostico.
- **Files**:
  - `src/types/diagnostics.ts` — New file with all diagnostics interfaces
- **Dependencies**: T-007 (Rust types finalized)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `DiagnosticoCIE10 { codigo, descripcion, categoria, subcategoria? }`
  - [ ] `DiagnosticoDSM5 { codigo, descripcion, categoria, criteriosDiagnosticos?, especificadores? }`
  - [ ] `CategoriaCIE10` union type (22 categories)
  - [ ] `CategoriaDSM5` union type (22 categories)
  - [ ] `MapeoDiagnostico { id, patientId, diagnosticoId, tipo, notas?, fechaCreacion }`
  - [ ] `DiagnosisType` union: `PRINCIPAL` | `SECUNDARIO` | `COMORBILIDAD`
  - [ ] Zero TypeScript errors

### T-013: Create Age TypeScript Types
- **Capability**: `frontend-pages`, `frontend-components`
- **Description**: Define TypeScript interfaces for formatted age calculation response.
- **Files**:
  - `src/types/age.ts` — New file with age interfaces
- **Dependencies**: T-008 (Rust types finalized)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `FormattedAge { texto: string, anos: number, meses: number, dias: number }`
  - [ ] `AgeBreakdown extends FormattedAge { totalDays, totalMonths, isMinor, ageOfMajority, formattedShort, formattedLong }`
  - [ ] Zero TypeScript errors

### T-014: Create Enum → Spanish Label Map (Single Source of Truth)
- **Capability**: `frontend-components`, `brand-integration`
- **Description**: Create `ENUM_LABEL_MAP` object mapping all Rust enums to Spanish labels for UI dropdowns and displays.
- **Files**:
  - `src/types/enums.ts` — New file with enum label map
- **Dependencies**: T-009 (Patient enums finalized), T-007 (Diagnostics enums finalized)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `DocumentType`: CI, PASAPORTE, RUC → Spanish labels
  - [ ] `Gender`: MASCULINO, FEMENINO, OTRO → Spanish labels
  - [ ] `AppointmentStatus`: 6 statuses → Spanish labels
  - [ ] `NoteType`: 4 types → Spanish labels
  - [ ] `DiagnosisType`: 3 types → Spanish labels
  - [ ] `CategoriaCIE10`: 22 categories → Spanish labels (with Roman numerals)
  - [ ] `CategoriaDSM5`: 22 categories → Spanish labels
  - [ ] Exported as `ENUM_LABEL_MAP: Record<string, Record<string, string>>`

### T-015: Create Accounting API Layer
- **Capability**: `frontend-pages`
- **Description**: Create `accountingApi.ts` with `invoke` wrappers for all 8 accounting commands.
- **Files**:
  - `src/api/accountingApi.ts` — New file
  - `src/api/index.ts` — Re-export `accountingApi`
- **Dependencies**: T-011
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `addAsiento(data)`, `removeAsiento(id)`, `listAsientos(params?)`
  - [ ] `getLibroDiario(dateRange)`, `getBalanceGeneral(fechaCorte)`
  - [ ] `getEstadoResultados(fechaInicio, fechaFin)`, `validateBalance(fechaCorte)`
  - [ ] `filterAsientosByCuenta(cuenta, dateRange?)`
  - [ ] All functions use `invoke<ReturnType>('command_name', args)`
  - [ ] Proper TypeScript generics for return types

### T-016: Create Diagnostics API Layer
- **Capability**: `frontend-pages`
- **Description**: Create `diagnosticsApi.ts` with `invoke` wrappers for all 6 diagnostics commands.
- **Files**:
  - `src/api/diagnosticsApi.ts` — New file
  - `src/api/index.ts` — Re-export `diagnosticsApi`
- **Dependencies**: T-012
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `searchCIE10(query, categoria?)`, `searchDSM5(query, categoria?)`
  - [ ] `getCIE10ByCategory(categoria)`, `getDSM5ByCategory(categoria)`
  - [ ] `createMapeo(data)`, `getMapeos(patientId)`
  - [ ] All functions use `invoke<ReturnType>('command_name', args)`
  - [ ] Proper TypeScript generics for return types

### T-017: Create Age API Layer
- **Capability**: `frontend-pages`, `frontend-components`
- **Description**: Create `ageApi.ts` with `invoke` wrapper for age calculation command.
- **Files**:
  - `src/api/ageApi.ts` — New file
  - `src/api/index.ts` — Re-export `ageApi`
- **Dependencies**: T-013
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `calculateAge(dateOfBirth)` returns `Promise<FormattedAge>`
  - [ ] Uses `invoke<FormattedAge>('calculate_age_formatted', { dateOfBirth })`

### T-018: Remove Duplicate Types from lib/api.ts
- **Capability**: `frontend-pages`
- **Description**: Delete `src/lib/api.ts` and consolidate all types into `src/types/index.ts` with re-exports from new type files.
- **Files**:
  - `src/lib/api.ts` — **DELETE**
  - `src/types/index.ts` — Modify: re-export from accounting.ts, diagnostics.ts, age.ts, enums.ts
- **Dependencies**: T-011, T-012, T-013, T-014
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `src/lib/api.ts` removed
  - [ ] `src/types/index.ts` exports all types via `export * from './accounting'`, etc.
  - [ ] All imports updated across codebase
  - [ ] `tsc --noEmit` passes with zero errors

---

## Phase 4: Frontend Hooks (TanStack Query v5)

### T-019: Create Accounting Hooks
- **Capability**: `frontend-pages`
- **Description**: Create TanStack Query hooks for accounting: queries (list, reports) and mutations (add, remove).
- **Files**:
  - `src/hooks/useAccounting.ts` — New file
- **Dependencies**: T-015
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `useAsientos(params?)` — `useQuery` with `staleTime: 30_000`
  - [ ] `useLibroDiario(dateRange)` — `useQuery`
  - [ ] `useBalanceGeneral(fechaCorte)` — `useQuery`
  - [ ] `useEstadoResultados(fechaInicio, fechaFin)` — `useQuery`
  - [ ] `useValidateBalance(fechaCorte)` — `useQuery`
  - [ ] `useAddAsiento()` — `useMutation` with `onSuccess: invalidateQueries(['asientos'])`
  - [ ] `useRemoveAsiento()` — `useMutation` with cache invalidation
  - [ ] All hooks handle loading/error states

### T-020: Create Diagnostics Hooks
- **Capability**: `frontend-pages`
- **Description**: Create TanStack Query hooks for diagnostics: search queries, category queries, mapping mutations.
- **Files**:
  - `src/hooks/useDiagnostics.ts` — New file
- **Dependencies**: T-016
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `useSearchCIE10(query, categoria?)` — `useQuery` with `enabled: query.length >= 2`
  - [ ] `useSearchDSM5(query, categoria?)` — `useQuery` with `enabled: query.length >= 2`
  - [ ] `useCIE10ByCategory(categoria)` — `useQuery`
  - [ ] `useDSM5ByCategory(categoria)` — `useQuery`
  - [ ] `useCreateMapeo()` — `useMutation` with `onSuccess: invalidateQueries(['mapeos', patientId])`
  - [ ] `useMapeos(patientId)` — `useQuery`
  - [ ] Debounced search handled at component level (300ms)

### T-021: Create Age Hook
- **Capability**: `frontend-pages`, `frontend-components`
- **Description**: Create `useCalculateAge` hook with debounced DOB input for real-time age display.
- **Files**:
  - `src/hooks/useAge.ts` — New file
- **Dependencies**: T-017
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `useCalculateAge(dateOfBirth)` — returns `{ age: FormattedAge | null, isLoading: boolean }`
  - [ ] Debounced (300ms) via `useDebounce` hook
  - [ ] Handles edge cases: empty DOB, future DOB, >150 years
  - [ ] Used by `AgeDisplay` component

---

## Phase 5: Frontend Shared Components

### T-022: Create MetricCard Component
- **Capability**: `frontend-components`, `brand-integration`
- **Description**: Reusable metric card with Sage Green background, icon, title, value, and optional trend indicator.
- **Files**:
  - `src/components/ui/MetricCard.tsx` — New component
  - `src/components/ui/MetricCard.test.tsx` — Vitest + RTL tests
- **Dependencies**: T-014 (for icon mapping if needed), Brand tokens (T-033)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Props: `title`, `value`, `trend?`, `icon`, `backgroundColor?` (default Sage)
  - [ ] Sage #E5F1EE background by default
  - [ ] Icon left, title+value center, trend right
  - [ ] Accessible: proper heading structure, color contrast
  - [ ] Vitest tests >85% coverage (prop variants, trend display)

### T-023: Create SearchDropdown Component
- **Capability**: `frontend-components`
- **Description**: Native `<select>` styled with Tailwind that translates Rust enums to Spanish labels via `ENUM_LABEL_MAP`.
- **Files**:
  - `src/components/search/SearchDropdown.tsx` — New component
  - `src/components/search/SearchDropdown.test.tsx` — Vitest + RTL tests
- **Dependencies**: T-014
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Props: `options`, `value`, `onChange`, `placeholder`, `label`, `enumKey` (to lookup labels)
  - [ ] Uses `ENUM_LABEL_MAP[enumKey]` to display Spanish labels
  - [ ] Supports all 7 enum types: DocumentType, Gender, AppointmentStatus, NoteType, DiagnosisType, CategoriaCIE10, CategoriaDSM5
  - [ ] Keyboard navigable, focus ring (Primary #0F4C5C)
  - [ ] Vitest tests >85% coverage (all enum mappings)

### T-024: Create DiagnosisAutocomplete Component
- **Capability**: `frontend-components`
- **Description**: Debounced autocomplete input for CIE-10/DSM-5 search with keyboard navigation, highlighting matches, showing code + description.
- **Files**:
  - `src/components/diagnostics/DiagnosisAutocomplete.tsx` — New component
  - `src/components/diagnostics/DiagnosisAutocomplete.test.tsx` — Vitest + RTL tests
- **Dependencies**: T-020 (useSearchCIE10/DSM5 hooks)
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Props: `value`, `onChange`, `onSelect`, `tipo` ('CIE10' | 'DSM5'), `placeholder`, `categoria?`
  - [ ] Debounced search (300ms) via parent hook
  - [ ] Keyboard navigation: arrows, enter, escape
  - [ ] Highlights matching text in results
  - [ ] Shows code + description in dropdown
  - [ ] Sage hover background, Primary focus ring
  - [ ] Vitest tests >85% coverage (debounce, keyboard, selection)

### T-025: Create AgeDisplay Component
- **Capability**: `frontend-components`
- **Description**: Formatted age display component that internally calls `useCalculateAge` on DOB prop change.
- **Files**:
  - `src/components/ui/AgeDisplay.tsx` — New component
  - `src/components/ui/AgeDisplay.test.tsx` — Vitest + RTL tests
- **Dependencies**: T-021
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Props: `dateOfBirth: string`, `className?`, `format?` ('long' | 'short')
  - [ ] Displays "45 años, 2 meses y 3 días" format
  - [ ] Updates automatically on `dateOfBirth` prop change
  - [ ] Monospace font for numbers (JetBrains Mono)
  - [ ] Handles edge cases gracefully
  - [ ] Vitest tests >85% coverage

### T-026: Create FinancialTable Component
- **Capability**: `frontend-components`
- **Description**: Sortable table for accounting data with right-aligned debit/credit columns, negative values in Coral, summary rows.
- **Files**:
  - `src/components/accounting/FinancialTable.tsx` — New component
  - `src/components/accounting/FinancialTable.test.tsx` — Vitest + RTL tests
- **Dependencies**: T-011 (types), T-014 (enum labels for cuenta)
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Props: `data: AsientoContable[]`, `columns`, `sortable`, `onRowClick?`
  - [ ] Sortable by any column (click header)
  - [ ] Debit/Credit columns right-aligned
  - [ ] Negative values (credit > debit) in Coral #E3645F
  - [ ] Summary row with totals
  - [ ] Sage #E5F1EE header background
  - [ ] Vitest tests >85% coverage

### T-027: Create AsientoForm Component
- **Capability**: `frontend-components`
- **Description**: Modal form for creating/editing `AsientoContable` with dynamic detalle rows, real-time debit=credit validation.
- **Files**:
  - `src/components/accounting/AsientoForm.tsx` — New component
  - `src/components/accounting/AsientoForm.test.tsx` — Vitest + RTL tests
- **Dependencies**: T-011 (types), T-019 (useAddAsiento hook)
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Props: `initialData?`, `onSubmit`, `onCancel`, `isOpen`
  - [ ] Fields: fecha, descripcion, dynamic detalle rows (add/remove)
  - [ ] Real-time validation: sum(debito) === sum(credito)
  - [ ] Shows validation error in Coral when unbalanced
  - [ ] Uses `react-hook-form` + `zod` schema
  - [ ] Primary submit button, accessible modal (focus trap, ARIA)
  - [ ] Vitest tests >85% coverage

### T-028: Create AlertCard Component
- **Capability**: `frontend-components`, `brand-integration`
- **Description**: Dismissible alert banner. **Coral #E3645F background ONLY for net loss warnings**. Other severities use Sage/Primary.
- **Files**:
  - `src/components/ui/AlertCard.tsx` — New component
  - `src/components/ui/AlertCard.test.tsx` — Vitest + RTL tests
- **Dependencies**: Brand tokens (T-033), ESLint rule (T-035)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Props: `title`, `message`, `severity` ('warning' | 'error' | 'info'), `onDismiss?`, `isNetLoss?`
  - [ ] Coral background ONLY when `severity='error' && isNetLoss=true`
  - [ ] Sage background for info/warning (non-net-loss)
  - [ ] Dismissible with close button
  - [ ] Accessible: role="alert", proper contrast
  - [ ] Vitest tests >85% coverage (severity variants, Coral restriction)

### T-029: Create TemplateCard Component
- **Capability**: `frontend-components`
- **Description**: Clinical note template preview card with Primary border when selected, Sage hover.
- **Files**:
  - `src/components/clinical/TemplateCard.tsx` — New component
  - `src/components/clinical/TemplateCard.test.tsx` — Vitest + RTL tests
- **Dependencies**: Brand tokens (T-033)
- **Effort**: Small
- **Priority**: Medium
- **Acceptance Criteria**:
  - [ ] Props: `template`, `onSelect`, `selected`
  - [ ] Shows template preview (name, structure)
  - [ ] Primary border when `selected=true`
  - [ ] Sage hover background
  - [ ] Accessible: keyboard selectable, focus ring
  - [ ] Vitest tests >85% coverage

---

## Phase 6: Frontend Pages

### T-030: Update DashboardPage with Real Metric Cards
- **Capability**: `frontend-pages`
- **Description**: Replace hardcoded dashboard with 4 real `MetricCard` components using IPC data: patient count, appointments today, unsigned notes, monthly revenue.
- **Files**:
  - `src/pages/DashboardPage.tsx` — Modify existing
- **Dependencies**: T-019 (useAsientos for revenue), T-022 (MetricCard), T-028 (AlertCard for net loss)
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] 4 MetricCards render with real data (Sage background)
  - [ ] Patient count from `get_patient_count`
  - [ ] Appointments today placeholder (FASE 3) — shows "—" with info tooltip
  - [ ] Unsigned notes placeholder (FASE 3) — shows "—" with info tooltip
  - [ ] Monthly revenue from accounting `get_estado_resultados` (current month)
  - [ ] AlertCard shows Coral net loss warning only when utilidadNeta < 0
  - [ ] Loading skeletons while fetching
  - [ ] Error state with retry button

### T-031: Create AccountingPage
- **Capability**: `frontend-pages`
- **Description**: Full accounting page with Libro Diario table (CRUD), Balance General, Estado Resultados tabs, and PDF export.
- **Files**:
  - `src/pages/AccountingPage.tsx` — New page
- **Dependencies**: T-019 (hooks), T-023 (SearchDropdown for cuenta filter), T-026 (FinancialTable), T-027 (AsientoForm), T-028 (AlertCard)
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] **Libro Diario Tab**: FinancialTable with asientos, date range filter, cuenta filter (SearchDropdown)
  - [ ] **CRUD**: AsientoForm modal for add/edit, delete confirmation
  - [ ] **Balance General Tab**: Renders BalanceGeneral with Activos/Pasivos/Patrimonio sections
  - [ ] **Estado Resultados Tab**: Renders EstadoResultados with Ingresos/Costos/Gastos/UtilidadNeta
  - [ ] **PDF Export**: `@react-pdf/renderer` generates downloadable PDF (dynamic import)
  - [ ] **Validation**: `validate_balance` called on report generation, AlertCard if unbalanced
  - [ ] Responsive at 1400x900, sidebar navigation works

### T-032: Create DiagnosticsPage
- **Capability**: `frontend-pages`
- **Description**: Split-view diagnostics page: search panel (CIE-10/DSM-5 tabs) + mapping creation form + patient mapping history.
- **Files**:
  - `src/pages/DiagnosticsPage.tsx` — New page
- **Dependencies**: T-020 (hooks), T-023 (SearchDropdown for category filter), T-024 (DiagnosisAutocomplete), T-029 (TemplateCard for mapping form?)
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] **Search Panel**: Two tabs (CIE-10, DSM-5) with search input + category filter (SearchDropdown)
  - [ ] **Results**: List with code + description, click to select for mapping
  - [ ] **Mapping Form**: Patient dropdown (SearchDropdown), selected diagnosis, DiagnosisType (SearchDropdown), notas
  - [ ] **History List**: Patient's existing mappings with date, type, diagnosis
  - [ ] Search returns results < 500ms (LIMIT 20, pagination)
  - [ ] Category filters work for both CIE-10 and DSM-5
  - [ ] Mapping creation saves via `create_mapeo` IPC

### T-033: Update ClinicalNotesPage with Predictive Form
- **Capability**: `frontend-pages`
- **Description**: Enhance clinical notes page with predictive form: patient dropdown, real-time age calculation from DOB, diagnosis autocomplete (CIE-10/DSM-5), template selector.
- **Files**:
  - `src/pages/ClinicalNotesPage.tsx` — Modify existing
- **Dependencies**: T-019 (usePatients), T-021 (useCalculateAge), T-024 (DiagnosisAutocomplete), T-025 (AgeDisplay), T-029 (TemplateCard)
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] **Patient Dropdown**: SearchDropdown with patient search (name/cedula)
  - [ ] **Real-time Age**: AgeDisplay updates < 300ms on DOB change
  - [ ] **Diagnosis Autocomplete**: DiagnosisAutocomplete for CIE-10/DSM-5 with category filter
  - [ ] **Template Selector**: TemplateCard grid for note templates
  - [ ] **Note Editor**: Textarea with auto-save draft
  - [ ] All form fields validate on submit
  - [ ] Loading/error states for all IPC calls

### T-034: Update Layout Navigation
- **Capability**: `frontend-pages`, `brand-integration`
- **Description**: Add "Contabilidad" (Calculator icon) and "Diagnósticos" (Search icon) to sidebar navigation. Apply Primary #0F4C5C background to sidebar.
- **Files**:
  - `src/components/layout/Layout.tsx` — Modify nav items array
- **Dependencies**: T-031, T-032 (pages exist), Brand tokens (T-036)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] 7 nav items: Dashboard, Pacientes, Turnos, Historia Clínica, Contabilidad, Diagnósticos, Configuración
  - [ ] Icons: LayoutDashboard, Users, Calendar, FileText, Calculator, Search, Settings
  - [ ] Sidebar background: Primary #0F4C5C
  - [ ] Active item highlighted with Sage background
  - [ ] Responsive collapse on narrow widths

---

## Phase 7: Brand Integration

### T-035: Add CSS Custom Properties for Brand Tokens
- **Capability**: `brand-integration`
- **Description**: Define CSS custom properties in `src/index.css` for all brand colors, typography, spacing.
- **Files**:
  - `src/index.css` — Modify: add `:root` custom properties + base styles
- **Dependencies**: None (can start early)
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `--color-primary: #0F4C5C`
  - [ ] `--color-sage: #E5F1EE`
  - [ ] `--color-coral: #E3645F`
  - [ ] `--color-background: #F8F9FA`
  - [ ] `--color-text: #212529`
  - [ ] `--color-text-muted: #6C757D`
  - [ ] `--color-border: #DEE2E6`
  - [ ] `--color-error: #E3645F` (same as coral — restricted)
  - [ ] `--color-success: #198754`
  - [ ] `--color-warning: #FFC107`
  - [ ] Base styles: reset, focus-visible ring (Primary), typography defaults

### T-036: Update Tailwind Config with Brand Theme
- **Capability**: `brand-integration`
- **Description**: Extend `tailwind.config.js` theme with brand colors mapped to CSS custom properties, font families (Inter, JetBrains Mono), safelist brand colors.
- **Files**:
  - `tailwind.config.js` — Modify: extend theme
- **Dependencies**: T-035
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `colors: { primary: 'var(--color-primary)', sage: 'var(--color-sage)', coral: 'var(--color-coral)', ... }`
  - [ ] `fontFamily: { sans: ['Inter', ...], mono: ['JetBrains Mono', ...] }`
  - [ ] `safelist: ['bg-primary', 'bg-sage', 'bg-coral', 'text-primary', 'text-sage', 'text-coral', ...]`
  - [ ] `npm run build` compiles without purging brand colors

### T-037: Update tauri.conf.json Product Name
- **Capability**: `brand-integration`
- **Description**: Update Tauri configuration with product name "MindLedger" and identifier.
- **Files**:
  - `src-tauri/tauri.conf.json` — Modify: `productName`, `identifier`
- **Dependencies**: None
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `"productName": "MindLedger"`
  - [ ] `"identifier": "com.softgloria.mindledger"`
  - [ ] Window title shows "MindLedger" on macOS/Windows
  - [ ] `cargo tauri build` succeeds

### T-038: Add ESLint Rule for Coral Restriction
- **Capability**: `brand-integration`
- **Description**: Create custom ESLint rule `no-restricted-color-coral` that errors when Coral #E3645F is used outside `AlertCard` with net loss context.
- **Files**:
  - `eslint.config.js` or `.eslintrc.js` — Add rule
  - `src/components/ui/AlertCard.tsx` — Add `/* eslint-disable no-restricted-color-coral */` comment for allowed usage
- **Dependencies**: T-036
- **Effort**: Small
- **Priority**: Medium
- **Acceptance Criteria**:
  - [ ] Rule detects `bg-coral`, `text-coral`, `border-coral`, `bg-[#E3645F]`, etc.
  - [ ] Allows usage in `AlertCard.tsx` only
  - [ ] `npm run lint` passes on clean codebase
  - [ ] Rule documented in contributing guide

### T-039: Bundle Inter Font Locally
- **Capability**: `brand-integration`
- **Description**: Add `@fontsource/inter` and `@fontsource/jetbrains-mono` for offline font loading.
- **Files**:
  - `package.json` — Add dependencies
  - `src/index.css` — Import fontsource CSS
- **Dependencies**: T-035
- **Effort**: Small
- **Priority**: Medium
- **Acceptance Criteria**:
  - [ ] Fonts load without network (offline)
  - [ ] Fallback stack works if font fails
  - [ ] No layout shift on font load

---

## Phase 8: Testing & Verification

### T-040: Rust Integration Tests for All New Commands
- **Capability**: `tauri-ipc-commands`
- **Description**: Write `#[cfg(test)]` integration tests for all 15 new commands (8 accounting + 6 diagnostics + 1 age) using in-memory SQLite.
- **Files**:
  - `src-tauri/commands/src/accounting_commands.rs` — `mod tests`
  - `src-tauri/commands/src/diagnostics_commands.rs` — `mod tests`
  - `src-tauri/commands/src/age_commands.rs` — `mod tests`
- **Dependencies**: T-006, T-007, T-008
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Each command tested: happy path, validation errors, edge cases
  - [ ] >90% coverage on new command code
  - [ ] Tests use in-memory SQLite (`:memory:`)
  - [ ] No external file dependencies
  - [ ] `cargo test` passes with 74 existing + new tests

### T-041: Rust Domain Unit Tests
- **Capability**: `tauri-ipc-commands`
- **Description**: Ensure domain layer has comprehensive unit tests for accounting, diagnostics, age logic.
- **Files**:
  - `src-tauri/domain/src/accounting.rs` — Unit tests
  - `src-tauri/domain/src/diagnostics.rs` — Unit tests
  - `src-tauri/domain/src/age.rs` — Unit tests
- **Dependencies**: T-003, T-004
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `AsientoContable::new` validation tests (debit=credit, non-empty)
  - [ ] `BalanceGeneral`/`EstadoResultados` computation tests
  - [ ] `Age::from_birth_date` edge cases (leap years, boundaries)
  - [ ] CIE-10/DSM-5 mapping logic tests
  - [ ] >95% coverage on domain logic

### T-042: Frontend Component Tests (Vitest + RTL)
- **Capability**: `frontend-components`
- **Description**: Write Vitest + React Testing Library tests for all new shared components.
- **Files**:
  - `src/components/ui/MetricCard.test.tsx`
  - `src/components/search/SearchDropdown.test.tsx`
  - `src/components/diagnostics/DiagnosisAutocomplete.test.tsx`
  - `src/components/ui/AgeDisplay.test.tsx`
  - `src/components/accounting/FinancialTable.test.tsx`
  - `src/components/accounting/AsientoForm.test.tsx`
  - `src/components/ui/AlertCard.test.tsx`
  - `src/components/clinical/TemplateCard.test.tsx`
- **Dependencies**: T-022 through T-029
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] >85% coverage on all new components
  - [ ] Tests cover prop variants, interactions, accessibility
  - [ ] `npm run test` passes
  - [ ] MSW mocks for IPC calls in hooks

### T-043: Frontend Hook Tests (Vitest + MSW)
- **Capability**: `frontend-pages`
- **Description**: Test TanStack Query hooks with MSW mocking `invoke` calls.
- **Files**:
  - `src/hooks/useAccounting.test.ts`
  - `src/hooks/useDiagnostics.test.ts`
  - `src/hooks/useAge.test.ts`
- **Dependencies**: T-019, T-020, T-021
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `useAsientos` cache/invalidation tested
  - [ ] `useSearchCIE10` debounce/pagination tested
  - [ ] `useCalculateAge` debounce/edge cases tested
  - [ ] >80% coverage on hooks

### T-044: E2E IPC Round-trip Tests (Playwright)
- **Capability**: `frontend-pages`, `tauri-ipc-commands`
- **Description**: Playwright tests running against Tauri dev mode, testing full IPC round-trips for all commands.
- **Files**:
  - `src/e2e/accounting.spec.ts`
  - `src/e2e/diagnostics.spec.ts`
  - `src/e2e/clinical-notes.spec.ts`
  - `src/e2e/dashboard.spec.ts`
- **Dependencies**: T-010 (commands registered), T-030 through T-033 (pages implemented)
- **Effort**: Large
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] 100% of commands tested: invoke → validate → persist → return
  - [ ] Tests run in Tauri dev mode (`cargo tauri dev`)
  - [ ] Playwright config for WebKit (macOS) and WebView2 (Windows)
  - [ ] CI runs E2E on both platforms

### T-045: Accessibility Tests (axe-core + Playwright)
- **Capability**: `frontend-pages`, `frontend-components`, `brand-integration`
- **Description**: Automated accessibility testing with axe-core in Playwright, plus manual VoiceOver (macOS) and Narrator (Windows) verification.
- **Files**:
  - `src/e2e/a11y.spec.ts` — axe-core integration
  - Manual test checklist document
- **Dependencies**: T-030 through T-033, T-035 through T-037
- **Effort**: Medium
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] Zero axe-core violations on all pages
  - [ ] Focus rings visible on all interactive elements (Primary #0F4C5C)
  - [ ] ARIA labels present on all form controls
  - [ ] Keyboard navigation works end-to-end
  - [ ] Screen reader announces dynamic content (age, search results)
  - [ ] Contrast ratios meet WCAG 2.1 AA

### T-046: Visual Regression Tests (Storybook - Optional)
- **Capability**: `brand-integration`, `frontend-components`
- **Description**: Storybook stories for key components to verify brand compliance visually.
- **Files**:
  - `src/components/ui/MetricCard.stories.tsx`
  - `src/components/accounting/FinancialTable.stories.tsx`
  - `src/components/ui/AlertCard.stories.tsx`
  - `.storybook/main.ts` — Config
- **Dependencies**: T-022, T-026, T-028, T-035, T-036
- **Effort**: Small
- **Priority**: Low
- **Acceptance Criteria**:
  - [ ] Stories render with all prop variants
  - [ ] Brand colors applied correctly
  - [ ] Coral restriction visible in AlertCard stories

### T-047: Performance Testing
- **Capability**: `frontend-pages`
- **Description**: Profile IPC latency, React re-renders, global state management. Verify no unnecessary re-renders.
- **Files**:
  - Performance test scripts / profiling results
- **Dependencies**: T-030 through T-033
- **Effort**: Medium
- **Priority**: Medium
- **Acceptance Criteria**:
  - [ ] IPC round-trip < 100ms for simple commands
  - [ ] Search autocomplete < 300ms debounce + < 500ms backend
  - [ ] No unnecessary re-renders (React DevTools Profiler)
  - [ ] TanStack Query cache hit rate > 80%
  - [ ] Bundle size < 500KB gzipped (excluding PDF library dynamic import)

### T-048: TypeScript & Rust Quality Gates
- **Capability**: Cross-cutting
- **Description**: Verify zero TypeScript errors, zero Rust clippy warnings, zero cargo audit vulnerabilities.
- **Files**:
  - CI pipeline config (GitHub Actions)
- **Dependencies**: All implementation tasks
- **Effort**: Small
- **Priority**: High
- **Acceptance Criteria**:
  - [ ] `tsc --noEmit` — zero errors
  - [ ] `cargo clippy -- -D warnings` — zero warnings
  - [ ] `cargo audit` — zero vulnerabilities
  - [ ] `npm run lint` — zero errors (including coral rule)
  - [ ] All checks pass in CI

---

## Dependency Graph Summary

```
T-001 → T-002 → T-003 → T-006 → T-010
                ↘ T-004 → T-007 → T-010
T-001 → T-005 → T-006, T-007, T-008, T-009
T-008 → T-010

T-006 → T-011 → T-015 → T-019 → T-026, T-027, T-030, T-031
T-007 → T-012 → T-016 → T-020 → T-024, T-032
T-008 → T-013 → T-017 → T-021 → T-025, T-033
T-009, T-007 → T-014 → T-023, T-034

T-035 → T-036 → T-038, T-039
T-037 (independent)

T-022 → T-030
T-023 → T-031, T-032, T-033, T-034
T-024 → T-032, T-033
T-025 → T-033
T-026 → T-031
T-027 → T-031
T-028 → T-030, T-031
T-029 → T-032, T-033

T-019 → T-027, T-030, T-031
T-020 → T-024, T-032
T-021 → T-025, T-033

T-031, T-032 → T-034

T-006, T-007, T-008 → T-040
T-003, T-004 → T-041
T-022..T-029 → T-042
T-019..T-021 → T-043
T-030..T-033 → T-044
T-030..T-033, T-035..T-037 → T-045
T-022, T-026, T-028, T-035, T-036 → T-046
T-030..T-033 → T-047
All → T-048
```

---

## Effort Summary

| Phase | Tasks | Small | Medium | Large | Total Effort |
|-------|-------|-------|--------|-------|--------------|
| Phase 1: Backend Infrastructure | 4 | 0 | 2 | 2 | 4M+2L |
| Phase 2: Backend Commands | 5 | 2 | 1 | 2 | 2S+1M+2L |
| Phase 3: Types & API Layer | 8 | 7 | 1 | 0 | 7S+1M |
| Phase 4: Frontend Hooks | 3 | 1 | 2 | 0 | 1S+2M |
| Phase 5: Shared Components | 8 | 4 | 4 | 0 | 4S+4M |
| Phase 6: Frontend Pages | 4 | 1 | 1 | 2 | 1S+1M+2L |
| Phase 7: Brand Integration | 5 | 5 | 0 | 0 | 5S |
| Phase 8: Testing | 9 | 1 | 3 | 5 | 1S+3M+5L |
| **Total** | **46** | **21** | **14** | **11** | **21S+14M+11L** |

---

## Priority Breakdown

| Priority | Count | Tasks |
|----------|-------|-------|
| High | 34 | T-001 through T-010, T-011 through T-021, T-022 through T-034, T-035 through T-037, T-040 through T-045, T-048 |
| Medium | 8 | T-009, T-029, T-038, T-039, T-046, T-047, plus 2 from testing |
| Low | 4 | T-046 (optional), plus 3 from testing |

---

## Recommended Implementation Order

### Week 1: Backend Foundation
1. T-001 (Database pool migration)
2. T-002 (Migrations)
3. T-003 (Accounting repo) + T-004 (Diagnostics repo) — parallel
4. T-005 (Error types)

### Week 2: Backend Commands
5. T-006 (Accounting commands)
6. T-007 (Diagnostics commands)
7. T-008 (Age command)
8. T-009 (Patient command audit)
9. T-010 (Command registration)

### Week 3: Types & API Layer
10. T-011 through T-014 (Types + Enum map) — parallel
11. T-015 through T-018 (API layers + cleanup) — parallel

### Week 4: Hooks & Components
12. T-019 through T-021 (Hooks)
13. T-022 through T-029 (Components) — parallel where possible

### Week 5: Pages & Brand
14. T-030 through T-033 (Pages)
15. T-034 (Navigation)
16. T-035 through T-039 (Brand)

### Week 6: Testing & Polish
17. T-040 through T-045 (Core testing)
18. T-046 through T-048 (Optional + quality gates)

---

## Success Criteria Checklist (from Spec)

- [ ] All 7 existing patient IPC commands pass integration tests
- [ ] 8+ new accounting IPC commands implemented + tested
- [ ] 6+ new diagnostics IPC commands implemented + tested
- [ ] 1 age calculation IPC command implemented + tested
- [ ] DashboardPage shows 4 metric cards with real data (Sage Green #E5F1EE)
- [ ] AccountingPage renders LibroDiario CRUD + BalanceGeneral + EstadoResultados + PDF export
- [ ] DiagnosticsPage renders CIE-10/DSM-5 search + category filters + mapping creation
- [ ] ClinicalNotesPage has predictive form: real-time age + diagnosis autocomplete
- [ ] SearchDropdown translates all 7 Rust enums to Spanish labels
- [ ] Sidebar navigation has 7 items (Contabilidad, Diagnósticos added)
- [ ] tauri.conf.json productName = "MindLedger"
- [ ] Brand colors applied correctly (Primary, Sage, Coral restricted, Background, Text)
- [ ] 0 TypeScript errors, 0 Rust clippy warnings
- [ ] All 74 existing tests + new tests pass (>90% coverage new code)
- [ ] Accessibility audit passes (axe-core, VoiceOver, Narrator)

---

## Notes for Implementers

1. **TDD Required**: Every task follows RED-GREEN-REFACTOR. Write tests first.
2. **Type Sync**: Keep Rust ↔ TypeScript types in sync manually (tauri-specta deferred to FASE 3).
3. **Coral Restriction**: Enforce via ESLint rule + PR review. Only `AlertCard` with net loss.
4. **Feature Flags**: Wrap new pages in `VITE_FEATURE_ACCOUNTING` / `VITE_FEATURE_DIAGNOSTICS` for instant rollback.
5. **In-Memory Tests**: All Rust integration tests use `:memory:` SQLite — no external files.
6. **Accessibility**: Test on both WebKit (macOS) and WebView2 (Windows) from Day 1.
7. **Bundle Size**: Dynamic import `@react-pdf/renderer` only on AccountingPage.

---

*Generated from SDD spec/design for `tauri-ipc-frontend` change.*
*Total: 46 tasks across 8 phases.*