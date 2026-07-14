# Design: Cognitive Agenda Synchronization with Automated Accounting Triggers (cognitive-agenda-sync)

## Technical Approach

This design implements **FASE 3** of MindLdger: Cognitive Agenda (Agenda Cognitiva) with automated double-entry accounting triggers, OS calendar synchronization, predictive patient search, and KPI dashboard. FASE 2 (Tauri IPC + Frontend SPA) is complete with Accounting, Diagnostics, Age Calculation commands and brand-compliant UI.

The architecture follows **Clean Architecture** principles extended with **Command Query Responsibility Segregation (CQRS)** for appointment lifecycle:

- **Domain Layer** (`mindledger_domain`): Pure Rust, zero dependencies. `Appointment` aggregate with explicit state machine, `CalendarProvider` trait, `ReminderScheduler` domain service, `AccountingTrigger` domain service
- **Infrastructure Layer** (`mindledger_infrastructure`): `SqliteAppointmentRepository`, `SqlcipherCalendarSyncRepository`, `OSCalendarProvider` (macOS EventKit), `GoogleCalendarProvider`, `OutlookCalendarProvider`, `TokioReminderScheduler`
- **Commands Layer** (`mindledger_commands`): Tauri v2 `#[tauri::command]` handlers for appointment CRUD, calendar sync, reminders, KPI queries
- **Frontend** (React/TS + Vite + Tailwind + TanStack Query v5): `/agenda` route with `react-big-calendar`, `KPIPanel`, `SearchDropdown`, `SessionModal`, code-split via `React.lazy`

---

## Architecture Decisions

### Decision: Appointment State Machine

| Option | Tradeoff | Decision |
|--------|----------|----------|
| String status + ad-hoc validation | Flexible, but runtime errors | **Enum `AppointmentStatus` with explicit transitions** |
| Database CHECK constraint only | DB-level safety, no domain logic | **Domain-layer `TransitionValidator` + DB constraint (defense in depth)** |
| Event sourcing | Full audit trail, complex | **State machine + immutable `audit_log` table** (simpler, sufficient) |

**State Machine Definition:**

```
Programada ──▶ Realizada
    │              ▲
    │              │
    ▼              │
Reagendada ────────┘
    │
    ▼
Cancelada
```

**Explicit Transitions (validated at domain + command layer):**
- `Programada` → `Realizada` (on `finalizar_sesion_agenda`)
- `Programada` → `Reagendada` (on `reagendar_cita`, creates new `Appointment` with `reagendada_from_id`)
- `Programada` / `Reagendada` → `Cancelada` (on `cancelar_cita`)
- **No transitions from `Realizada` or `Cancelada`** (terminal states)

---

### Decision: Double-Entry Accounting Trigger

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Separate accounting command | Manual, error-prone | **Automatic in `finalizar_sesion_agenda` transaction** |
| Event-driven (outbox pattern) | Decoupled, eventual consistency | **Single atomic transaction** (strong consistency required for accounting) |
| Domain service called from command | Testable, explicit | **`AccountingTrigger::build_session_asiento()` helper on `AsientoContable`** |

**Atomic Transaction (`finalizar_sesion_agenda`):**
```sql
BEGIN TRANSACTION;
  UPDATE appointments SET status = 'Realizada', updated_at = ? WHERE id = ?;
  INSERT INTO asientos_contables (...) VALUES (...);  -- build_session_asiento()
  INSERT INTO audit_log (entity_type, entity_id, action, payload, user_id, created_at) 
    VALUES ('appointment', ?, 'finalizar_sesion', ?, ?, ?);
COMMIT;
```

**Accounting Entry Structure (`build_session_asiento`):**
- **Debit:** `1.1.1.01 Caja/Banco` (or `1.1.2.01 Cuentas por Cobrar` if not paid) — `fee_cents`
- **Credit:** `4.1.1.01 Honorarios Profesionales` — `fee_cents`
- **Description:** `Sesión: {patient_name} - {appointment_date} - {modality}`

---

### Decision: CalendarProvider Trait

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Single provider with config | Simple, but platform-specific code leaks | **`async_trait` + `Send + Sync` trait with platform implementations** |
| Separate crates per provider | Clean separation, build complexity | **Single crate, feature-gated implementations** (`os-calendar`, `google-calendar`, `outlook-calendar`) |

```rust
#[async_trait]
pub trait CalendarProvider: Send + Sync {
    async fn create_event(&self, event: CalendarEvent) -> Result<String, CalendarError>;
    async fn update_event(&self, external_id: &str, event: CalendarEvent) -> Result<(), CalendarError>;
    async fn delete_event(&self, external_id: &str) -> Result<(), CalendarError>;
    async fn list_events(&self, range: DateRange) -> Result<Vec<CalendarEvent>, CalendarError>;
}
```

**Implementations:**
- `OSCalendarProvider` (macOS EventKit via `objc2`/`eventkit` crate) — **feature `os-calendar`**
- `GoogleCalendarProvider` (OAuth2 + Google Calendar API) — **feature `google-calendar`**
- `OutlookCalendarProvider` (Microsoft Graph API) — **feature `outlook-calendar`**

---

### Decision: Reminder Scheduler

| Option | Tradeoff | Decision |
|--------|----------|----------|
| System cron / launchd | Reliable, external dependency | **Tokio interval task supervised by Tauri runtime** |
| External scheduler (Redis + worker) | Scalable, overkill for desktop | **In-process Tokio task** (single-user desktop app) |
| One-shot timers per appointment | Precise, resource-heavy | **1-minute tick + due-check** (negligible overhead) |

```rust
pub struct TokioReminderScheduler {
    repo: Arc<dyn AppointmentRepository>,
    notifier: Arc<dyn ReminderNotifier>,
    interval: Duration,
}

impl TokioReminderScheduler {
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;
            let due = self.repo.find_reminders_due(Utc::now()).await?;
            for appt in due {
                self.notifier.send(&appt).await?;
                self.repo.mark_reminder_sent(appt.id).await?;
            }
        }
    }
}
```
- **Tick interval:** 1 minute (configurable via `ReminderSchedulerConfig`)
- **Supervision:** Spawned from `main.rs` via `tauri::async_runtime::spawn` with `AbortHandle` for graceful shutdown

---

### Decision: Frontend Calendar

| Option | Tradeoff | Decision |
|--------|----------|----------|
| FullCalendar (jQuery/React) | Feature-rich, heavy, licensing | **`react-big-calendar`** (MIT, lighter, React-native) |
| Custom calendar | Full control, high effort | **`react-big-calendar` + custom views** |
| `react-day-picker` | Date picking only | **Not a calendar view** |

**Configuration:**
- Views: `Month`, `Week`, `Day`, `Agenda` (react-big-calendar built-in + `Agenda` custom)
- Locale: Spanish (`es`), week starts Monday
- Code-split: `const AgendaPage = lazy(() => import('./pages/AgendaPage'))` on `/agenda` route
- Event rendering: Custom `EventComponent` with status color dot + patient name + modality icon

---

### Decision: Predictive Patient Search

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Native `<select>` with all patients | Simple, O(n) render | **Debounced search + TanStack Query cache** |
| react-select / downshift | Accessible, heavy | **Custom `SearchDropdown`** (native `<select>` + Tailwind, keyboard nav) |
| Server-side search only | No offline, latency | **Client-side filter on cached TanStack Query data** (patients ≤ 5k typical) |

**Implementation:**
```tsx
const usePatientSearch = (query: string) => {
  const { data: patients } = useQuery({ queryKey: ['patients'], queryFn: patientApi.list });
  const debounced = useDebounce(query, 300);
  return useMemo(() => patients?.filter(p => 
    p.name.toLowerCase().includes(debounced.toLowerCase()) ||
    p.document_number.includes(debounced)
  ) ?? [], [patients, debounced]);
};
```
- **Component:** `SearchDropdown` (reusable, native `<select>`, `optgroup` for recent/active/inactive)
- **Integration:** `SessionModal` patient combobox → `usePatientSearch`

---

## Data Models

### Database Schema Changes

**Existing tables (reuse):**
- `asientos_contables` — double-entry accounting (from FASE 2)
- `audit_log` — immutable audit trail (from FASE 2)

**Extended `appointments` table:**
```sql
-- Existing columns: id, patient_id, professional_id, start_at, end_at, modality, notes, created_at, updated_at
ALTER TABLE appointments ADD COLUMN status TEXT NOT NULL DEFAULT 'Programada' 
  CHECK (status IN ('Programada','Realizada','Reagendada','Cancelada'));
ALTER TABLE appointments ADD COLUMN fee_cents INTEGER NOT NULL DEFAULT 0;
ALTER TABLE appointments ADD COLUMN reminder_sent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE appointments ADD COLUMN reminder_external_id TEXT;  -- OS calendar notification ID
ALTER TABLE appointments ADD COLUMN reagendada_from_id TEXT REFERENCES appointments(id);
ALTER TABLE appointments ADD COLUMN external_calendar_id TEXT;  -- Google/Outlook event ID
ALTER TABLE appointments ADD COLUMN calendar_provider TEXT;     -- 'os' | 'google' | 'outlook'

-- Indexes for reminder scheduler + queries
CREATE INDEX idx_appointments_reminder_due ON appointments(start_at) WHERE reminder_sent = 0;
CREATE INDEX idx_appointments_status ON appointments(status);
CREATE INDEX idx_appointments_patient_date ON appointments(patient_id, start_at);
```

**`asientos_contables` / `asiento_lineas`** — unchanged (FASE 2)

**`audit_log`** — unchanged (FASE 2)

---

### Domain Models (Rust)

```rust
// mindledger_domain/src/appointment.rs

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AppointmentStatus {
    Programada,
    Realizada,
    Reagendada,
    Cancelada,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub professional_id: Uuid,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub modality: Modality,
    pub status: AppointmentStatus,
    pub fee_cents: i64,
    pub notes: Option<String>,
    pub reminder_sent: bool,
    pub reminder_external_id: Option<String>,
    pub reagendada_from_id: Option<Uuid>,
    pub external_calendar_id: Option<String>,
    pub calendar_provider: Option<CalendarProviderType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// State machine transitions
impl Appointment {
    pub fn transition(&mut self, next: AppointmentStatus) -> DomainResult<()> {
        use AppointmentStatus::*;
        match (self.status.clone(), next.clone()) {
            (Programada, Realizada) | (Programada, Reagendada) | (Programada, Cancelada) |
            (Reagendada, Realizada) | (Reagendada, Cancelada) => {
                self.status = next;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(DomainError::InvalidTransition {
                from: self.status.clone(),
                to: next,
            }),
        }
    }
}
```

```rust
// mindledger_domain/src/accounting_trigger.rs

pub struct AccountingTrigger;

impl AccountingTrigger {
    pub fn build_session_asiento(
        appointment: &Appointment,
        patient: &Patient,
        professional: &Professional,
    ) -> AsientoContable {
        let descripcion = format!(
            "Sesión: {} - {} - {}",
            patient.full_name,
            appointment.start_at.format("%d/%m/%Y"),
            appointment.modality.label()
        );

        let mut asiento = AsientoContable::new(
            appointment.start_at.date_naive(),
            descripcion,
        );

        // Debit: Caja/Banco or Cuentas por Cobrar
        let cuenta_debito = if appointment.paid_at.is_some() {
            "1.1.1.01"  // Caja/Banco
        } else {
            "1.1.2.01"  // Cuentas por Cobrar
        };
        asiento.add_linea(cuenta_debito, appointment.fee_cents, 0, 1);

        // Credit: Honorarios Profesionales
        asiento.add_linea("4.1.1.01", 0, appointment.fee_cents, 2);

        asiento
    }
}
```

---

### TypeScript Types (Frontend)

```typescript
// src/types/appointment.ts

export type AppointmentStatus = 
  | 'Programada' 
  | 'Realizada' 
  | 'Reagendada' 
  | 'Cancelada';

export type CalendarProviderType = 'os' | 'google' | 'outlook';

export type Modality = 'Presencial' | 'Virtual' | 'Hibrida';

export interface Appointment {
  id: string;
  patient_id: string;
  professional_id: string;
  start_at: string;           // ISO 8601
  end_at: string;
  modality: Modality;
  status: AppointmentStatus;
  fee_cents: number;
  notes?: string;
  reminder_sent: boolean;
  reminder_external_id?: string;
  reagendada_from_id?: string;
  external_calendar_id?: string;
  calendar_provider?: CalendarProviderType;
  created_at: string;
  updated_at: string;
}

export interface AppointmentWithRelations extends Appointment {
  patient: Patient;
  professional: Professional;
}

export interface SessionPayload {
  patient_id: string;
  professional_id: string;
  start_at: string;
  end_at: string;
  modality: Modality;
  fee_cents: number;
  notes?: string;
}

export interface ReagendarPayload {
  appointment_id: string;
  new_start_at: string;
  new_end_at: string;
  notes?: string;
}

export interface KPIMetrics {
  ocupacion_semanal_pct: number;      // 0-100
  ingresos_mes_cents: number;
  no_show_rate_pct: number;           // 0-100
  pacientes_nuevos_mes: number;
}

export const APPOINTMENT_STATUS_COLORS: Record<AppointmentStatus, string> = {
  Programada: '#0F4C5C',   // Primary
  Realizada: '#28A745',    // Success green
  Reagendada: '#FFC107',   // Warning amber
  Cancelada: '#E3645F',    // Coral (restricted)
};

export const APPOINTMENT_STATUS_LABELS: Record<AppointmentStatus, string> = {
  Programada: 'Programada',
  Realizada: 'Realizada',
  Reagendada: 'Reagendada',
  Cancelada: 'Cancelada',
};
```

---

## IPC Contracts

All commands return typed `AppResult<T>` via `tauri-specta` generated types.

### Appointment Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `crear_cita_agenda` | `SessionPayload` | `Appointment` | Create appointment, schedule reminder, sync to calendar |
| `listar_citas_agenda` | `{ range: DateRange; status?: AppointmentStatus; patient_id?: string }` | `AppointmentWithRelations[]` | Filtered list for calendar view |
| `obtener_cita_agenda` | `{ id: string }` | `AppointmentWithRelations` | Single appointment detail |
| `finalizar_sesion_agenda` | `{ id: string; notes?: string }` | `Appointment` | **Atomic**: status→Realizada + asiento_contable + audit_log |
| `reagendar_cita` | `ReagendarPayload` | `Appointment` | Creates new `Appointment` (status=Reagendada), links via `reagendada_from_id` |
| `cancelar_cita` | `{ id: string; reason: string }` | `Appointment` | Status→Cancelada, cancels reminders, deletes calendar event |

### Calendar Sync Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `sincronizar_calendario` | `{ provider: CalendarProviderType; range: DateRange }` | `SyncResult { created: number; updated: number; deleted: number }` | Bidirectional sync with OS/Google/Outlook |
| `configurar_proveedor_calendario` | `{ provider: CalendarProviderType; config: CalendarConfig }` | `void` | Store OAuth tokens / grant permissions |

### Reminder Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `programar_recordatorio` | `{ appointment_id: string; minutes_before: number }` | `void` | Schedule OS notification (stores `reminder_external_id`) |
| `cancelar_recordatorio` | `{ appointment_id: string }` | `void` | Cancel scheduled notification |

### KPI Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `obtener_kpis_agenda` | `{ professional_id?: string; range: DateRange }` | `KPIMetrics` | Real-time dashboard metrics |

---

### Frontend API Layer

```typescript
// src/api/agendaApi.ts

export const agendaApi = {
  crearCita: (payload: SessionPayload) => 
    invoke<Appointment>('crear_cita_agenda', { payload }),
  
  listarCitas: (params: { 
    range: DateRange; 
    status?: AppointmentStatus; 
    patient_id?: string 
  }) => invoke<AppointmentWithRelations[]>('listar_citas_agenda', params),
  
  obtenerCita: (id: string) => 
    invoke<AppointmentWithRelations>('obtener_cita_agenda', { id }),
  
  finalizarSesion: (id: string, notes?: string) => 
    invoke<Appointment>('finalizar_sesion_agenda', { id, notes }),
  
  reagendarCita: (payload: ReagendarPayload) => 
    invoke<Appointment>('reagendar_cita', { payload }),
  
  cancelarCita: (id: string, reason: string) => 
    invoke<Appointment>('cancelar_cita', { id, reason }),
  
  sincronizarCalendario: (provider: CalendarProviderType, range: DateRange) => 
    invoke<SyncResult>('sincronizar_calendario', { provider, range }),
  
  obtenerKPIs: (range: DateRange, professional_id?: string) => 
    invoke<KPIMetrics>('obtener_kpis_agenda', { range, professional_id }),
};
```

### TanStack Query Hooks

```typescript
// src/hooks/useAgenda.ts

export const useCitas = (params: { range: DateRange; status?: AppointmentStatus }) => 
  useQuery({ 
    queryKey: ['citas', params], 
    queryFn: () => agendaApi.listarCitas(params),
    staleTime: 30_000,
  });

export const useKPIs = (range: DateRange) => 
  useQuery({ 
    queryKey: ['kpis', range], 
    queryFn: () => agendaApi.obtenerKPIs(range),
    refetchInterval: 60_000,  // Refresh every minute
  });

export const useFinalizarSesion = () => 
  useMutation({ 
    mutationFn: ({ id, notes }: { id: string; notes?: string }) => 
      agendaApi.finalizarSesion(id, notes),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['citas'] });
      queryClient.invalidateQueries({ queryKey: ['kpis'] });
      queryClient.invalidateQueries({ queryKey: ['asientos'] });  // Accounting sync
    },
  });
```

---

## File Changes

### New Files (Backend - Rust)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/domain/src/appointment.rs` | Create | `Appointment` aggregate, `AppointmentStatus` enum, state machine, `TransitionValidator` |
| `src-tauri/domain/src/calendar_provider.rs` | Create | `CalendarProvider` trait, `CalendarEvent`, `CalendarError`, `CalendarProviderType` enum |
| `src-tauri/domain/src/reminder_scheduler.rs` | Create | `ReminderScheduler` trait, `ReminderNotifier` trait, `TokioReminderScheduler` |
| `src-tauri/domain/src/accounting_trigger.rs` | Create | `AccountingTrigger::build_session_asiento()` helper |
| `src-tauri/infrastructure/src/calendar_providers/os_calendar.rs` | Create | `OSCalendarProvider` (macOS EventKit) — feature `os-calendar` |
| `src-tauri/infrastructure/src/calendar_providers/google_calendar.rs` | Create | `GoogleCalendarProvider` (OAuth2 + API) — feature `google-calendar` |
| `src-tauri/infrastructure/src/calendar_providers/outlook_calendar.rs` | Create | `OutlookCalendarProvider` (MS Graph) — feature `outlook-calendar` |
| `src-tauri/infrastructure/src/reminder_scheduler.rs` | Create | `TokioReminderScheduler` implementation |
| `src-tauri/infrastructure/src/repositories/appointment_repo.rs` | Create | `SqliteAppointmentRepository` with reminder-due queries |
| `src-tauri/infrastructure/src/database.rs` | Modify | Add migrations for `appointments` extensions |
| `src-tauri/commands/src/agenda_commands.rs` | Create | 7 appointment IPC commands |
| `src-tauri/commands/src/calendar_commands.rs` | Create | 2 calendar sync commands |
| `src-tauri/commands/src/reminder_commands.rs` | Create | 2 reminder commands |
| `src-tauri/commands/src/kpi_commands.rs` | Create | 1 KPI query command |
| `src-tauri/commands/src/error.rs` | Modify | Add `AgendaError`, `CalendarError`, `ReminderError` variants |
| `src-tauri/commands/src/lib.rs` | Modify | Re-export new command modules |
| `src-tauri/app/src/main.rs` | Modify | Register commands, spawn `TokioReminderScheduler` |

### New Files (Frontend - React/TypeScript)

| File | Action | Description |
|------|--------|-------------|
| `src/types/appointment.ts` | Create | TypeScript interfaces for `Appointment`, `AppointmentStatus`, `KPIMetrics`, status colors/labels |
| `src/api/agendaApi.ts` | Create | `invoke` wrappers for all agenda/calendar/reminder/KPI commands |
| `src/hooks/useAgenda.ts` | Create | TanStack Query hooks: `useCitas`, `useKPIs`, `useFinalizarSesion`, `useReagendar`, `useCancelar` |
| `src/hooks/usePatientSearch.ts` | Create | `usePatientSearch` with debounced filtering on cached patient data |
| `src/pages/AgendaPage.tsx` | Create | Main calendar page (code-split), `react-big-calendar` integration |
| `src/components/agenda/CalendarView.tsx` | Create | Wrapper for `react-big-calendar` with Spanish locale, custom views |
| `src/components/agenda/EventComponent.tsx` | Create | Custom event rendering: status dot, patient name, modality icon |
| `src/components/agenda/KPIPanel.tsx` | Create | Sage Green bg (`#E5F1EE`), 4 `MetricCard`s, reactive via TanStack Query |
| `src/components/agenda/SessionModal.tsx` | Create | Modal form: patient `SearchDropdown`, fee defaults to `patient.session_fee_cents`, modality, notes |
| `src/components/search/SearchDropdown.tsx` | Create | Reusable native `<select>` with debounced search, keyboard nav, Tailwind styling |
| `src/components/ui/MetricCard.tsx` | Reuse | Existing from FASE 2 (Sage bg, icon, title, value, trend) |
| `src/components/layout/Layout.tsx` | Modify | Add "Agenda" nav item (Calendar icon) |
| `src/index.css` | Modify | Add `--color-sage`, `--color-coral`, `--color-success` CSS custom properties |
| `tailwind.config.js` | Modify | Extend theme with status colors, ensure Sage/Coral tokens |
| `vite.config.ts` | Modify | Add `React.lazy` code-split for `/agenda` route |

### Removed Files

| File | Reason |
|------|--------|
| None | All additive changes |

---

## Migration / Rollout

### Database Migrations

Run via `run_migrations` in `infrastructure/src/migrations.rs` (idempotent):

```sql
-- Migration: 2025_01_15_001_extend_appointments.sql
ALTER TABLE appointments ADD COLUMN status TEXT NOT NULL DEFAULT 'Programada' 
  CHECK (status IN ('Programada','Realizada','Reagendada','Cancelada'));
ALTER TABLE appointments ADD COLUMN fee_cents INTEGER NOT NULL DEFAULT 0;
ALTER TABLE appointments ADD COLUMN reminder_sent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE appointments ADD COLUMN reminder_external_id TEXT;
ALTER TABLE appointments ADD COLUMN reagendada_from_id TEXT REFERENCES appointments(id);
ALTER TABLE appointments ADD COLUMN external_calendar_id TEXT;
ALTER TABLE appointments ADD COLUMN calendar_provider TEXT;

CREATE INDEX idx_appointments_reminder_due ON appointments(start_at) WHERE reminder_sent = 0;
CREATE INDEX idx_appointments_status ON appointments(status);
CREATE INDEX idx_appointments_patient_date ON appointments(patient_id, start_at);
CREATE INDEX idx_appointments_reagendada_from ON appointments(reagendada_from_id);
```

### Feature Flags (Instant Rollback)

```typescript
// vite.config.ts
export default defineConfig({
  define: {
    'import.meta.env.VITE_FEATURE_AGENDA': JSON.stringify(process.env.VITE_FEATURE_AGENDA ?? 'true'),
    'import.meta.env.VITE_FEATURE_CALENDAR_SYNC': JSON.stringify(process.env.VITE_FEATURE_CALENDAR_SYNC ?? 'true'),
    'import.meta.env.VITE_FEATURE_REMINDERS': JSON.stringify(process.env.VITE_FEATURE_REMINDERS ?? 'true'),
    'import.meta.env.VITE_FEATURE_KPI_PANEL': JSON.stringify(process.env.VITE_FEATURE_KPI_PANEL ?? 'true'),
  },
});
```

```tsx
// src/App.tsx
{import.meta.env.VITE_FEATURE_AGENDA === 'true' && (
  <Route path="/agenda" element={<Suspense fallback={<Loader />}><AgendaPage /></Suspense>} />
)}
```

### Rollback Plan

1. **Git revert** on merge commit (stacked PRs allow single-change revert)
2. **Database**: No destructive migrations; new columns only. Down-migration SQL provided:
   ```sql
   ALTER TABLE appointments DROP COLUMN status;
   ALTER TABLE appointments DROP COLUMN fee_cents;
   ALTER TABLE appointments DROP COLUMN reminder_sent;
   ALTER TABLE appointments DROP COLUMN reminder_external_id;
   ALTER TABLE appointments DROP COLUMN reagendada_from_id;
   ALTER TABLE appointments DROP COLUMN external_calendar_id;
   ALTER TABLE appointments DROP COLUMN calendar_provider;
   DROP INDEX idx_appointments_reminder_due;
   DROP INDEX idx_appointments_status;
   DROP INDEX idx_appointments_patient_date;
   DROP INDEX idx_appointments_reagendada_from;
   ```
3. **Frontend**: Feature flags disable new pages/components instantly without rebuild
4. **Tauri Commands**: Comment out entries in `invoke_handler::generate!` in `main.rs`
5. **Background Task**: `ReminderScheduler` abort handle dropped on flag disable

---

## UI/UX Specifications

### Calendar View (`react-big-calendar`)

| Aspect | Specification |
|--------|---------------|
| **Views** | Month, Week, Day, Agenda (custom) |
| **Locale** | Spanish (`es`), week starts Monday |
| **Time format** | 24-hour (HH:mm) |
| **Slot duration** | 30 minutes |
| **Event component** | Custom `EventComponent`: 8px status dot (left), patient name, modality icon (🏠/💻/🔄) |
| **Selection** | Click event → `SessionModal` (view/edit) |
| **Drag & drop** | Disabled for FASE 3 (enable in FASE 4) |
| **Code-split** | `React.lazy` on `/agenda` route |

### Status Colors (Strict)

| Status | Color Token | HEX | Usage |
|--------|-------------|-----|-------|
| Programada | `--color-primary` | `#0F4C5C` | Event dot, badge, primary actions |
| Realizada | `--color-success` | `#28A745` | Event dot, badge, KPI positive |
| Reagendada | `--color-warning` | `#FFC107` | Event dot, badge, warning states |
| Cancelada | `--color-coral` | `#E3645F` | **Only** cancellation alerts, net loss |

> **ESLint Rule:** `no-restricted-color-coral` — Coral (`#E3645F`) ONLY for cancellation/loss contexts. CI fails on violation.

### KPIPanel

- **Background:** Sage Green (`#E5F1EE` / `--color-sage`)
- **Layout:** 4 `MetricCard`s in responsive grid (1 col mobile, 2 tablet, 4 desktop)
- **Metrics:**
  1. **Ocupación Semanal** — `%` with trend arrow (vs previous week)
  2. **Ingresos del Mes** — Currency (ARS/USD), green if ↑
  3. **No-Show Rate** — `%`, red if > 15%
  4. **Pacientes Nuevos** — Count, blue trend
- **Reactivity:** TanStack Query cache invalidation on appointment mutations + 60s `refetchInterval`

### SessionModal

- **Trigger:** Click calendar event OR "Nueva Cita" FAB
- **Fields:**
  - Patient: `SearchDropdown` (debounced 300ms, shows recent/active/inactive groups)
  - Professional: `<select>` (pre-filled with current user)
  - Date/Time: `start_at`, `end_at` (default 50 min)
  - Modality: Radio group (Presencial / Virtual / Híbrida)
  - Fee: Number input (cents), **defaults to `patient.session_fee_cents`**
  - Notes: Textarea
- **Actions:** Cancel / Guardar (crear) / Finalizar (if editing Programada)

### SearchDropdown (Reusable)

- **Base:** Native `<select>` + `<option>` / `<optgroup>` (accessible, lightweight)
- **Styling:** Tailwind `appearance-none`, custom chevron, focus ring `--color-primary`
- **Search:** Debounced 300ms input above dropdown (filters options client-side)
- **Keyboard:** Arrow up/down, Enter to select, Escape to close
- **Groups:** `<optgroup label="Recientes">`, `<optgroup label="Activos">`, `<optgroup label="Inactivos">`

---

## Open Questions

- [ ] **Calendar provider default**: macOS EventKit (OS) as default? Requires "Full Calendar Access" permission — UX for permission prompt needed.
- [ ] **Google/Outlook OAuth storage**: Encrypt tokens in SQLite vs Keychain/Secret Service. Decision: Keychain (macOS) / libsecret (Linux) / Credential Manager (Windows) via `keyring` crate.
- [ ] **Reminder lead time**: Configurable per patient? Default 30 min. Add `patient.reminder_minutes_before` in FASE 4.
- [ ] **Timezone handling**: Appointments stored UTC, displayed in local. Calendar providers may return different TZ. Normalize to UTC on sync.
- [ ] **Recurring appointments**: Not in FASE 3 scope. Design for RRULE extension in `appointments` table (FASE 4).
- [ ] **Conflict detection**: Overlapping appointments for same professional. Add validation in `crear_cita_agenda` / `reagendar_cita`.
- [ ] **Multi-professional calendar**: Filter by `professional_id` in `listar_citas_agenda`. UI: professional selector in header.

---

## Next Step

Ready for **task planning (sdd-tasks)**. The design captures:
- 4 new domain modules (appointment, calendar_provider, reminder_scheduler, accounting_trigger)
- 3 calendar provider implementations (feature-gated)
- 1 background scheduler (Tokio interval, supervised)
- 12 new IPC commands across 4 command modules
- 6 new frontend pages/components + hooks + API layer
- Atomic accounting trigger in `finalizar_sesion_agenda`
- Brand token compliance with Coral restriction enforcement
- Type synchronization via `tauri-specta` (or manual + CI check)
- Full test matrix: domain, integration, component, E2E
- Rollback via feature flags + git revert + down-migrations