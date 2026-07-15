# Design: Tauri IPC Commands + Frontend SPA (tauri-ipc-frontend)

## Technical Approach

This design implements **FASE 2** of MindLdger: Tauri v2 IPC commands layer (Rust) + React/TypeScript SPA frontend. FASE 1 (Domain Core, Encrypted Persistence, ETL Documental) is complete with 74/74 tests passing. This change delivers Accounting, Diagnostics, Age Calculation IPC commands and a fully functional desktop SPA with brand-compliant UI.

The architecture follows **Clean Architecture** principles:
- **Domain Layer** (`soft_gloria_domain`): Pure Rust, zero dependencies, contains `accounting.rs`, `diagnostics.rs`, `age.rs`
- **Infrastructure Layer** (`soft_gloria_infrastructure`): `rusqlite` + `bundled-sqlcipher` repositories
- **Commands Layer** (`soft_gloria_commands`): Tauri v2 `#[tauri::command]` handlers, validation, error mapping
- **Frontend** (React/TS + Vite + Tailwind): Pages, components, hooks, API layer with TanStack Query v5

---

## Architecture Decisions

### Decision: IPC Command Layer Structure

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Single `commands.rs` with all commands | Simpler registration, but monolithic | **Separate modules**: `accounting_commands.rs`, `diagnostics_commands.rs`, `age_commands.rs` |
| Commands call domain directly | Couples Tauri to domain | **Commands → Repository → Domain** (Clean Architecture) |
| `sqlx` async pool | Requires external SQLite, no sqlcipher | **`rusqlite` + `bundled-sqlcipher`** with `Arc<Mutex<Connection>>` pool |
| Manual Rust↔TS type sync | Error-prone, drift | **`tauri-specta` for auto-generated TypeScript types** |

### Decision: Frontend State Management

| State Type | Solution | Rationale |
|------------|----------|-----------|
| Server state (IPC data) | **TanStack Query v5** | Caching, deduping, invalidation, optimistic updates |
| Client UI state | **React `useState`/`useReducer`** | Local, no persistence needed |
| Auth/session (FASE 3) | **Zustand** (future) | Minimal boilerplate, SSR-safe |

### Decision: Error Handling Strategy

| Layer | Approach |
|-------|----------|
| **Domain** | Pure `Result<T, DomainError>` — no Tauri types |
| **Infrastructure** | `RepositoryError` variants (`NotFound`, `Constraint`, `Database`, `Serialization`) |
| **Commands** | Map to `AppError` enum (serializable via `#[serde(tag="type", content="message")]`) |
| **Frontend** | `AppError` → toast notifications; `useMutation` `onError` for form validation |

### Decision: Brand Color Tokens

| Token | HEX | Usage | Restriction |
|-------|-----|-------|-------------|
| `--color-primary` | `#0F4C5C` | Sidebar, primary buttons, focus rings | Unrestricted |
| `--color-sage` | `#E5F1EE` | Metric cards, table headers, success backgrounds | Unrestricted |
| `--color-coral` | `#E3645F` | **Only** cancellation alerts / net loss warnings | **ESLint rule: `no-restricted-color-coral`** |
| `--color-background` | `#F8F9FA` | Page background | Unrestricted |
| `--color-text` | `#212529` | All body text | Unrestricted |

---

## Data Flow

### Backend: IPC Command Invocation

```
Frontend (React)
    │ invoke("add_asiento", { date, descripcion, detalles })
    ▼
Tauri IPC Router (v2)
    │
    ▼
#[tauri::command] fn add_asiento(
    State<Arc<DbPool>>,
    date: String,
    descripcion: String,
    detalles: Vec<AsientoDetalle>
) -> AppResult<AsientoContable>
    │
    ├─► 1. Validate input (validator crate / manual)
    ├─► 2. Create domain object: AsientoContable::new(...)
    ├─► 3. Persist via SqliteAccountingRepository
    │       └─► INSERT into asientos + asiento_lineas (single transaction)
    ├─► 4. Audit log insert (immutable)
    └─► 5. Return serialized AsientoContable (JSON)
    │
    ▼
Frontend receives: { id, fecha, descripcion, lineas: [...] }
```

### Frontend: Data Fetching + Mutation

```
Component (e.g., AccountingPage)
    │
    ├─► useQuery({ queryKey: ['asientos'], queryFn: accountingApi.listAsientos })
    │       │
    │       └─► TanStack Query cache → Suspense/loading/error states
    │
    └─► useMutation({ mutationFn: accountingApi.addAsiento, onSuccess: invalidateQueries(['asientos']) })
            │
            └─► Optimistic update → UI immediate → Server confirm → Cache sync
```

---

## File Changes

### New Files (Backend - Rust)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/commands/src/accounting_commands.rs` | Create | 8 accounting IPC commands (add/remove/list asientos, libro diario, balance general, estado resultados, validate balance) |
| `src-tauri/commands/src/diagnostics_commands.rs` | Create | 6 diagnostics IPC commands (CIE-10/DSM-5 search, categories, mappings) |
| `src-tauri/commands/src/age_commands.rs` | Create | 1 age calculation command (formatted Spanish string) |
| `src-tauri/infrastructure/src/repositories.rs` | Modify | Add `SqliteAccountingRepository`, `SqliteDiagnosticsRepository` |
| `src-tauri/infrastructure/src/database.rs` | Modify | Ensure `create_pool` works for tests (in-memory) + migrations for new tables |
| `src-tauri/commands/src/error.rs` | Modify | Add `AccountingError`, `DiagnosticsError` variants to `AppError` |
| `src-tauri/commands/src/lib.rs` | Modify | Re-export new command modules |
| `src-tauri/app/src/main.rs` | Modify | Register all new commands in `invoke_handler` |

### New Files (Frontend - React/TypeScript)

| File | Action | Description |
|------|--------|-------------|
| `src/types/accounting.ts` | Create | TypeScript interfaces for `AsientoContable`, `LineaAsiento`, `BalanceGeneral`, `EstadoResultados` |
| `src/types/diagnostics.ts` | Create | Interfaces for `DiagnosticoCIE10`, `DiagnosticoDSM5`, `CategoriaCIE10`, `CategoriaDSM5`, `MapeoDiagnostico` |
| `src/types/age.ts` | Create | `FormattedAge`, `AgeBreakdown` interfaces |
| `src/types/enums.ts` | Create | `ENUM_LABEL_MAP` for all Rust enums → Spanish labels |
| `src/api/accountingApi.ts` | Create | `accountingApi` with `invoke` wrappers for all accounting commands |
| `src/api/diagnosticsApi.ts` | Create | `diagnosticsApi` for CIE-10/DSM-5/mapping commands |
| `src/api/ageApi.ts` | Create | `ageApi.calculateAge()` wrapper |
| `src/hooks/useAccounting.ts` | Create | TanStack Query hooks: `useAsientos`, `useBalanceGeneral`, `useEstadoResultados`, `useAddAsiento`, etc. |
| `src/hooks/useDiagnostics.ts` | Create | Hooks: `useSearchCIE10`, `useSearchDSM5`, `useCategorias`, `useCreateMapeo`, `useMapeos` |
| `src/hooks/useAge.ts` | Create | `useCalculateAge` with debounced DOB input |
| `src/pages/AccountingPage.tsx` | Create | Libro Diario table + CRUD modal + Balance/Resultados tabs + PDF export |
| `src/pages/DiagnosticsPage.tsx` | Create | Split view: search panel (CIE-10/DSM-5 tabs) + mapping form + history list |
| `src/pages/DashboardPage.tsx` | Modify | Replace hardcoded cards with real `MetricCard` components using IPC data |
| `src/pages/ClinicalNotesPage.tsx` | Modify | Predictive form: patient dropdown, real-time `AgeDisplay`, `DiagnosisAutocomplete` |
| `src/components/ui/MetricCard.tsx` | Create | Reusable metric card (Sage bg, icon, title, value, trend) |
| `src/components/accounting/AsientoForm.tsx` | Create | Modal form with dynamic detalle rows, real-time debit=credit validation |
| `src/components/accounting/FinancialTable.tsx` | Create | Sortable table, right-aligned amounts, Coral for negatives |
| `src/components/diagnostics/DiagnosisAutocomplete.tsx` | Create | Debounced search, keyboard nav, shows code + description |
| `src/components/search/SearchDropdown.tsx` | Create | Native `<select>` styled with Tailwind, enum→Spanish labels |
| `src/components/ui/AgeDisplay.tsx` | Create | Formatted age string from DOB prop, internal `useEffect` → `ageApi` |
| `src/components/ui/AlertCard.tsx` | Create | Dismissible banner, Coral bg ONLY for net loss context |
| `src/components/clinical/TemplateCard.tsx` | Create | Template preview card, Primary border when selected |
| `src/components/layout/Layout.tsx` | Modify | Add "Contabilidad" (Calculator icon) and "Diagnósticos" (Search icon) nav items |
| `src/index.css` | Modify | CSS custom properties for brand tokens + base styles |
| `tailwind.config.js` | Modify | Extend theme with brand colors, fonts (Inter, JetBrains Mono) |
| `tauri.conf.json` | Modify | `productName: "MindLdger"`, `identifier: "com.softgloria.mindledger"` |

### Removed Files

| File | Reason |
|------|--------|
| `src/lib/api.ts` | Duplicate types + API functions — consolidated into `src/types/index.ts` + `src/api/index.ts` |

---

## Interfaces / Contracts

### Rust Command Signatures (Backend → Frontend)

```rust
// accounting_commands.rs
#[tauri::command]
pub async fn add_asiento(
    db: State<'_, Arc<DbPool>>,
    date: String,
    descripcion: String,
    detalles: Vec<AsientoDetalleRequest>,
) -> AppResult<AsientoContable>;

#[tauri::command]
pub async fn remove_asiento(
    db: State<'_, Arc<DbPool>>,
    id: String,
) -> AppResult<bool>;

#[tauri::command]
pub async fn list_asientos(
    db: State<'_, Arc<DbPool>>,
    date_range: Option<DateRange>,
    cuenta: Option<String>,
) -> AppResult<Vec<AsientoContable>>;

#[tauri::command]
pub async fn get_libro_diario(
    db: State<'_, Arc<DbPool>>,
    date_range: DateRange,
) -> AppResult<LibroDiario>;

#[tauri::command]
pub async fn get_balance_general(
    db: State<'_, Arc<DbPool>>,
    fecha_corte: String,
) -> AppResult<BalanceGeneral>;

#[tauri::command]
pub async fn get_estado_resultados(
    db: State<'_, Arc<DbPool>>,
    fecha_inicio: String,
    fecha_fin: String,
) -> AppResult<EstadoResultados>;

#[tauri::command]
pub async fn validate_balance(
    db: State<'_, Arc<DbPool>>,
    fecha_corte: String,
) -> AppResult<ValidationResult>;

// diagnostics_commands.rs
#[tauri::command]
pub async fn search_cie10(
    db: State<'_, Arc<DbPool>>,
    query: String,
    categoria: Option<CategoriaCIE10>,
) -> AppResult<Vec<DiagnosticoCIE10>>;

#[tauri::command]
pub async fn search_dsm5(
    db: State<'_, Arc<DbPool>>,
    query: String,
    categoria: Option<CategoriaDSM5>,
) -> AppResult<Vec<DiagnosticoDSM5>>;

#[tauri::command]
pub async fn get_cie10_by_category(
    db: State<'_, Arc<DbPool>>,
    categoria: CategoriaCIE10,
) -> AppResult<Vec<DiagnosticoCIE10>>;

#[tauri::command]
pub async fn get_dsm5_by_category(
    db: State<'_, Arc<DbPool>>,
    categoria: CategoriaDSM5,
) -> AppResult<Vec<DiagnosticoDSM5>>;

#[tauri::command]
pub async fn create_mapeo(
    db: State<'_, Arc<DbPool>>,
    patient_id: String,
    diagnostico_id: String,
    tipo: DiagnosisType,
    notas: Option<String>,
) -> AppResult<MapeoDiagnostico>;

#[tauri::command]
pub async fn get_mapeos(
    db: State<'_, Arc<DbPool>>,
    patient_id: String,
) -> AppResult<Vec<MapeoDiagnostico>>;

// age_commands.rs
#[tauri::command]
pub async fn calculate_age_formatted(
    _db: State<'_, Arc<DbPool>>, // unused but keeps signature consistent
    date_of_birth: String,
) -> AppResult<FormattedAge>;
```

### TypeScript API Layer (Frontend)

```typescript
// src/api/accountingApi.ts
export const accountingApi = {
  addAsiento: (data: AddAsientoRequest) => 
    invoke<AsientoContable>('add_asiento', data),
  removeAsiento: (id: string) => 
    invoke<boolean>('remove_asiento', { id }),
  listAsientos: (params?: ListAsientosParams) => 
    invoke<AsientoContable[]>('list_asientos', params),
  getLibroDiario: (dateRange: DateRange) => 
    invoke<LibroDiario>('get_libro_diario', { dateRange }),
  getBalanceGeneral: (fechaCorte: string) => 
    invoke<BalanceGeneral>('get_balance_general', { fechaCorte }),
  getEstadoResultados: (fechaInicio: string, fechaFin: string) => 
    invoke<EstadoResultados>('get_estado_resultados', { fechaInicio, fechaFin }),
  validateBalance: (fechaCorte: string) => 
    invoke<ValidationResult>('validate_balance', { fechaCorte }),
};

// src/api/diagnosticsApi.ts
export const diagnosticsApi = {
  searchCIE10: (query: string, categoria?: CategoriaCIE10) => 
    invoke<DiagnosticoCIE10[]>('search_cie10', { query, categoria }),
  searchDSM5: (query: string, categoria?: CategoriaDSM5) => 
    invoke<DiagnosticoDSM5[]>('search_dsm5', { query, categoria }),
  getCIE10ByCategory: (categoria: CategoriaCIE10) => 
    invoke<DiagnosticoCIE10[]>('get_cie10_by_category', { categoria }),
  getDSM5ByCategory: (categoria: CategoriaDSM5) => 
    invoke<DiagnosticoDSM5[]>('get_dsm5_by_category', { categoria }),
  createMapeo: (data: CreateMapeoRequest) => 
    invoke<MapeoDiagnostico>('create_mapeo', data),
  getMapeos: (patientId: string) => 
    invoke<MapeoDiagnostico[]>('get_mapeos', { patientId }),
};

// src/api/ageApi.ts
export const ageApi = {
  calculateAge: (dateOfBirth: string) => 
    invoke<FormattedAge>('calculate_age_formatted', { dateOfBirth }),
};
```

### Shared TypeScript Types (Single Source of Truth)

```typescript
// src/types/accounting.ts
export interface LineaAsiento {
  cuenta: string;
  debito: string; // Decimal as string
  credito: string;
}

export interface AsientoContable {
  id: string;
  fecha: string; // ISO date
  descripcion: string;
  lineas: LineaAsiento[];
}

export interface BalanceGeneral {
  fechaCorte: string;
  activos: Activo[];
  pasivos: Pasivo[];
  patrimonio: PatrimonioItem[];
  totalActivos: string;
  totalPasivosPatrimonio: string;
  estaBalanceado: boolean;
}

export interface EstadoResultados {
  fechaInicio: string;
  fechaFin: string;
  ingresos: CuentaMonto[];
  costos: CuentaMonto[];
  gastos: CuentaMonto[];
  utilidadBruta: string;
  utilidadNeta: string;
}

// src/types/diagnostics.ts
export interface DiagnosticoCIE10 {
  codigo: string;
  descripcion: string;
  categoria: CategoriaCIE10;
  subcategoria?: string;
}

export interface DiagnosticoDSM5 {
  codigo: string;
  descripcion: string;
  categoria: CategoriaDSM5;
  criteriosDiagnosticos?: string[];
  especificadores?: string[];
}

export type CategoriaCIE10 = 
  | 'EnfermedadesInfecciosas' | 'Neoplasias' | 'EnfermedadesSangre'
  | 'EndocrinasNutricionalesMetabolicas' | 'TrastornosMentales' 
  | 'SistemaNervioso' | 'OjoAnexos' | 'OidoMastoides' | 'SistemaCirculatorio'
  | 'SistemaRespiratorio' | 'SistemaDigestivo' | 'PielTejiidoSubcutaneo'
  | 'OsteomuscularConectivo' | 'Genitourinario' | 'EmbarazoPartoPuerperio'
  | 'Perinatal' | 'MalformacionesCongenitas' | 'SintomasSignosHallazgos'
  | 'LesionesEnvenenamiento' | 'CausasExternas' | 'FactoresInfluyenSalud'
  | 'CodigosEspeciales';

export type CategoriaDSM5 = 
  | 'TrastornosNeurodelDesarrollo' | 'EspectroEsquizofreniaYTrastornosPsicoticos'
  | 'TrastornosBipolaresYRelacionados' | 'TrastornosDepresivos'
  | 'TrastornosDeAnsiedad' | 'TrastornosObsesivoCompulsivosYRelacionados'
  | 'TrastornosRelacionadosConTraumaYFactoresDeEstres' | 'TrastornosDisociativos'
  | 'TrastornosSomaticosYRelacionados' | 'TrastornosDeLaIngestaDeAlimentos'
  | 'TrastornosDeEliminacion' | 'TrastornosDelSuenoYVigilia' | 'DisfuncionesSexuales'
  | 'DisforiaDeGenero' | 'TrastornosDisruptivosDelControlDeImpulsosYDeLaConducta'
  | 'TrastornosRelacionadosConSustanciasYAdictivos' | 'TrastornosNeurocognitivos'
  | 'TrastornosDeLaPersonalidad' | 'TrastornosParafiliicos' | 'OtrosTrastornosMentales'
  | 'TrastornosRelacionadosConProblemasDeSalud';

export interface MapeoDiagnostico {
  id: string;
  patientId: string;
  diagnosticoId: string;
  tipo: DiagnosisType;
  notas?: string;
  fechaCreacion: string;
}

// src/types/age.ts
export interface FormattedAge {
  texto: string; // "45 años, 2 meses y 3 días"
  anos: number;
  meses: number;
  dias: number;
}

export interface AgeBreakdown extends FormattedAge {
  totalDays: number;
  totalMonths: number;
  isMinor: boolean;
  ageOfMajority: number;
  formattedShort: string;
  formattedLong: string;
}
```

### Enum → Spanish Label Map (Single Source)

```typescript
// src/types/enums.ts
export const ENUM_LABEL_MAP: Record<string, Record<string, string>> = {
  DocumentType: {
    CI: 'Cédula de Identidad',
    PASAPORTE: 'Pasaporte',
    RUC: 'RUC',
  },
  Gender: {
    MASCULINO: 'Masculino',
    FEMENINO: 'Femenino',
    OTRO: 'Otro',
  },
  AppointmentStatus: {
    SCHEDULED: 'Programado',
    CONFIRMED: 'Confirmado',
    IN_PROGRESS: 'En curso',
    COMPLETED: 'Completado',
    CANCELLED: 'Cancelado',
    NO_SHOW: 'No asistió',
  },
  NoteType: {
    CONSULTA: 'Consulta',
    CONTROL: 'Control',
    URGENCIA: 'Urgencia',
    INTERCONSULTA: 'Interconsulta',
  },
  DiagnosisType: {
    PRINCIPAL: 'Principal',
    SECUNDARIO: 'Secundario',
    COMORBILIDAD: 'Comorbilidad',
  },
  CategoriaCIE10: {
    EnfermedadesInfecciosas: 'I - Enfermedades infecciosas y parasitarias',
    Neoplasias: 'II - Neoplasias',
    // ... all 22 categories
  },
  CategoriaDSM5: {
    TrastornosNeurodelDesarrollo: 'Trastornos del neurodesarrollo',
    TrastornosDepresivos: 'Trastornos depresivos',
    // ... all 22 categories
  },
};
```

---

## Testing Strategy

| Layer | Tool | Coverage Target | Key Scenarios |
|-------|------|-----------------|---------------|
| **Rust Commands** | `#[cfg(test)]` + `rusqlite` in-memory | 90%+ new code | Validation errors, domain logic, persistence, audit log |
| **Rust Domain** | Unit tests in `domain/src/*.rs` | 95%+ | `AsientoContable::new` validation, `Age::from_birth_date` edge cases, CIE-10/DSM-5 mapping |
| **React Components** | Vitest + React Testing Library | 85%+ | `MetricCard` prop variants, `SearchDropdown` enum mapping, `DiagnosisAutocomplete` debounce/keyboard, `AsientoForm` debit=credit validation |
| **Hooks** | Vitest + MSW (mock invoke) | 80%+ | `useAsientos` cache/invalidation, `useCalculateAge` debounce, `useSearchCIE10` pagination |
| **E2E IPC Round-trips** | Playwright (Tauri dev mode) | 100% commands | Each command: invoke → validate → persist → return |
| **Accessibility** | axe-core + Playwright | Zero violations | Focus rings, ARIA labels, keyboard nav, screen reader |
| **Visual Regression** | Storybook (optional) | Key components | `MetricCard`, `FinancialTable`, `AlertCard` brand compliance |

### Test Organization

```
src-tauri/
├── commands/src/
│   ├── accounting_commands.rs      # #[cfg(test)] mod tests { ... }
│   ├── diagnostics_commands.rs     # #[cfg(test)] mod tests { ... }
│   └── age_commands.rs             # #[cfg(test)] mod tests { ... }
├── domain/src/
│   ├── accounting.rs               # 30+ existing tests
│   ├── diagnostics.rs              # 20+ existing tests
│   └── age.rs                      # 15+ existing tests
└── infrastructure/src/
    └── repositories.rs             # Integration tests with in-memory SQLite

src/
├── components/
│   ├── ui/MetricCard.test.tsx
│   ├── accounting/AsientoForm.test.tsx
│   ├── diagnostics/DiagnosisAutocomplete.test.tsx
│   └── search/SearchDropdown.test.tsx
├── hooks/
│   ├── useAccounting.test.ts
│   ├── useDiagnostics.test.ts
│   └── useAge.test.ts
└── e2e/
    ├── accounting.spec.ts          # Playwright IPC round-trips
    ├── diagnostics.spec.ts
    └── clinical-notes.spec.ts
```

---

## Migration / Rollout

### Database Migrations

New tables required (run via `run_migrations` in `infrastructure/src/migrations.rs`):

```sql
-- Accounting
CREATE TABLE asientos (
    id TEXT PRIMARY KEY,
    fecha TEXT NOT NULL,
    descripcion TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE asiento_lineas (
    id TEXT PRIMARY KEY,
    asiento_id TEXT NOT NULL REFERENCES asientos(id) ON DELETE CASCADE,
    cuenta TEXT NOT NULL,
    debito TEXT NOT NULL DEFAULT '0',
    credito TEXT NOT NULL DEFAULT '0',
    orden INTEGER NOT NULL
);

-- Diagnostics
CREATE TABLE diagnosticos_cie10 (
    codigo TEXT PRIMARY KEY,
    descripcion TEXT NOT NULL,
    categoria TEXT NOT NULL,
    subcategoria TEXT
);

CREATE TABLE diagnosticos_dsm5 (
    codigo TEXT PRIMARY KEY,
    descripcion TEXT NOT NULL,
    categoria TEXT NOT NULL,
    criterios_diagnosticos TEXT, -- JSON array
    especificadores TEXT        -- JSON array
);

CREATE TABLE mapeos_diagnostico (
    id TEXT PRIMARY KEY,
    patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
    diagnostico_id TEXT NOT NULL,
    tipo TEXT NOT NULL,
    notas TEXT,
    fecha_creacion TEXT NOT NULL
);

-- Seed data: CIE-10 (~22k codes) + DSM-5 (~300 codes) loaded via ETL (FASE 1 complete)
```

### Feature Flags (Instant Rollback)

```typescript
// vite.config.ts
export default defineConfig({
  define: {
    'import.meta.env.VITE_FEATURE_ACCOUNTING': JSON.stringify(process.env.VITE_FEATURE_ACCOUNTING ?? 'true'),
    'import.meta.env.VITE_FEATURE_DIAGNOSTICS': JSON.stringify(process.env.VITE_FEATURE_DIAGNOSTICS ?? 'true'),
  },
});
```

```tsx
// src/App.tsx
{import.meta.env.VITE_FEATURE_ACCOUNTING === 'true' && <Route path="/contabilidad" element={<AccountingPage />} />}
{import.meta.env.VITE_FEATURE_DIAGNOSTICS === 'true' && <Route path="/diagnosticos" element={<DiagnosticsPage />} />}
```

### Rollback Plan

1. **Git revert** on merge commit (stacked PRs allow single-change revert)
2. **Database**: No destructive migrations; new tables only. Down-migration SQL provided if needed.
3. **Frontend**: Feature flags disable new pages instantly without rebuild
4. **Tauri Commands**: Comment out entries in `invoke_handler::generate!` in `main.rs`

---

## Open Questions

- [ ] **tauri-specta vs manual sync**: `tauri-specta` adds build complexity. Decision: start with manual sync + CI type-check job; adopt `tauri-specta` in FASE 3 if drift becomes problematic.
- [ ] **PDF export library**: `@react-pdf/renderer` (client-side) vs server-side Rust PDF generation. Decision: client-side `@react-pdf/renderer` for FASE 2 (no extra backend deps), evaluate migration in FASE 3.
- [ ] **CIE-10/DSM-5 seed data size**: ~22k CIE-10 codes. Current ETL loads at startup. Decision: lazy-load search results via `LIMIT 20` + pagination; full dataset stays in SQLite.
- [ ] **Dark mode**: Not in FASE 2 scope. Brand tokens defined for light mode only. Defer to FASE 3.

---

## Next Step

Ready for **task planning (sdd-tasks)**. The design captures:
- 4 new Rust command modules + repo extensions
- 6 new frontend pages/components + hooks + API layer
- Brand token system with Coral restriction enforcement
- Type synchronization strategy
- Test matrix covering unit, integration, E2E, a11y
- Rollback via feature flags + git revert