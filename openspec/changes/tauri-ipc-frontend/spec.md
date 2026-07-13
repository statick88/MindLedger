# Spec: Tauri IPC Commands + Frontend SPA (tauri-ipc-frontend)

## Change ID
`tauri-ipc-frontend`

## Overview
This spec defines the Tauri IPC commands layer and React/TypeScript frontend SPA for **MindLedger**, a clinical psychology practice management desktop application for Ecuador. FASE 1 (Domain Core, Encrypted Persistence, ETL Documental) is complete with 74/74 tests passing. This change delivers FASE 2.

---

## Capability 1: tauri-ipc-commands

### Capability ID
`tauri-ipc-commands`

### Capability Name
Tauri IPC Commands Layer

### Description
Backend Rust functions exposed as Tauri v2 commands (`#[tauri::command]`) that handle validation, domain logic, and encrypted SQLite persistence via `rusqlite` + `bundled-sqlcipher`. Commands are organized into four domains: Patient, Accounting, Diagnostics, and Age Calculation.

### Invariants Preserved
- **I-01**: Existing data never deleted — all commands are additive or update-only
- **I-02**: Each command is an atomic operation — single transaction per command
- **I-03**: Public interfaces maintain compatibility — command signatures are stable
- **I-04**: No code duplication in infrastructure — shared repository patterns
- **I-05**: No duplicate persistence — single repository per domain
- **I-06**: TDD without exception — every command has integration tests
- **I-07**: Immutable audit in database — all writes include audit trail
- **I-08**: Concurrent access without deadlocks — connection pooling with WAL mode
- **I-09**: Read endpoints use existing repositories — no new query paths for reads
- **I-10**: Tests don't depend on external files — in-memory SQLite for tests
- **I-11**: Domain self-contained and portable — no Tauri types in domain layer

### Input
| Command | Parameters | Validation Rules |
|---------|------------|------------------|
| **Patient Commands** | | |
| `create_patient` | `name: String, cedula: String, date_of_birth: NaiveDate, email: Option<String>, phone: Option<String>, notes: Option<String>` | Cedula unique (10 digits Ecuador), email format, DOB not future |
| `get_patient` | `id: Uuid` | UUID v4 format |
| `list_patients` | `limit: Option<u32>, offset: Option<u32>` | Limit ≤ 100, offset ≥ 0 |
| `update_patient` | `id: Uuid, fields: PatientUpdateFields` | At least one field, cedula unique if changed |
| `delete_patient` | `id: Uuid` | Soft delete only (audit trail) |
| `search_patients` | `query: String` | Min 2 chars, max 100 |
| `get_patient_count` | *(none)* | — |
| **Accounting Commands** | | |
| `add_asiento` | `date: NaiveDate, descripcion: String, detalles: Vec<AsientoDetalle>` | Detalles non-empty, debit = credit, valid accounts |
| `remove_asiento` | `id: Uuid` | Only if not in closed period |
| `list_asientos` | `date_range: Option<DateRange>, cuenta: Option<String>` | Date range max 365 days |
| `get_libro_diario` | `date_range: DateRange` | Required range, max 365 days |
| `get_balance_general` | `fecha_corte: NaiveDate` | Fecha corte not future |
| `get_estado_resultados` | `fecha_inicio: NaiveDate, fecha_fin: NaiveDate` | Inicio ≤ fin, max 365 days |
| `validate_balance` | `fecha_corte: NaiveDate` | Returns ValidationResult |
| **Diagnostics Commands** | | |
| `search_cie10` | `query: String, categoria: Option<CategoriaCIE10>` | Query min 2 chars |
| `search_dsm5` | `query: String, categoria: Option<CategoriaDSM5>` | Query min 2 chars |
| `get_cie10_by_category` | `categoria: CategoriaCIE10` | Valid category enum |
| `get_dsm5_by_category` | `categoria: CategoriaDSM5` | Valid category enum |
| `create_mapeo` | `patient_id: Uuid, diagnostico_id: String, tipo: DiagnosisType, notas: Option<String>` | Patient exists, diagnostico exists in CIE-10/DSM-5 |
| `get_mapeos` | `patient_id: Uuid` | Patient exists |
| **Age Command** | | |
| `calculate_age_formatted` | `date_of_birth: NaiveDate` | DOB not future, not > 150 years ago |

### Processing
1. **Command Invocation**: Frontend calls `invoke("command_name", args)` via `@tauri-apps/api/core`
2. **Validation**: Input validated via `validator` crate; returns `AppError::Validation` on failure
3. **State Access**: `State<'_, AppState>` provides `DbPool` (rusqlite connection pool with WAL)
4. **Domain Logic**: Delegates to domain layer (`domain/src/*.rs`) — pure functions, no side effects
5. **Persistence**: Repository trait implementations (`infrastructure/src/repositories.rs`) execute parameterized SQL
6. **Audit**: Every write inserts into `audit_log` table (immutable, append-only)
7. **Serialization**: Return types implement `Serialize` — JSON response to frontend

### Output
| Command | Success Response | Error Response |
|---------|------------------|----------------|
| Patient CRUD | `Patient` / `Vec<Patient>` / `u64` / `bool` | `AppError { code, message, details }` |
| Accounting | `AsientoContable` / `Vec<AsientoContable>` / `BalanceGeneral` / `EstadoResultados` / `ValidationResult` | `AppError::AccountingError` |
| Diagnostics | `Vec<CIE10>` / `Vec<DSM5>` / `Vec<MapeoDiagnostico>` / `MapeoDiagnostico` | `AppError::DiagnosticsError` |
| Age | `FormattedAge { texto: String, anos: u8, meses: u8, dias: u8 }` | `AppError::Validation` |

### Acceptance Criteria
- [ ] All 7 existing patient commands pass integration tests (100% coverage on new logic)
- [ ] 8 accounting commands implemented + tested (add_asiento, remove_asiento, list_asientos, get_libro_diario, get_balance_general, get_estado_resultados, validate_balance, filter_asientos_by_cuenta)
- [ ] 6 diagnostics commands implemented + tested (search_cie10, search_dsm5, get_cie10_by_category, get_dsm5_by_category, create_mapeo, get_mapeos)
- [ ] 1 age command implemented + tested (calculate_age_formatted)
- [ ] All commands return proper `AppError` variants with user-facing Spanish messages
- [ ] rusqlite + bundled-sqlcipher compiles on macOS (Apple Silicon) and Windows x64
- [ ] Zero clippy warnings, zero `cargo audit` vulnerabilities
- [ ] Integration tests use in-memory SQLite; no external file dependencies (I-10)
- [ ] Commands registered in `invoke_handler::generate!` in `main.rs`
- [ ] `tauri-specta` or manual TypeScript type sync verified for all commands

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| rusqlite + sqlcipher compilation fails on macOS/Windows | Medium | High | Pre-test with `cargo build --features bundled-sqlcipher` on both targets; CI matrix |
| IPC command signature mismatch (Rust ↔ TypeScript) | High | High | Use `tauri-specta` for auto-generated types; add integration test per command |
| Duplicate type drift between frontend/backend | High | Medium | Single source of truth in `src/types/index.ts`; remove `src/lib/api.ts` |
| Audit log growth unbounded | Low | Medium | Implement retention policy in FASE 3; monitor in staging |
| Concurrent write contention on SQLite | Low | Medium | WAL mode + connection pool (max 10); short transactions |

---

## Capability 2: frontend-pages

### Capability ID
`frontend-pages`

### Capability Name
Frontend SPA Pages

### Description
React/TypeScript pages for the MindLedger SPA, built with Vite + Tailwind CSS. Four main pages replace hardcoded stubs with real data from IPC commands, plus navigation updates.

### Invariants Preserved
- **I-01**: Existing patient data preserved — no migration, only reads
- **I-02**: Atomic UI updates — React state + TanStack Query mutations
- **I-03**: API compatibility — `src/api/index.ts` is single IPC boundary
- **I-04**: No component duplication — shared components in `src/components/`
- **I-06**: TDD — Vitest + React Testing Library for components; Playwright for E2E
- **I-09**: Read endpoints use existing repositories — frontend calls existing IPC reads
- **I-11**: Types portable — `src/types/index.ts` has no React dependencies

### Input
| Page | User Interactions | Data Sources (IPC) |
|------|-------------------|---------------------|
| **DashboardPage** | View metrics, click navigation | `get_patient_count`, `list_appointments_today` (FASE 3), `get_unsigned_notes_count` (FASE 3), `get_monthly_revenue` (accounting) |
| **AccountingPage** | CRUD asientos, filter by date/cuenta, generate reports, export PDF | `add_asiento`, `remove_asiento`, `list_asientos`, `get_libro_diario`, `get_balance_general`, `get_estado_resultados`, `validate_balance` |
| **DiagnosticsPage** | Search CIE-10/DSM-5, filter by category, create mappings, view history | `search_cie10`, `search_dsm5`, `get_cie10_by_category`, `get_dsm5_by_category`, `create_mapeo`, `get_mapeos` |
| **ClinicalNotesPage** | Create note, select patient, auto-calc age, diagnosis autocomplete, template select | `calculate_age_formatted`, `list_patients`, `search_cie10`, `search_dsm5`, `create_clinical_note` (FASE 3) |
| **Layout/Navigation** | Sidebar clicks, responsive collapse | Static config + route state |

### Processing
1. **Routing**: `react-router-dom` v6 in `src/App.tsx` — routes: `/`, `/pacientes`, `/turnos`, `/historia-clinica`, `/contabilidad`, `/diagnosticos`, `/configuracion`
2. **State Management**: TanStack Query v5 for server state (IPC calls); React `useState`/`useReducer` for local UI state
3. **Data Fetching**: Custom hooks in `src/hooks/` (e.g., `usePatients`, `useAccounting`, `useDiagnostics`) wrapping `src/api/index.ts`
4. **Forms**: `react-hook-form` + `zod` validation; Spanish error messages
5. **Real-time Age**: `useEffect` on DOB change → debounced (300ms) `ageApi.calculateAge()` call
6. **Autocomplete**: `DiagnosisAutocomplete` component → debounced search → `diagnosticsApi.searchCie10/Dsm5`
7. **PDF Export**: `@react-pdf/renderer` client-side generation from report data

### Output
| Page | Rendered Elements | Brand Compliance |
|------|-------------------|------------------|
| **DashboardPage** | 4 `MetricCard` components (Sage #E5F1EE bg), `AlertCard` (Coral #E3645F only for net loss) | ✅ Primary sidebar, Sage metrics, Coral alerts only |
| **AccountingPage** | `AsientoForm` (modal), `FinancialTable` (sortable), report tabs (Libro Diario / Balance / Resultados), export button | ✅ Primary buttons, Sage table headers, Coral for negative balances |
| **DiagnosticsPage** | Split view: search panel (CIE-10/DSM-5 tabs) + mapping form + history list | ✅ Primary search buttons, Sage category chips, Coral for invalid mappings |
| **ClinicalNotesPage** | Patient dropdown, DOB → AgeDisplay, DiagnosisAutocomplete, template selector, note editor | ✅ Primary save button, Sage template cards, Coral for required field alerts |
| **Layout** | Sidebar (Primary #0F4C5C bg), 7 nav items with `lucide-react` icons | ✅ Full brand compliance |

### Acceptance Criteria
- [ ] **DashboardPage**: 4 metric cards render with real data (Sage #E5F1EE background); Coral alert only shows when net loss
- [ ] **AccountingPage**: Libro Diario table loads with pagination; CRUD modal works; Balance General + Estado Resultados tabs render correctly; PDF export downloads valid file
- [ ] **DiagnosticsPage**: CIE-10/DSM-5 search returns results < 500ms; category filters work; mapping creation saves via IPC; history list shows patient mappings
- [ ] **ClinicalNotesPage**: DOB change → real-time age display (< 300ms); diagnosis autocomplete shows CIE-10/DSM-5 results; patient search dropdown works
- [ ] **Navigation**: Sidebar has 7 items (Dashboard, Pacientes, Turnos, Historia Clínica, Contabilidad, Diagnósticos, Configuración) with correct icons
- [ ] **Responsive**: Works at 1400x900 (desktop target); sidebar collapses on narrow widths
- [ ] **TypeScript**: Zero errors (`tsc --noEmit`); strict mode enabled
- [ ] **Tests**: Vitest component tests > 80% coverage; Playwright E2E for IPC round-trips

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| IPC command signature mismatch | High | High | `tauri-specta` auto-generated types; CI type-check job |
| TanStack Query cache staleness | Medium | Medium | `invalidateQueries` on mutations; `staleTime: 30_000` |
| PDF generation bundle size | Low | Medium | Dynamic import `@react-pdf/renderer` only on AccountingPage |
| Accessibility regressions | Medium | High | axe-core in CI; manual VoiceOver (macOS) + Narrator (Windows) test |
| WebKit/WebView2 rendering differences | Medium | Medium | Test on both; CSS custom properties for consistent theming |

---

## Capability 3: frontend-components

### Capability ID
`frontend-components`

### Capability Name
Shared UI Components

### Description
Reusable, accessible React components that enforce MindLedger brand tokens and translate Rust domain enums to Spanish labels. Components are pure presentational (no IPC logic) and live in `src/components/`.

### Invariants Preserved
- **I-03**: Public interfaces stable — component props are TypeScript interfaces
- **I-04**: No duplication — single component per UI pattern
- **I-06**: TDD — Vitest + RTL for every component
- **I-11**: Portable — no Tauri/IPC imports in components

### Input
| Component | Props | Data Source |
|-----------|-------|-------------|
| **MetricCard** | `title: string, value: string | number, trend?: { value: number, label: string }, icon: LucideIcon, backgroundColor?: string` | Parent passes IPC data |
| **SearchDropdown** | `options: SearchOption[], value: string, onChange: (v: string) => void, placeholder: string, label: string` | `SearchOption` from `src/types/search.ts` (enum → label map) |
| **DiagnosisAutocomplete** | `value: string, onChange: (v: string) => void, onSelect: (d: DiagnosisResult) => void, tipo: 'CIE10' | 'DSM5', placeholder: string` | Calls `diagnosticsApi.searchCie10/Dsm5` via parent hook |
| **AgeDisplay** | `dateOfBirth: string, className?: string` | Calls `ageApi.calculateAge()` internally via `useEffect` |
| **FinancialTable** | `data: AsientoContable[], columns: ColumnDef[], sortable: boolean, onRowClick?: (row) => void` | Parent passes `list_asientos` result |
| **AsientoForm** | `initialData?: AsientoContable, onSubmit: (data: AsientoFormData) => Promise<void>, onCancel: () => void` | Parent manages IPC call |
| **AlertCard** | `title: string, message: string, severity: 'warning' | 'error' | 'info', onDismiss?: () => void` | Only Coral #E3645F for severity='error' + net loss |
| **TemplateCard** | `template: ClinicalNoteTemplate, onSelect: () => void, selected: boolean` | Static template registry |

### Processing
1. **Rendering**: Functional components with `React.memo` where beneficial
2. **Brand Tokens**: CSS custom properties from `index.css` via Tailwind `bg-[var(--color-sage)]` etc.
3. **Enum Translation**: `SearchDropdown` uses `ENUM_LABEL_MAP` from `src/types/enums.ts` — single source of truth
4. **Accessibility**: ARIA labels, `role` attributes, keyboard navigation (combobox` for autocomplete, focus rings (`focus-visible:ring-2 focus-visible:ring-primary`)
5. **Debounce**: `DiagnosisAutocomplete` and `AgeDisplay` use `useDebounce` hook (300ms)
6. **Forms**: `AsientoForm` uses `react-hook-form` + `zod` schema matching Rust validation

### Output
| Component | Rendered Output | Brand Compliance |
|-----------|-----------------|------------------|
| **MetricCard** | Card with icon (left), title + value (center), trend (right), Sage #E5F1EE background | ✅ Sage bg, Primary text, Primary icon |
| **SearchDropdown** | Native `<select>` styled with Tailwind; options show Spanish labels | ✅ Primary border focus, Text #212529 |
| **DiagnosisAutocomplete** | Input + dropdown list; highlights match; keyboard navigable | ✅ Primary focus ring, Sage hover bg |
| **AgeDisplay** | Formatted text: "45 años, 2 meses y 3 días" | ✅ Text #212529, monospace font for numbers |
| **FinancialTable** | Sortable columns, debit/credit aligned right, negative in Coral | ✅ Sage header, Primary text, Coral negatives |
| **AsientoForm** | Modal with date, descripcion, dynamic detalle rows (add/remove), debit=credit validation | ✅ Primary submit, Coral validation errors |
| **AlertCard** | Dismissible banner; Coral bg only for net loss; Sage for info | ✅ Coral restricted to net loss alerts |
| **TemplateCard** | Card with template preview, Primary border when selected | ✅ Primary border, Sage hover |

### Acceptance Criteria
- [ ] **MetricCard**: Renders correctly with all prop combinations; Sage background by default
- [ ] **SearchDropdown**: Translates all 7 Rust enums (DocumentType, Gender, AppointmentStatus, NoteType, DiagnosisType, CategoriaCIE10, CategoriaDSM5) to Spanish labels
- [ ] **DiagnosisAutocomplete**: Debounced search (< 300ms); keyboard navigation (arrows, enter, escape); shows code + description
- [ ] **AgeDisplay**: Updates on DOB prop change; shows formatted string; handles edge cases (future DOB, > 150 years)
- [ ] **FinancialTable**: Sortable by any column; debit/credit columns right-aligned; negative values in Coral #E3645F
- [ ] **AsientoForm**: Validates debit = credit in real-time; dynamic row add/remove; submits valid `AsientoFormData`
- [ ] **AlertCard**: Coral background ONLY when `severity='error'` AND net loss context; otherwise Sage/Primary
- [ ] **All components**: Vitest tests > 85% coverage; RTL tests for interactions; Storybook stories (optional)
- [ ] **Accessibility**: axe-core passes; keyboard navigable; screen reader labels present

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Coral accent misuse on non-alert elements | Low | High | Design token lint rule: `no-coral-outside-alerts`; PR review checklist |
| Enum label drift (Rust ↔ TypeScript) | Medium | Medium | `tauri-specta` generates enum map; build-time check |
| Autocomplete performance with large datasets | Low | Medium | Backend pagination (limit 20); frontend virtualization if needed |
| Form validation mismatch (Zod vs Rust) | Medium | High | Shared schema via `tauri-specta` or manual sync; integration test |

---

## Capability 4: brand-integration

### Capability ID
`brand-integration`

### Capability Name
MindLedger Brand Integration

### Description
Applies MindLedger visual identity across the entire Tauri + React application. Configures Tailwind CSS with brand color tokens, updates `tauri.conf.json` product name, and enforces accessibility standards.

### Invariants Preserved
- **I-03**: Public interfaces stable — brand tokens are CSS custom properties
- **I-04**: No duplication — single token definition in `index.css` + `tailwind.config.js`
- **I-11**: Portable — tokens usable in any CSS context

### Input
| Asset | Source | Format |
|-------|--------|--------|
| **Color Palette** | Brand requirements | HEX + CSS custom properties |
| **Typography** | Brand requirements | Font stack + Tailwind config |
| **Spacing** | 4px grid system | Tailwind spacing scale |
| **Icons** | `lucide-react` | SVG components |
| **tauri.conf.json** | Existing config | JSON patch |

### Processing
1. **CSS Custom Properties**: Define in `src/index.css`:
   ```css
   :root {
     --color-primary: #0F4C5C;      /* Azul Teal Profundo */
     --color-sage: #E5F1EE;         /* Verde Sage Suave */
     --color-coral: #E3645F;        /* Coral/Terracota */
     --color-background: #F8F9FA;   /* Fondo */
     --color-text: #212529;         /* Antracita */
     --color-text-muted: #6C757D;
     --color-border: #DEE2E6;
     --color-error: #E3645F;        /* Same as coral — restricted */
     --color-success: #198754;
     --color-warning: #FFC107;
   }
   ```
2. **Tailwind Config**: Map tokens in `tailwind.config.js`:
   ```js
   theme: {
     extend: {
       colors: {
         primary: 'var(--color-primary)',
         sage: 'var(--color-sage)',
         coral: 'var(--color-coral)',
         background: 'var(--color-background)',
         text: 'var(--color-text)',
         'text-muted': 'var(--color-text-muted)',
         border: 'var(--color-border)',
         error: 'var(--color-error)',
         success: 'var(--color-success)',
         warning: 'var(--color-warning)',
       },
       fontFamily: {
         sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
         mono: ['JetBrains Mono', 'monospace'],
       },
       spacing: { /* 4px grid — default Tailwind spacing works */ },
     }
   }
   ```
3. **Component Application**: Use `bg-primary`, `bg-sage`, `text-coral`, `bg-background`, `text-text` in components
4. **Coral Restriction**: ESLint rule `no-restricted-color-coral` — only allowed in `AlertCard` + net loss contexts
5. **tauri.conf.json**: Update `"productName": "MindLedger"`, `"identifier": "com.softgloria.mindledger"`
6. **Accessibility**: WCAG 2.1 AA — contrast ratios verified (Primary/Text = 7.1:1, Sage/Text = 4.8:1, Coral/Text = 4.5:1)

### Output
| Artifact | Changes | Verification |
|----------|---------|--------------|
| `src/index.css` | CSS custom properties + base styles (reset, focus-visible) | Visual regression test |
| `tailwind.config.js` | Extended theme with brand colors, fonts | `npm run build` compiles |
| `tauri.conf.json` | `productName: "MindLedger"` | Tauri window title shows "MindLedger" |
| Components | All use brand tokens via Tailwind classes | Storybook/Visual test |
| Accessibility | Focus rings (`focus-visible:ring-2 focus-visible:ring-primary`), ARIA labels | axe-core CI + manual test |

### Acceptance Criteria
- [ ] **Colors**: Primary #0F4C5C used for sidebar, primary buttons, focus rings; Sage #E5F1EE for metric cards, table headers; Coral #E3645F ONLY for cancellation alerts / net loss warnings; Background #F8F9FA for page bg; Text #212529 for all body text
- [ ] **Typography**: Inter font loads (Google Fonts or local); fallback stack works offline
- [ ] **tauri.conf.json**: `productName = "MindLedger"`; window title shows "MindLedger" on macOS/Windows
- [ ] **Contrast**: All text/background combos meet WCAG 2.1 AA (4.5:1 normal, 3:1 large)
- [ ] **Focus States**: Visible focus rings on all interactive elements (Primary #0F4C5C)
- [ ] **Coral Restriction**: ESLint rule passes; no Coral usage outside `AlertCard` net loss context
- [ ] **Build**: `npm run build` + `cargo tauri build` succeed with zero warnings
- [ ] **Cross-platform**: Verified on macOS (WebKit) and Windows (WebView2)

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Coral accent misuse | Low | High | ESLint rule + PR checklist; design token documentation |
| Font loading failure offline | Low | Medium | Bundle Inter locally via `@fontsource/inter` |
| Tauri window title not updating | Low | Low | Verify `tauri.conf.json` + rebuild; test on both platforms |
| Contrast regression in dark mode | Low | Medium | No dark mode in FASE 2; defer to FASE 3 |
| Tailwind purge removing unused tokens | Low | Low | `safelist` brand colors in `tailwind.config.js` |

---

## Cross-Cutting Concerns

### Type Synchronization (Rust ↔ TypeScript)
- **Tool**: `tauri-specta` (optional) or manual sync
- **Source of Truth**: Rust command signatures + `src/types/index.ts`
- **CI Check**: `npm run typecheck` + `cargo check` in same pipeline

### Testing Strategy
| Layer | Tool | Coverage Target |
|-------|------|-----------------|
| Rust Commands | `#[cfg(test)]` + `rusqlite` in-memory | 90%+ on new code |
| React Components | Vitest + React Testing Library | 85%+ |
| IPC Round-trips | Playwright (Tauri dev mode) | 100% of commands |
| Accessibility | axe-core + Playwright | Zero violations |
| Visual | Storybook (optional) | Key components |

### Rollback Triggers
- Any IPC command fails integration tests → revert command module
- Frontend TypeScript errors → feature flag page (`VITE_FEATURE_*`)
- Brand contrast failures → revert `tailwind.config.js` + `index.css`
- rusqlite compilation failure → revert to sqlx (FASE 1 state)

---

## File Manifest (Spec → Implementation Mapping)

| Spec Capability | Implementation Files |
|-----------------|---------------------|
| `tauri-ipc-commands` | `src-tauri/commands/src/{patient,accounting,diagnostics,age}_commands.rs`, `src-tauri/commands/src/error.rs`, `src-tauri/commands/src/lib.rs`, `src-tauri/infrastructure/src/repositories.rs`, `src-tauri/app/src/main.rs` |
| `frontend-pages` | `src/pages/{Dashboard,Accounting,Diagnostics,ClinicalNotes}Page.tsx`, `src/App.tsx`, `src/components/layout/Layout.tsx` |
| `frontend-components` | `src/components/ui/MetricCard.tsx`, `src/components/search/SearchDropdown.tsx`, `src/components/diagnostics/DiagnosisAutocomplete.tsx`, `src/components/ui/AgeDisplay.tsx`, `src/components/accounting/FinancialTable.tsx`, `src/components/accounting/AsientoForm.tsx`, `src/components/ui/AlertCard.tsx`, `src/components/clinical/TemplateCard.tsx` |
| `brand-integration` | `src/index.css`, `tailwind.config.js`, `tauri.conf.json`, `src/types/enums.ts` (enum label map) |

---

## Sign-Off Checklist
- [ ] Spec reviewed by backend lead (Rust commands)
- [ ] Spec reviewed by frontend lead (React/TypeScript)
- [ ] Spec reviewed by designer (brand compliance)
- [ ] Spec reviewed by QA (testability)
- [ ] All invariants traceable to implementation files
- [ ] Risk mitigations assigned owners