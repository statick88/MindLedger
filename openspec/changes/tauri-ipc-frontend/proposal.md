# Proposal: Tauri IPC Commands Layer + Frontend React/TypeScript SPA

## Intent

Build the Tauri IPC commands layer (backend commands) and complete the React/TypeScript frontend SPA for **MindLdger** (brand: Soft Gloria), a clinical psychology practice management desktop app for Ecuador. FASE 1 (Domain Core, Encrypted Persistence, ETL Documental) is complete with 74/74 tests passing. This change delivers FASE 2: Tauri IPC Commands + Frontend SPA.

## Scope

### In Scope
- **Patient IPC Commands**: Audit/refine existing 7 commands (create, get, list, update, delete, search, count)
- **Accounting IPC Commands**: LibroDiario CRUD (add, remove, list, filter), BalanceGeneral, EstadoResultados generation
- **Diagnostics IPC Commands**: CIE-10/DSM-5 search, category filtering, mapping creation, autocomplete
- **Age Calculation IPC Command**: Formatted age strings ("45 años, 2 meses y 3 días")
- **Financial Reports IPC**: Generate BalanceGeneral + EstadoResultados from LibroDiario
- **Frontend - DashboardPage**: Metric cards with Sage Green (#E5F1EE) for patient count, appointments today, unsigned notes, revenue
- **Frontend - AccountingPage**: LibroDiario table (add/edit/delete asientos), BalanceGeneral view, EstadoResultados view
- **Frontend - DiagnosticsPage**: CIE-10/DSM-5 searchable browsers, category filters, MapeoDiagnostico creation UI
- **Frontend - ClinicalNotesPage**: Predictive form with real-time age calculation from DOB, diagnosis autocomplete (CIE-10/DSM-5)
- **Frontend - Search Dropdown**: Translate Rust enums to readable Spanish labels (DocumentType, Gender, AppointmentStatus, NoteType, DiagnosisType)
- **Frontend - Navigation**: Simple sidebar (no nested menus), 5 items: Dashboard, Pacientes, Turnos, Historia Clínica, Contabilidad, Diagnósticos, Configuración
- **Brand Compliance**: Update tauri.conf.json productName to "MindLdger", apply color tokens (Primary #0F4C5C, Sage #E5F1EE, Coral #E3645F, Background #F8F9FA, Text #212529)
- **Infrastructure**: Migrate from sqlx to rusqlite with bundled-sqlcipher for encrypted persistence

### Out of Scope
- Appointment/ClinicalNote IPC commands (deferred to FASE 3)
- Authentication/authorization (deferred to FASE 3)
- Multi-tenancy / clinic branching
- Mobile/responsive breakpoints beyond desktop (1400x900)
- Offline sync / PWA features

## Capabilities

### New Capabilities
- `tauri-ipc-patient`: Patient CRUD + search + count via Tauri commands
- `tauri-ipc-accounting`: LibroDiario management + financial report generation
- `tauri-ipc-diagnostics`: CIE-10/DSM-5 queries + mapping management
- `tauri-ipc-age`: Formatted age calculation from DOB
- `frontend-dashboard`: Dashboard with metric cards (Sage Green background)
- `frontend-accounting`: LibroDiario CRUD + BalanceGeneral + EstadoResultados
- `frontend-diagnostics`: CIE-10/DSM-5 browsers + mapping creation
- `frontend-clinical-notes`: Predictive form with real-time age + diagnosis autocomplete
- `frontend-search-dropdown`: Enum-to-label translation for search filters
- `brand-mindledger`: Visual identity tokens + tauri.conf.json update

### Modified Capabilities
- `tauri-app-setup`: Migrate DB pool from sqlx to rusqlite bundled-sqlcipher; register new IPC commands
- `frontend-types`: Consolidate duplicate types (remove src/lib/api.ts overlap with src/types/index.ts)
- `frontend-navigation`: Add "Contabilidad" and "Diagnósticos" to sidebar

## Approach

### Backend (Rust/Tauri v2)
1. **Infrastructure Migration**: Replace sqlx `DbPool` with `rusqlite::Connection` + `bundled-sqlcipher` feature; implement `PatientRepositorySqlite` using `rusqlite`; add `AccountingRepositorySqlite` and `DiagnosticsRepositorySqlite`
2. **Command Modules**: Create `accounting_commands.rs` and `diagnostics_commands.rs` in `src-tauri/commands/src/`; add `age_commands.rs` for formatted age
3. **Error Handling**: Extend `AppError` with `AccountingError`, `DiagnosticsError` variants; implement `From` for domain errors
4. **Command Registration**: Update `src-tauri/commands/src/lib.rs` to re-export new modules; update `src-tauri/app/src/main.rs` to register all commands

### Frontend (React/TypeScript + Vite + Tailwind)
1. **Type Consolidation**: Remove `src/lib/api.ts` duplicate; single source of truth in `src/types/index.ts`; add Accounting, Diagnostics, Age types
2. **API Layer**: Extend `src/api/index.ts` with `accountingApi`, `diagnosticsApi`, `ageApi` using `invoke()` from `@tauri-apps/api/core`
3. **Pages**: Create `AccountingPage.tsx`, `DiagnosticsPage.tsx`; enhance `DashboardPage.tsx` with real metric cards; enhance `ClinicalNotesPage.tsx` with predictive form
4. **Components**: Build `MetricCard.tsx` (Sage Green #E5F1EE), `AsientoForm.tsx`, `DiagnosisAutocomplete.tsx`, `SearchDropdown.tsx` (enum→label)
5. **Navigation**: Update `Layout.tsx` navigation array with Contabilidad (Calculator icon) and Diagnósticos (Search icon)
6. **Styling**: Add CSS custom properties for brand colors in `index.css`; apply to components via Tailwind config
7. **Accessibility**: ARIA labels, keyboard navigation, focus management; test in WebKit (macOS) and WebView2 (Windows)

### TDD Strategy
- Rust: Write integration tests for each command module (`#[cfg(test)] mod tests`)
- Frontend: Vitest + React Testing Library for components; Playwright for E2E IPC round-trips
- All new code follows RED-GREEN-REFACTOR; 74 existing tests must continue passing

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/commands/src/accounting_commands.rs` | New | LibroDiario CRUD, BalanceGeneral, EstadoResultados commands |
| `src-tauri/commands/src/diagnostics_commands.rs` | New | CIE-10/DSM-5 search, mapping commands |
| `src-tauri/commands/src/age_commands.rs` | New | Formatted age calculation command |
| `src-tauri/commands/src/patient_commands.rs` | Modified | Audit/refine existing 7 commands |
| `src-tauri/commands/src/error.rs` | Modified | Add AccountingError, DiagnosticsError variants |
| `src-tauri/commands/src/lib.rs` | Modified | Re-export new command modules |
| `src-tauri/infrastructure/src/repositories.rs` | New | rusqlite repositories for Accounting, Diagnostics |
| `src-tauri/app/src/main.rs` | Modified | Register new commands; rusqlite pool setup |
| `src-tauri/tauri.conf.json` | Modified | productName → "MindLdger" |
| `src/types/index.ts` | Modified | Add Accounting, Diagnostics, Age types; remove duplicates |
| `src/api/index.ts` | Modified | Add accountingApi, diagnosticsApi, ageApi |
| `src/pages/DashboardPage.tsx` | Modified | Real metric cards with Sage Green background |
| `src/pages/AccountingPage.tsx` | New | LibroDiario + financial reports UI |
| `src/pages/DiagnosticsPage.tsx` | New | CIE-10/DSM-5 browsers + mapping UI |
| `src/pages/ClinicalNotesPage.tsx` | Modified | Predictive form + diagnosis autocomplete |
| `src/components/layout/Layout.tsx` | Modified | Add Contabilidad, Diagnósticos nav items |
| `src/components/ui/MetricCard.tsx` | New | Reusable metric card (Sage Green #E5F1EE) |
| `src/components/accounting/AsientoForm.tsx` | New | AsientoContable create/edit form |
| `src/components/diagnostics/DiagnosisAutocomplete.tsx` | New | CIE-10/DSM-5 search dropdown |
| `src/components/search/SearchDropdown.tsx` | New | Enum-to-Spanish-label translation |
| `src/index.css` / `tailwind.config.js` | Modified | Brand color tokens (Primary, Sage, Coral, Background, Text) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| rusqlite + sqlcipher compilation fails on macOS/Windows | Medium | Pre-test with `cargo build --features bundled-sqlcipher` on both targets; use Tauri's sidecar if needed |
| IPC command signature mismatch (Rust ↔ TypeScript) | High | Use `tauri-specta` or manual type sync; add integration test for each command |
| Duplicate type drift between frontend/backend | High | Single source of truth in `src/types/index.ts`; remove `src/lib/api.ts` |
| Coral accent (#E3645F) misuse on non-alert elements | Low | Design system tokens enforce usage; lint rule for coral usage |
| Accessibility regressions in WebKit/WebView2 | Medium | Test with VoiceOver (macOS) and Narrator (Windows); axe-core in CI |
| Tauri v2 API changes (invoke, State management) | Low | Pin tauri@2.0; follow migration guide; test early |

## Rollback Plan

1. **Git revert**: `git revert <merge-commit>` on main branch (stacked PRs allow single-change revert)
2. **Database**: No schema migrations in this phase (only new tables for accounting/diagnostics if needed); if added, provide down-migration SQL
3. **Frontend**: Feature-flag new pages behind `VITE_FEATURE_ACCOUNTING` / `VITE_FEATURE_DIAGNOSTICS` env vars for instant disable
4. **Tauri commands**: Unregister commands in `main.rs` by commenting `invoke_handler::generate!` entries

## Dependencies

- Tauri v2 (pinned in workspace Cargo.toml)
- `rusqlite` with `bundled-sqlcipher` feature (already in workspace deps)
- `tauri-specta` (optional, for type-safe IPC) or manual type sync
- `lucide-react` (already in package.json for icons)
- `date-fns` or `chrono` (for age formatting on frontend)

## Success Criteria

- [ ] All 7 existing patient IPC commands pass integration tests
- [ ] 8+ new accounting IPC commands implemented + tested (add_asiento, remove_asiento, list_asientos, filter_asientos, get_balance_general, get_estado_resultados, get_libro_diario, validate_balance)
- [ ] 6+ new diagnostics IPC commands implemented + tested (search_cie10, search_dsm5, get_cie10_by_category, get_dsm5_by_category, create_mapping, get_mappings)
- [ ] 1 age calculation IPC command implemented + tested (calculate_age_formatted)
- [ ] DashboardPage shows 4 metric cards with real data (Sage Green #E5F1EE background)
- [ ] AccountingPage renders LibroDiario table with CRUD, BalanceGeneral, EstadoResultados views
- [ ] DiagnosticsPage renders CIE-10/DSM-5 searchable tables with category filters + mapping creation form
- [ ] ClinicalNotesPage has predictive form: real-time age from DOB, diagnosis autocomplete (CIE-10/DSM-5)
- [ ] SearchDropdown translates all Rust enums to Spanish labels (DocumentType, Gender, AppointmentStatus, NoteType, DiagnosisType, CategoriaCIE10, CategoriaDSM5)
- [ ] Sidebar navigation has 7 items (added Contabilidad, Diagnósticos)
- [ ] tauri.conf.json productName = "MindLdger"
- [ ] Brand colors applied: Primary #0F4C5C (sidebar, primary buttons), Sage #E5F1EE (metric cards), Coral #E3645F (cancellation alerts only), Background #F8F9FA, Text #212529
- [ ] 0 TypeScript errors, 0 Rust clippy warnings
- [ ] All 74 existing tests + new tests pass (target: >90% coverage on new code)
- [ ] Accessibility audit passes (axe-core, manual VoiceOver/Narrator test)