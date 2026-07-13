# Spec: Cognitive Agenda Synchronization with Automated Accounting Triggers

## Change ID
`cognitive-agenda-sync`

## Overview
This spec defines the Agenda domain with appointment lifecycle management, automated double-entry accounting triggers on session completion, OS-level reminder notifications with CalendarProvider abstraction for future Google/Outlook sync, and a reactive AgendaPage with calendar view and KPI panel. FASE 1 (Domain Core, Encrypted Persistence, ETL Documental) and FASE 2 (Tauri IPC + Frontend SPA) are complete. This change delivers FASE 3: Agenda & Accounting Integration.

---

## Capability 1: domain-entities

### Capability ID
`domain-entities`

### Capability Name
Agenda & Accounting Domain Entities

### Description
Core domain layer for appointments, accounting integration helpers, and reminder service abstractions. Pure Rust domain with zero Tauri/extern dependencies — portable, testable in isolation.

### Invariants Preserved
- **I-01**: Appointment state machine strictly enforced — no invalid transitions (Programada → Realizada/Reagendada/Cancelada; Reagendada → Realizada/Cancelada; Cancelada terminal)
- **I-02**: Double-entry accounting helper guarantees debit = credit; never persists imbalanced entries
- **I-03**: CalendarProvider trait is `Send + Sync + 'static` — backend-agnostic, testable with mock
- **I-04**: No floating-point in money — all amounts in centavos (i64)
- **I-05**: Domain layer has zero external crate dependencies except `chrono`, `uuid`, `rust_decimal`
- **I-06**: TDD — every domain function has unit test; state machine exhaustively tested
- **I-07**: Immutable audit trail — all state transitions append to `audit_log` via domain event

### Input
| Entity / Helper | Fields / Methods | Validation Rules |
|-----------------|------------------|------------------|
| **Appointment** | `id: Uuid, patient_id: Uuid, starts_at: DateTime<Utc>, ends_at: DateTime<Utc>, status: AppointmentStatus, fee_cents: i64, notes: Option<String>, reminder_sent: bool, created_at: DateTime<Utc>, updated_at: DateTime<Utc>` | `starts_at < ends_at`; duration ≥ 15 min; fee_cents ≥ 0; status enum valid |
| **AppointmentStatus** | `Programada \| Realizada \| Reagendada \| Cancelada` | State machine transitions only (see Processing) |
| **AccountingHelper** | `build_session_asiento(appointment: &Appointment, patient: &Patient) -> Result<AsientoContable, AccountingError>` | Debit 1110 (Caja) = Credit 4110 (Ingresos Servicios Clínicos); amount = appointment.fee_cents or patient.session_fee_cents |
| **ReminderDomain** | `schedule_reminder(appointment: &Appointment) -> Result<(), ReminderError>`, `cancel_reminder(appointment_id: Uuid) -> Result<(), ReminderError>`, `process_due_reminders(now: DateTime<Utc>) -> Vec<Appointment>` | 30-min before `starts_at`; idempotent; CalendarProvider trait for OS notification |

### Processing
1. **Appointment State Machine**:
   - `Programada` → `Realizada` (trigger: `finalizar_sesion_agenda`)
   - `Programada` → `Reagendada` (new `starts_at`/`ends_at`; old slot freed)
   - `Programada` → `Cancelada` (reason required)
   - `Reagendada` → `Realizada` / `Cancelada`
   - `Cancelada` / `Realizada` are terminal

2. **Double-Entry Accounting Helper** (`domain/src/accounting.rs`):
   - Input: `Appointment` (with `fee_cents`) + `Patient` (fallback `session_fee_cents`)
   - Output: `AsientoContable { date: NaiveDate, descripcion: String, detalles: vec![
       Detalle { cuenta: "1110", nombre: "Caja", debe: amount, haber: 0 },
       Detalle { cuenta: "4110", nombre: "Ingresos Servicios Clínicos", debe: 0, haber: amount }
     ]}`
   - Invariant: `sum(debe) == sum(haber)` always; amount in centavos

3. **CalendarProvider Trait** (`domain/src/reminder.rs`):
   ```rust
   #[async_trait]
   pub trait CalendarProvider: Send + Sync {
       async fn schedule_notification(&self, appointment: &Appointment) -> Result<String, ReminderError>;
       async fn cancel_notification(&self, external_id: &str) -> Result<(), ReminderError>;
   }
   ```
   - Default impl: `OsNotificationProvider` (macOS `osascript` / Windows `toast` / Linux `notify-send`)
   - Future: `GoogleCalendarProvider`, `OutlookCalendarProvider`

### Output
| Entity / Helper | Output |
|-----------------|--------|
| **Appointment** | Persisted via `AppointmentRepository`; state transitions emit `AppointmentEvent` for audit |
| **AccountingHelper** | `AsientoContable` ready for `add_asiento` IPC command |
| **ReminderDomain** | Scheduled OS notification at `starts_at - 30min`; `reminder_sent = true` on appointment |

### Acceptance Criteria
- [ ] `Appointment::new()` validates all invariants; invalid returns `DomainError::Validation`
- [ ] State machine: 100% transition coverage in tests (valid + invalid transitions)
- [ ] `AccountingHelper::build_session_asiento()` produces balanced entry; uses patient `session_fee_cents` if appointment `fee_cents = 0`
- [ ] `CalendarProvider` trait compiles; `OsNotificationProvider` sends test notification on macOS/Windows/Linux
- [ ] `ReminderDomain::process_due_reminders()` returns only unsent reminders within ±1 min window
- [ ] All domain functions pure — zero I/O, zero Tauri types
- [ ] Unit tests: `cargo test -p mindledger-domain` 100% pass; >95% coverage on new code

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| State machine edge cases (re-schedule after cancel) | Medium | High | Exhaustive property-based tests with `proptest` |
| Centavo rounding errors in accounting | Low | High | `rust_decimal::Decimal` for internal math; cast to i64 centavos only at persistence |
| OS notification permission denied | Medium | Medium | Graceful degradation; log warning; appointment still created |
| CalendarProvider trait object safety | Low | Medium | Keep trait simple; no generics; `async_trait` for async methods |

---

## Capability 2: ipc-commands

### Capability ID
`ipc-commands`

### Capability Name
Agenda & Reminder IPC Commands

### Description
Tauri v2 commands (`#[tauri::command]`) exposing appointment lifecycle, atomic session-finalization with double-entry accounting, and reminder service control. All commands atomic, validated, audited.

### Invariants Preserved
- **I-01**: Existing data never deleted — soft delete only; appointments retain audit trail
- **I-02**: Each command single transaction — `finalizar_sesion_agenda` = appointment UPDATE + asiento INSERT + audit INSERT in one SQLCipher transaction
- **I-03**: Command signatures stable — TypeScript types auto-generated via `tauri-specta`
- **I-04**: Shared repository pattern — `AppointmentRepository` single source of truth
- **I-05**: No duplicate persistence — accounting uses existing `add_asiento` internally
- **I-06**: TDD — every command has integration test with in-memory SQLite
- **I-07**: Immutable audit — every write appends to `audit_log`
- **I-08**: WAL mode + connection pool (max 10) — no deadlocks
- **I-09**: Read commands use existing repositories
- **I-10**: Tests use in-memory SQLite — no file I/O
- **I-11**: Domain layer pure — no Tauri types in `domain/`

### Input
| Command | Parameters | Validation Rules |
|---------|------------|------------------|
| **Appointment CRUD** | | |
| `create_appointment` | `patient_id: Uuid, starts_at: DateTime<Utc>, ends_at: DateTime<Utc>, fee_cents: Option<i64>, notes: Option<String>` | Patient exists; `starts_at < ends_at`; duration ≥ 15 min; no overlapping `Programada` for same patient; fee_cents ≥ 0 if provided |
| `get_appointment` | `id: Uuid` | UUID v4 |
| `list_appointments` | `date_range: Option<DateRange>, patient_id: Option<Uuid>, status: Option<AppointmentStatus>, limit: Option<u32>, offset: Option<u32>` | Date range max 365 days; limit ≤ 100 |
| `update_appointment` | `id: Uuid, fields: AppointmentUpdateFields` | At least one field; if `starts_at`/`ends_at` changed → no overlap; status change only via transition commands |
| `delete_appointment` | `id: Uuid` | Soft delete (status → `Cancelada` with reason) |
| **State Transitions** | | |
| `finalizar_sesion_agenda` | `appointment_id: Uuid, notes: Option<String>` | Appointment exists, status = `Programada`; atomic: UPDATE appointment → INSERT asiento → INSERT audit_log |
| `reagendar_appointment` | `appointment_id: Uuid, new_starts_at: DateTime<Utc>, new_ends_at: DateTime<Utc>, reason: String` | Status = `Programada` or `Reagendada`; new slot no overlap; reason non-empty |
| `cancelar_appointment` | `appointment_id: Uuid, reason: String` | Status = `Programada` or `Reagendada`; reason non-empty |
| **Reminder Commands** | | |
| `schedule_reminder` | `appointment_id: Uuid` | Appointment exists, status = `Programada`, `reminder_sent = false` |
| `cancel_reminder` | `appointment_id: Uuid` | Appointment exists |
| `process_due_reminders` | *(none)* | Background job trigger; returns count processed |

### Processing
1. **Command Invocation**: Frontend calls `invoke("command_name", args)` via `@tauri-apps/api/core`
2. **Validation**: `validator` crate on input; `AppError::Validation` with Spanish messages
3. **State Access**: `State<'_, AppState>` provides `DbPool` (rusqlite pool, WAL mode)
4. **Domain Logic**: Delegates to `domain/src/appointment.rs`, `domain/src/accounting.rs`, `domain/src/reminder.rs`
5. **Persistence**: `AppointmentRepository` (implements `AppointmentRepo` trait) executes parameterized SQL
6. **Atomic Transaction** (`finalizar_sesion_agenda`):
   ```sql
   BEGIN TRANSACTION;
   UPDATE appointments SET status = 'Realizada', notes = ?, updated_at = ? WHERE id = ? AND status = 'Programada';
   INSERT INTO asientos_contables (...) VALUES (...);  -- from AccountingHelper
   INSERT INTO audit_log (entity_type, entity_id, action, payload, created_at) VALUES ('appointment', ?, 'finalizar_sesion', ?, ?);
   COMMIT;
   ```
7. **Audit**: Every write appends to `audit_log` (immutable, append-only)
8. **Serialization**: Return types implement `Serialize` → JSON to frontend

### Output
| Command | Success Response | Error Response |
|---------|------------------|----------------|
| Appointment CRUD | `Appointment` / `Vec<Appointment>` / `Paginated<Appointment>` / `bool` | `AppError::Validation` / `AppError::NotFound` / `AppError::Conflict` |
| `finalizar_sesion_agenda` | `AsientoContable` (created asiento) | `AppError::InvalidTransition` / `AppError::AccountingError` |
| State transitions | `Appointment` (updated) | `AppError::InvalidTransition` / `AppError::Conflict` |
| Reminder commands | `bool` / `usize` | `AppError::ReminderError` |

### Acceptance Criteria
- [ ] All 10 appointment commands implemented + integration tests (100% coverage on new logic)
- [ ] `finalizar_sesion_agenda`: atomic transaction verified — failure at any step rolls back all
- [ ] Double-entry asiento created with Debit 1110 = Credit 4110; amount = appointment.fee_cents or patient.session_fee_cents
- [ ] State machine enforced at command level — invalid transitions return `AppError::InvalidTransition`
- [ ] Reminder commands schedule/cancel OS notifications via `CalendarProvider`
- [ ] `process_due_reminders` callable manually + via background job (tokio interval)
- [ ] All commands registered in `invoke_handler::generate!` in `main.rs`
- [ ] `tauri-specta` TypeScript types generated and committed to `src/types/ipc.ts`
- [ ] Zero clippy warnings; zero `cargo audit` vulnerabilities
- [ ] Integration tests use in-memory SQLite (I-10)

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Atomic transaction deadlock under load | Low | High | Short transactions; WAL mode; connection pool max 10; integration test with concurrent calls |
| `tauri-specta` type drift | Medium | High | CI job: `cargo tauri-specta export` → diff check |
| OS notification permission model changes breaking notification API | Low | Medium | `OsNotificationProvider` abstracted; graceful fallback to in-app toast |

---

## Capability 3: frontend-pages

### Capability ID
`frontend-pages`

### Capability Name
Agenda Frontend Pages

### Description
React/TypeScript pages for the Agenda feature: calendar-based `AgendaPage`, predictive `AppointmentModal`, and reactive `KPIPanel`. Built with Vite + Tailwind + TanStack Query + `react-big-calendar` (or custom calendar grid).

### Invariants Preserved
- **I-01**: Existing appointment data preserved — read-only until mutation
- **I-02**: Atomic UI updates — TanStack Query `onMutate` optimistic updates + `onError` rollback
- **I-03**: API compatibility — `src/api/agenda.ts` single IPC boundary
- **I-04**: No component duplication — shared calendar components in `src/components/agenda/`
- **I-06**: TDD — Vitest + RTL for components; Playwright for E2E IPC round-trips
- **I-09**: Read endpoints use existing IPC commands
- **I-11**: Types portable — `src/types/agenda.ts` no React deps

### Input
| Page / Component | User Interactions | Data Sources (IPC) |
|------------------|-------------------|---------------------|
| **AgendaPage** | Month/week/day view; click slot → `AppointmentModal`; drag-resize → `reagendar_appointment`; click appointment → detail popover; filter by patient/status | `list_appointments` (date range), `get_appointment` (detail) |
| **AppointmentModal** | Patient predictive combobox (debounced search); date/time pickers; fee override; notes; save → `create_appointment` / `update_appointment`; state transition buttons (Finalizar, Reagendar, Cancelar) | `search_patients`, `create_appointment`, `update_appointment`, `finalizar_sesion_agenda`, `reagendar_appointment`, `cancelar_appointment` |
| **KPIPanel** | Auto-refresh (30s); click metric → filter AgendaPage | `list_appointments` (filtered `Realizada` + date range) → reactive sum |

### Processing
1. **Routing**: New route `/agenda` in `src/App.tsx`; added to Sidebar nav (icon: `Calendar`)
2. **State Management**: TanStack Query v5
   - `useAppointments(dateRange, filters)` → `list_appointments`
   - `useCreateAppointment()` / `useUpdateAppointment()` / `useFinalizarSesion()` mutations with `onMutate` optimistic update
   - `useKPIs(dateRange)` → derived from `list_appointments` filtered to `Realizada`
3. **Calendar**: `react-big-calendar` with `moment`/`date-fns` localizer; Spanish locale; week starts Monday
   - Views: Month, Week, Day, Agenda (list)
   - Slot click → `AppointmentModal` pre-filled with slot time
   - Event click → popover with actions
   - Drag-resize → `reagendar_appointment` mutation
4. **Predictive Patient Combobox**: `SearchDropdown` + `useDebounce(300ms)` → `search_patients` IPC; shows name + cédula; keyboard navigable
5. **Fee Logic**: Modal shows patient `session_fee_cents` as default; editable override
6. **KPIPanel**: Reactive — subscribes to TanStack Query cache; computes:
   - `Sesiones Realizadas` (count)
   - `Ingresos del Mes` (sum fee_cents of `Realizada` in current month)
   - `Tasa de Asistencia` (Realizada / (Programada + Realizada))
   - `Próximas 24h` (count `Programada` in next 24h)

### Output
| Page / Component | Rendered Elements | Brand Compliance |
|------------------|-------------------|------------------|
| **AgendaPage** | Calendar grid (Sage #E5F1EE header bg), toolbar (view switcher, today, date picker), appointment blocks (Primary #0F4C5C bg, white text), filter sidebar | ✅ Primary buttons, Sage headers, Coral for `Cancelada` blocks |
| **AppointmentModal** | Patient combobox (Sage bg), date/time pickers, fee input (monospace), notes textarea, action buttons (Primary Finalizar, Secondary Reagendar, Coral Cancelar) | ✅ Primary submit, Coral destructive, Sage input focus |
| **KPIPanel** | 4 `MetricCard` in grid (Sage #E5F1EE bg), trend sparklines, click → filters AgendaPage | ✅ Sage bg, Primary text, Primary icon |

### Acceptance Criteria
- [ ] **AgendaPage**: Calendar renders month/week/day/agenda views; Spanish locale; appointments colored by status (Programada=Primary, Realizada=Success, Reagendada=Warning, Cancelada=Coral)
- [ ] **AppointmentModal**: Predictive patient search < 300ms; fee defaults to patient `session_fee_cents`; validation matches Rust (no overlap, duration ≥ 15min)
- [ ] **State Transitions**: Finalizar → calls `finalizar_sesion_agenda` → KPIPanel updates reactively (< 500ms); Reagendar drag-drop works; Cancelar requires reason
- [ ] **KPIPanel**: 4 metrics render with Sage background; reactive — completing session updates "Sesiones Realizadas" + "Ingresos del Mes" without refresh
- [ ] **Navigation**: Sidebar includes "Agenda" (Calendar icon) between "Pacientes" and "Historia Clínica"
- [ ] **Responsive**: Works at 1400x900; calendar stacks on narrow widths
- [ ] **TypeScript**: Zero errors (`tsc --noEmit`); strict mode
- [ ] **Tests**: Vitest component tests > 85%; Playwright E2E for create→finalizar→KPI update flow

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Calendar library bundle size | Medium | Medium | `react-big-calendar` is ~60kb gzipped; code-split with `React.lazy` on `/agenda` route |
| Optimistic update race conditions | Medium | High | TanStack Query `onMutate` + `onError` rollback; server state is source of truth |
| Patient search debounce UX | Low | Medium | 300ms debounce; show spinner; cache results 5min |
| KPI reactivity lag | Low | Medium | Subscribe to query cache; `staleTime: 30_000`; invalidate on mutations |

---

## Capability 4: reminder-service

### Capability ID
`reminder-service`

### Capability Name
Automated Reminder Service with CalendarProvider Abstraction

### Description
Background service that schedules OS-level notifications 30 minutes before appointments, with `CalendarProvider` trait enabling future Google Calendar / Outlook sync without changing domain logic.

### Invariants Preserved
- **I-01**: Reminders fire at `starts_at - 30 minutes` ± 60 seconds
- **I-02**: Idempotent — duplicate schedules for same appointment no-op
- **I-03**: `CalendarProvider` trait is backend-agnostic — OS, Google, Outlook implementations interchangeable
- **I-04**: No reminder for `Cancelada`/`Realizada` appointments
- **I-05**: Graceful degradation — notification failure logs warning, doesn't block appointment
- **I-06**: TDD — unit tests with `MockCalendarProvider`; integration test with `OsNotificationProvider`

### Input
| Component | Configuration | Trigger |
|-----------|---------------|---------|
| **ReminderScheduler** | Tokio interval (1 min tick); `CalendarProvider` impl | `process_due_reminders` IPC or background tick |
| **OsNotificationProvider** | macOS: `osascript -e 'display notification...'`; Windows: `powershell -c "New-BurntToastNotification"`; Linux: `notify-send` | Appointment `starts_at - 30min` within ±1 min window |
| **CalendarProvider Trait** | `schedule_notification(&self, appt: &Appointment) -> Result<String, ReminderError>`; `cancel_notification(&self, external_id: &str) -> Result<(), ReminderError>` | Implemented by OS provider; future: Google/Outlook |

### Processing
1. **Startup**: `ReminderService::spawn(app_state: AppState)` — tokio task, 1-min interval
2. **Tick**: `ReminderDomain::process_due_reminders(now)` → returns `Vec<Appointment>` needing reminder
3. **Schedule**: For each appointment, `calendar_provider.schedule_notification(appt)` → stores `external_id` in `appointment.reminder_external_id`
4. **Fire**: At notification time, OS shows: "MindLedger: Sesión con [Paciente] en 30 min — [Hora]"
5. **Cancel**: On `reagendar_appointment` / `cancelar_appointment` → `calendar_provider.cancel_notification(external_id)`
6. **Completion**: On `finalizar_sesion_agenda` → cancel any pending reminder

### Output
| Artifact | Description |
|----------|-------------|
| **OS Notification** | Native toast/banner at T-30min; title "MindLedger"; body includes patient name, time |
| **Database** | `appointments.reminder_sent = true`, `appointments.reminder_external_id = <provider_id>` |
| **Logs** | `tracing::info!("Reminder sent for appointment {}", id)`; warnings on failure |

### Acceptance Criteria
- [ ] `OsNotificationProvider` sends test notification on macOS (CI), Windows (CI), Linux (local)
- [ ] `ReminderScheduler` ticks every 60s; processes due reminders within ±60s accuracy
- [ ] Idempotency: calling `schedule_reminder` twice for same appointment → single notification
- [ ] State awareness: no reminder for `Cancelada`/`Realizada`; reminder cancelled on reagendar/cancelar
- [ ] `CalendarProvider` trait compiles; `MockCalendarProvider` used in unit tests
- [ ] Background task supervised — restart on panic; logged via `tracing`
- [ ] Integration test: create appointment → wait (mock time) → verify notification sent
- [ ] Zero clippy warnings; `cargo test -p mindledger-reminder` passes

### Risk Factors
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| OS notification permission not granted | Medium | High | First-run permission request; fallback to in-app toast if denied |
| Timezone / DST edge cases | Medium | Medium | All times UTC in DB; `DateTime<Utc>`; display converts to local |
| Background task killed by OS | Low | Medium | Tokio task + `tauri::async_runtime::spawn`; restart on app foreground |
| Google/Outlook OAuth complexity | Low | Medium | Trait abstracts; implement in FASE 4; OS provider works now |

---

## Cross-Cutting Concerns

### Type Synchronization (Rust ↔ TypeScript)
- **Tool**: `tauri-specta` (auto-generated from command signatures)
- **Source of Truth**: Rust command signatures in `src-tauri/commands/src/agenda_commands.rs` + `src/types/agenda.ts`
- **CI Check**: `pnpm typecheck` + `cargo check` in same pipeline; fail on drift

### Testing Strategy
| Layer | Tool | Coverage Target |
|-------|------|-----------------|
| Rust Domain | `#[cfg(test)]` + `proptest` (state machine) | 95%+ |
| Rust Commands | `rusqlite` in-memory + `tauri::test` | 90%+ |
| React Components | Vitest + React Testing Library | 85%+ |
| IPC Round-trips | Playwright (Tauri dev mode) | 100% of commands |
| Reminder Service | Mock time + `MockCalendarProvider` | 90%+ |
| Accessibility | axe-core + Playwright | Zero violations |

### Rollback Triggers
- Any IPC command fails integration tests → revert command module
- Frontend TypeScript errors → feature flag `VITE_FEATURE_AGENDA=false`
- Brand contrast failures → revert `tailwind.config.js` + `src/index.css`
- Reminder service panic loop → disable `ReminderService::spawn` via config flag
- rusqlite compilation failure → revert to FASE 2 state (sqlx)

---

## File Manifest (Spec → Implementation Mapping)

| Spec Capability | Implementation Files |
|-----------------|---------------------|
| `domain-entities` | `src-tauri/domain/src/appointment.rs`, `src-tauri/domain/src/accounting.rs`, `src-tauri/domain/src/reminder.rs`, `src-tauri/domain/src/error.rs`, `src-tauri/domain/src/lib.rs` |
| `ipc-commands` | `src-tauri/commands/src/agenda_commands.rs`, `src-tauri/commands/src/reminder_commands.rs`, `src-tauri/commands/src/error.rs`, `src-tauri/infrastructure/src/repositories.rs` (AppointmentRepository), `src-tauri/app/src/main.rs` (command registration) |
| `frontend-pages` | `src/pages/AgendaPage.tsx`, `src/components/agenda/AppointmentModal.tsx`, `src/components/agenda/KPIPanel.tsx`, `src/components/agenda/CalendarView.tsx`, `src/hooks/useAppointments.ts`, `src/api/agenda.ts`, `src/types/agenda.ts`, `src/App.tsx` (route + sidebar) |
| `reminder-service` | `src-tauri/reminder/src/scheduler.rs`, `src-tauri/reminder/src/providers/os_notification.rs`, `src-tauri/reminder/src/lib.rs`, `src-tauri/app/src/main.rs` (spawn on startup) |

---

## Sign-Off Checklist
- [ ] Spec reviewed by backend lead (Rust domain + commands)
- [ ] Spec reviewed by frontend lead (React/TypeScript + calendar UX)
- [ ] Spec reviewed by designer (brand compliance: Sage KPI, Primary calendar, Coral cancellations)
- [ ] Spec reviewed by QA (testability: state machine, atomic transaction, reminder timing)
- [ ] All invariants traceable to implementation files
- [ ] Risk mitigations assigned owners

(End of file)