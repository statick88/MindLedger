# Tasks: Cognitive Agenda Synchronization with Automated Accounting Triggers

**Feature Branch:** `feat/cognitive-agenda-sync`  
**Spec:** `openspec/changes/cognitive-agenda-sync/spec.md`  
**Design:** `openspec/changes/cognitive-agenda-sync/design.md`  
**Total Tasks:** 34 (6 Phases)

---

## Phase 1: Domain Layer (Rust) — 5 Tasks

### T001 — Appointment Aggregate & State Machine
- **Title:** `appointment.rs` — Appointment aggregate with explicit state transitions
- **Description:** Implement the `Appointment` aggregate root with state machine enforcing valid transitions: `Programada → Realizada | Reagendada | Cancelada`, `Reagendada → Realizada | Cancelada`. Include value objects: `AppointmentId`, `PatientId`, `TherapistId`, `DateTimeRange`, `SessionType`, `Status`. Enforce invariants: no overlap for same therapist, duration 30–120 min, patient exists.
- **Dependencies:** None
- **Effort:** M
- **Files to Create:** `domain/src/appointment.rs`
- **Files to Modify:** `domain/src/lib.rs` (export), `domain/src/error.rs` (add `DomainError::InvalidTransition`, `DomainError::OverlapConflict`, `DomainError::InvalidDuration`)
- **Acceptance Criteria:**
  - `Appointment::new()` validates invariants and returns `Result<Appointment, DomainError>`
  - `appointment.finalize()` transitions `Programada → Realizada`, emits `SessionFinalized` domain event
  - `appointment.reschedule(new_range)` transitions `Programada → Reagendada`, validates no overlap
  - `appointment.cancel(reason)` transitions `Programada|Reagendada → Cancelada`
  - Invalid transitions return `DomainError::InvalidTransition { from, to }`
  - Overlap detection uses `DateTimeRange::overlaps()`
- **Test Requirements:**
  - Unit tests: all valid transitions, all invalid transitions, overlap detection, duration bounds
  - Proptest: random valid appointments never overlap for same therapist

---

### T002 — CalendarProvider Trait & Types
- **Title:** `calendar_provider.rs` — CalendarProvider trait + CalendarEvent + CalendarError
- **Description:** Define the `CalendarProvider` trait for external calendar sync (OS Calendar, Google, Outlook). Define `CalendarEvent` DTO with `id`, `title`, `description`, `start`, `end`, `attendees`, `location`, `recurrence_rule`. Define `CalendarError` enum with variants: `AuthFailed`, `NotFound`, `RateLimited`, `NetworkError`, `QuotaExceeded`, `InvalidEvent`.
- **Dependencies:** None
- **Effort:** S
- **Files to Create:** `domain/src/calendar_provider.rs`
- **Files to Modify:** `domain/src/lib.rs` (export), `domain/src/error.rs` (add `DomainError::CalendarError(CalendarError)`)
- **Acceptance Criteria:**
  - Trait has async methods: `list_events(range) → Result<Vec<CalendarEvent>>`, `create_event(event) → Result<String>`, `update_event(id, event) → Result<()>`, `delete_event(id) → Result<()>`, `sync_token() → Result<String>`
  - `CalendarEvent` serializes to/from iCal/Google Calendar format
  - `CalendarError` implements `std::error::Error`, `From<CalendarError> for DomainError`
- **Test Requirements:** Unit tests for serialization round-trip, error mapping

---

### T003 — Reminder Scheduler & Notifier Traits
- **Title:** `reminder.rs` — ReminderScheduler + ReminderNotifier traits
- **Description:** Define `ReminderScheduler` trait for scheduling/cancelling reminders: `schedule(appointment_id, remind_at) → Result<ReminderId>`, `cancel(reminder_id) → Result<()>`, `process_due() → Vec<DueReminder>`. Define `ReminderNotifier` trait: `notify(reminder) → Result<()>`. Define `Reminder` struct with `id`, `appointment_id`, `patient_id`, `remind_at`, `channel` (Push/Email/SMS), `template_id`.
- **Dependencies:** T001 (AppointmentId, PatientId)
- **Effort:** S
- **Files to Create:** `domain/src/reminder.rs`
- **Files to Modify:** `domain/src/lib.rs` (export), `domain/src/error.rs` (add `DomainError::ReminderError`)
- **Acceptance Criteria:**
  - Traits are `Send + Sync + 'static`
  - `DueReminder` includes `reminder` + `appointment` snapshot for notification context
  - `ReminderChannel` enum: `Push`, `Email`, `Sms`, `InApp`
- **Test Requirements:** Trait compile-test (impl for mock)

---

### T004 — Accounting Trigger Helper
- **Title:** `accounting_trigger.rs` — AccountingTrigger::build_session_asiento() helper
- **Description:** Implement `AccountingTrigger::build_session_asiento(appointment: &Appointment, patient: &Patient, therapist: &Therapist) → Result<Asiento, DomainError>`. Creates double-entry accounting entry (`Asiento`) for a finalized session: Debit `Cuentas por Cobrar` (patient), Credit `Ingresos por Sesiones` (therapist specialty). Includes metadata: `appointment_id`, `session_type`, `therapist_id`, `patient_id`, `amount`, `currency`, `concepto`.
- **Dependencies:** T001 (Appointment), existing `domain/src/accounting.rs` (Asiento, Cuenta, Moneda)
- **Effort:** M
- **Files to Create:** `domain/src/accounting_trigger.rs`
- **Files to Modify:** `domain/src/lib.rs` (export), `domain/src/error.rs` (add `DomainError::AccountingError`)
- **Acceptance Criteria:**
  - Returns balanced `Asiento` (sum debits == sum credits)
  - Uses `Cuenta::from_str("1.1.1.01")` for Cuentas por Cobrar, `Cuenta::from_str("4.1.1.01")` for Ingresos
  - Amount derived from `SessionType::fee()` + therapist specialty modifier
  - Emits `AccountingEntryCreated` domain event
- **Test Requirements:**
  - Unit test: balanced entry for each SessionType
  - Proptest: random valid appointments always produce balanced entries

---

### T005 — Domain Exports & Error Variants
- **Title:** Update `domain/src/lib.rs` + `domain/src/error.rs` — exports + DomainError variants
- **Description:** Re-export all new types from `domain/src/lib.rs`. Add missing `DomainError` variants: `InvalidTransition`, `OverlapConflict`, `InvalidDuration`, `CalendarError`, `ReminderError`, `AccountingError`, `PatientNotFound`, `TherapistNotFound`. Implement `From` conversions for infrastructure errors.
- **Dependencies:** T001–T004
- **Effort:** S
- **Files to Modify:** `domain/src/lib.rs`, `domain/src/error.rs`
- **Acceptance Criteria:**
  - `pub use appointment::*; pub use calendar_provider::*; pub use reminder::*; pub use accounting_trigger::*;`
  - All error variants implement `std::fmt::Display` with user-facing messages
  - `DomainError: From<sqlx::Error>`, `From<reqwest::Error>`, `From<objc2::Error>`
- **Test Requirements:** Error message snapshot tests

---

## Phase 2: Infrastructure Layer (Rust) — 6 Tasks

### T006 — SQLite Appointment Repository
- **Title:** `appointment_repo.rs` — SqliteAppointmentRepository implementing AppointmentRepository trait
- **Description:** Implement `AppointmentRepository` trait with methods: `save(appointment)`, `find_by_id(id)`, `find_by_patient(patient_id, range)`, `find_by_therapist(therapist_id, range)`, `find_overlapping(therapist_id, range)`, `delete(id)`. Use `sqlx` with compile-time checked queries. Migrations in T010. Implement optimistic locking via `version` column.
- **Dependencies:** T001 (AppointmentRepository trait), T010 (migrations)
- **Effort:** L
- **Files to Create:** `infrastructure/src/appointment_repo.rs`
- **Files to Modify:** `infrastructure/src/lib.rs` (export), `infrastructure/Cargo.toml` (sqlx, uuid, chrono features)
- **Acceptance Criteria:**
  - All queries use `sqlx::query_as!` with compile-time verification
  - `find_overlapping` uses `tsrange` overlap operator `&&`
  - `save` uses `INSERT ... ON CONFLICT (id) DO UPDATE SET ... WHERE version = excluded.version - 1`
  - Returns `DomainError::ConcurrencyConflict` on version mismatch
- **Test Requirements:**
  - Integration test with testcontainers/sqlite: CRUD, overlap detection, concurrency conflict
  - Proptest: random appointments round-trip correctly

---

### T007 — macOS EventKit Calendar Provider (Feature: os-calendar)
- **Title:** `os_calendar.rs` — OSCalendarProvider using objc2 + EventKit
- **Description:** Implement `CalendarProvider` for macOS Calendar.app via `objc2` and `EventKit`. Requires `os-calendar` feature flag. Request `EKEntityTypeEvent` authorization. Map `CalendarEvent` ↔ `EKEvent`. Implement `list_events`, `create_event`, `update_event`, `delete_event`, `sync_token` (using `EKEventStore` change token).
- **Dependencies:** T002 (CalendarProvider trait)
- **Effort:** XL
- **Files to Create:** `infrastructure/src/os_calendar.rs`
- **Files to Modify:** `infrastructure/Cargo.toml` (add `objc2`, `objc2-foundation`, `objc2-eventkit` under `os-calendar` feature), `infrastructure/src/lib.rs` (cfg feature export)
- **Acceptance Criteria:**
  - Compiles only with `--features os-calendar`
  - Requests calendar access permission on first use
  - `list_events` filters by calendar identifier (configurable)
  - `sync_token` returns base64-encoded `EKEventStore` token
  - Errors map to `CalendarError` variants
- **Test Requirements:** Integration test on macOS CI runner (requires calendar access grant)

---

### T008 — Google Calendar Provider Stub (Feature: google-calendar)
- **Title:** `google_calendar.rs` — GoogleCalendarProvider stub with OAuth2 flow
- **Description:** Stub implementation of `CalendarProvider` for Google Calendar API. Define `GoogleCalendarConfig` with `client_id`, `client_secret`, `redirect_uri`, `token_storage`. Implement OAuth2 device code flow placeholder. Methods return `CalendarError::NotImplemented` for now. Feature flag `google-calendar`.
- **Dependencies:** T002
- **Effort:** M
- **Files to Create:** `infrastructure/src/google_calendar.rs`
- **Files to Modify:** `infrastructure/Cargo.toml` (add `oauth2`, `google-calendar`, `reqwest` under `google-calendar` feature), `infrastructure/src/lib.rs` (cfg feature export)
- **Acceptance Criteria:**
  - Compiles with `--features google-calendar`
  - `GoogleCalendarProvider::new(config)` validates config
  - All trait methods return `Err(CalendarError::NotImplemented)`
  - Token storage trait defined for future implementation
- **Test Requirements:** Unit test for config validation

---

### T009 — Outlook Calendar Provider Stub (Feature: outlook-calendar)
- **Title:** `outlook_calendar.rs` — OutlookCalendarProvider stub with Microsoft Graph
- **Description:** Stub implementation for Microsoft Graph Calendar API. Define `OutlookCalendarConfig` with `tenant_id`, `client_id`, `client_secret`, `redirect_uri`. Implement client credentials flow placeholder. Methods return `CalendarError::NotImplemented`. Feature flag `outlook-calendar`.
- **Dependencies:** T002
- **Effort:** M
- **Files to Create:** `infrastructure/src/outlook_calendar.rs`
- **Files to Modify:** `infrastructure/Cargo.toml` (add `azure-identity`, `msgraph-sdk` under `outlook-calendar` feature), `infrastructure/src/lib.rs` (cfg feature export)
- **Acceptance Criteria:**
  - Compiles with `--features outlook-calendar`
  - Config validation, trait methods return `NotImplemented`
- **Test Requirements:** Unit test for config validation

---

### T010 — Tokio Reminder Scheduler
- **Title:** `reminder_scheduler.rs` — TokioReminderScheduler with 1-minute tick
- **Description:** Implement `ReminderScheduler` using `tokio::time::interval(1 min)`. On each tick, query repository for reminders where `remind_at <= now()` and `sent_at IS NULL`. For each due reminder, call `ReminderNotifier::notify()`, then mark `sent_at = now()`. Handle shutdown gracefully. Spawn as background task from `commands/src/main.rs`.
- **Dependencies:** T003 (ReminderScheduler trait), T006 (repository for reminders)
- **Effort:** L
- **Files to Create:** `infrastructure/src/reminder_scheduler.rs`
- **Files to Modify:** `infrastructure/src/lib.rs` (export), `infrastructure/Cargo.toml` (tokio, tracing)
- **Acceptance Criteria:**
  - Tick interval configurable via `ReminderSchedulerConfig { tick_interval: Duration }`
  - Processes reminders in batches of 100 to avoid blocking
  - Logs each notification attempt with `tracing::info!` / `tracing::error!`
  - Implements `Shutdown` trait for graceful stop
- **Test Requirements:**
  - Unit test with `tokio::time::pause()` — verify tick fires, reminder processed, marked sent
  - Test: reminder at T-30min fires exactly once

---

### T011 — SQL Migrations for Agenda Extensions
- **Title:** `migrations_agenda.sql` — SQL migrations for appointments table + reminders
- **Description:** Create SQL migration files for: `appointments` table extensions (add `version`, `synced_at`, `calendar_event_id`, `external_calendar_id`), `reminders` table (id, appointment_id, patient_id, remind_at, sent_at, channel, template_id), `calendar_sync_tokens` table (provider, calendar_id, token, updated_at). Include indexes for overlap queries (`tsrange`), reminder due queries, patient/therapist lookups.
- **Dependencies:** T006 (repository expects these columns)
- **Effort:** M
- **Files to Create:** `infrastructure/migrations/20250115_001_agenda_extensions.sql`
- **Files to Modify:** `infrastructure/Cargo.toml` (sqlx-cli for migration embedding)
- **Acceptance Criteria:**
  - Migration runs cleanly on empty SQLite DB
  - `appointments` table has `EXCLUDE USING gist (therapist_id WITH =, time_range WITH &&)` equivalent via trigger (SQLite uses triggers for exclusion)
  - `reminders` table has index on `(remind_at, sent_at)` for due query
  - Foreign keys to `patients`, `therapists` tables
- **Test Requirements:** Migration test in integration suite (T031)

---

## Phase 3: Commands Layer (Rust) — 5 Tasks

### T012 — Agenda Commands (7 commands)
- **Title:** `agenda_commands.rs` — 7 Tauri commands for appointment CRUD + state transitions
- **Description:** Implement Tauri commands (using `tauri::command`): `create_appointment`, `get_appointment`, `list_appointments(filters)`, `update_appointment`, `delete_appointment`, `finalize_appointment`, `reschedule_appointment`, `cancel_appointment`. Each command: validates input → loads aggregate → executes → saves → emits domain events → returns DTO. Use `AppointmentRepository` from infrastructure. Wrap in `CommandError` mapping from `DomainError`.
- **Dependencies:** T001, T005, T006
- **Effort:** XL
- **Files to Create:** `commands/src/agenda_commands.rs`
- **Files to Modify:** `commands/src/lib.rs` (export), `commands/src/error.rs` (add `CommandError::Domain(DomainError)`), `commands/src/main.rs` (register commands)
- **Acceptance Criteria:**
  - `create_appointment` validates patient/therapist exist, no overlap, duration 30–120 min
  - `finalize_appointment` calls `AccountingTrigger::build_session_asiento()` and saves asiento via existing accounting command
  - `reschedule_appointment` validates new slot availability
  - `cancel_appointment` requires reason, emits `AppointmentCancelled`
  - All commands return `Result<AppointmentDto, CommandError>`
  - Input DTOs use `serde::Deserialize`, output DTOs use `serde::Serialize`
- **Test Requirements:** Integration tests (T031) cover all 7 commands with in-memory SQLite

---

### T013 — Calendar Commands (2 commands)
- **Title:** `calendar_commands.rs` — sync_calendar, get_calendar_events
- **Description:** Implement `sync_calendar(provider: String, calendar_id: Option<String>)` → pulls events from provider, upserts appointments for matching external IDs, updates `synced_at`. Implement `get_calendar_events(range: DateRange, provider: Option<String>)` → returns merged view of local + external events. Use `CalendarProvider` trait dynamically via feature flags.
- **Dependencies:** T002, T007–T009, T006
- **Effort:** L
- **Files to Create:** `commands/src/calendar_commands.rs`
- **Files to Modify:** `commands/src/lib.rs`, `commands/src/main.rs`
- **Acceptance Criteria:**
  - `sync_calendar` returns `SyncResult { created, updated, deleted, conflicts }`
  - Conflict detection: same external_id, different local data → returns conflict for manual resolution
  - `get_calendar_events` merges without duplication (by external_id)
  - Provider selection via feature flag at compile time
- **Test Requirements:** Mock provider test for sync logic

---

### T014 — Reminder Commands (2 commands)
- **Title:** `reminder_commands.rs` — schedule_reminder, process_due_reminders
- **Description:** Implement `schedule_reminder(appointment_id, remind_at, channel, template_id)` → creates reminder record, calls `ReminderScheduler::schedule()`. Implement `process_due_reminders()` → manual trigger for scheduler tick (useful for testing). Both return `Result<ReminderDto, CommandError>`.
- **Dependencies:** T003, T010
- **Effort:** M
- **Files to Create:** `commands/src/reminder_commands.rs`
- **Files to Modify:** `commands/src/lib.rs`, `commands/src/main.rs`
- **Acceptance Criteria:**
  - `schedule_reminder` validates `remind_at > now()`, `channel` valid
  - Reminder linked to appointment + patient
  - `process_due_reminders` returns count of processed reminders
- **Test Requirements:** Integration test with mocked notifier

---

### T015 — KPI Commands (1 command)
- **Title:** `kpi_commands.rs` — get_agenda_kpis (reactive metrics)
- **Description:** Implement `get_agenda_kpis(range: DateRange, therapist_id: Option<TherapistId>)` → returns `AgendaKpis { sessions_scheduled, sessions_completed, sessions_cancelled, occupancy_rate, avg_session_duration, revenue_projected, revenue_realized, no_show_rate }`. Computed reactively from appointment data (no materialized view). Uses SQL aggregations.
- **Dependencies:** T006
- **Effort:** M
- **Files to Create:** `commands/src/kpi_commands.rs`
- **Files to Modify:** `commands/src/lib.rs`, `commands/src/main.rs`
- **Acceptance Criteria:**
  - Single query with conditional aggregates
  - `occupancy_rate = completed_slots / total_slots_in_range`
  - `revenue_projected` = sum of fees for `Programada` + `Reagendada`
  - `revenue_realized` = sum of fees for `Realizada`
  - Returns in < 100ms for 10k appointments
- **Test Requirements:** Integration test with known dataset, verify calculations

---

### T016 — Commands Wiring: error.rs, lib.rs, main.rs
- **Title:** Update `commands/src/error.rs` + `lib.rs` + `main.rs` — register commands + spawn scheduler
- **Description:** Extend `CommandError` with `Reminder`, `Calendar`, `Kpi` variants. In `lib.rs`, re-export all command modules. In `main.rs`: initialize `TokioReminderScheduler` with config, spawn as background task, register all 12 Tauri commands (`tauri::generate_handler![]`), setup graceful shutdown on `Ctrl-C`.
- **Dependencies:** T012–T015, T010
- **Effort:** M
- **Files to Modify:** `commands/src/error.rs`, `commands/src/lib.rs`, `commands/src/main.rs`
- **Acceptance Criteria:**
  - `cargo build --features os-calendar` compiles all commands
  - Scheduler starts on `main()` and shuts down on signal
  - All 12 commands appear in Tauri invoke list
  - Error mapping covers all `DomainError` variants
- **Test Requirements:** Smoke test: `tauri dev` starts without panic, commands invokable

---

## Phase 4: Frontend Types & API — 4 Tasks

### T017 — TypeScript Appointment Types
- **Title:** `src/types/appointment.ts` — TypeScript interfaces matching Rust DTOs
- **Description:** Define TS interfaces for all appointment-related DTOs: `AppointmentDto`, `CreateAppointmentDto`, `UpdateAppointmentDto`, `AppointmentFilters`, `AppointmentStatus` (enum: 'scheduled' | 'completed' | 'rescheduled' | 'cancelled'), `SessionType`, `CalendarEventDto`, `ReminderDto`, `AgendaKpisDto`, `SyncResultDto`. Use `zod` schemas for runtime validation. Export `appointmentSchema`, `createAppointmentSchema`, etc.
- **Dependencies:** T012–T015 (Rust DTOs as source of truth)
- **Effort:** M
- **Files to Create:** `src/types/appointment.ts`, `src/types/index.ts` (re-export)
- **Files to Modify:** `package.json` (add zod if missing)
- **Acceptance Criteria:**
  - Interfaces match Rust `serde` output exactly (field names, types)
  - `zod` schemas validate all required fields
  - `AppointmentStatus` maps 1:1 to Rust `Status` enum
  - `DateTime` fields as ISO 8601 strings
- **Test Requirements:** Type test: `z.infer<typeof appointmentSchema>` matches interface

---

### T018 — Agenda API Wrapper
- **Title:** `src/api/agendaApi.ts` — invoke() wrappers for all 12 commands
- **Description:** Create typed API functions using `@tauri-apps/api/core.invoke`: `createAppointment`, `getAppointment`, `listAppointments`, `updateAppointment`, `deleteAppointment`, `finalizeAppointment`, `rescheduleAppointment`, `cancelAppointment`, `syncCalendar`, `getCalendarEvents`, `scheduleReminder`, `processDueReminders`, `getAgendaKpis`. Each wraps `invoke('command_name', args)` with zod validation of response. Export as `agendaApi` object.
- **Dependencies:** T017, T016 (commands registered)
- **Effort:** M
- **Files to Create:** `src/api/agendaApi.ts`, `src/api/index.ts` (re-export)
- **Acceptance Criteria:**
  - All 12 functions typed with input/output zod schemas
  - Errors caught and re-thrown as `ApiError` with `code`, `message`
  - `listAppointments` supports pagination, filters (patient, therapist, status, date_range)
  - `getAgendaKpis` returns typed `AgendaKpisDto`
- **Test Requirements:** Unit test with MSW mocking `invoke`

---

### T019 — useAgenda TanStack Query Hooks
- **Title:** `src/hooks/useAgenda.ts` — TanStack Query hooks with cache invalidation
- **Description:** Implement hooks: `useAppointments(filters)` → `useQuery` with `queryKey: ['appointments', filters]`, `useAppointment(id)` → `useQuery`, `useAppointmentMutations()` → `useMutation` for create/update/delete/finalize/reschedule/cancel with `onSuccess` invalidating `['appointments']`. `useCalendarSync()` → mutation for sync. `useAgendaKpis(range, therapistId)` → query with 30s staleTime. `useReminders(appointmentId)` → query.
- **Dependencies:** T018, `@tanstack/react-query` installed
- **Effort:** L
- **Files to Create:** `src/hooks/useAgenda.ts`, `src/hooks/index.ts` (re-export)
- **Files to Modify:** `src/providers/QueryProvider.tsx` (ensure queryClient configured)
- **Acceptance Criteria:**
  - Mutations invalidate correct query keys
  - `useAppointments` supports infinite scroll via `useInfiniteQuery`
  - Optimistic updates for status transitions (finalize/cancel/reschedule)
  - `useAgendaKpis` refetches on appointment mutations
- **Test Requirements:** RTL test with mocked queryClient (T032)

---

### T020 — usePatientSearch Hook
- **Title:** `src/hooks/usePatientSearch.ts` — debounced patient search hook
- **Description:** Implement `usePatientSearch(query: string, enabled: boolean)` → returns `{ patients, isLoading, error }`. Debounces query 300ms. Uses `patientApi.searchPatients(query)` (existing API). Minimum 2 chars to trigger. Caches results 5min. Returns empty array if query < 2 chars.
- **Dependencies:** Existing patient API
- **Effort:** S
- **Files to Create:** `src/hooks/usePatientSearch.ts`
- **Acceptance Criteria:**
  - Debounce verified: rapid keystrokes → single API call
  - Cancels in-flight request on new query
  - Returns cached results for repeat queries
- **Test Requirements:** RTL test with fake timers

---

## Phase 5: Frontend Components & Pages — 10 Tasks

### T021 — SearchDropdown Component
- **Title:** `SearchDropdown.tsx` — reusable native select with debounced filter
- **Description:** Build accessible `<select>` wrapper with: search input (filters options client-side), keyboard navigation (ArrowUp/Down, Enter, Escape), `option` groups support, `placeholder`, `value`, `onChange`, `disabled`, `loading` states. Uses `react-select`-like UX but native `<select>` for accessibility. Styled with Tailwind + brand tokens.
- **Dependencies:** T028 (brand tokens), T017 (Patient type)
- **Effort:** M
- **Files to Create:** `src/components/agenda/SearchDropdown.tsx`, `src/components/agenda/index.ts`
- **Acceptance Criteria:**
  - Passes axe-core accessibility audit
  - Filters 500+ options in < 16ms (virtualized if needed)
  - Works with controlled + uncontrolled modes
  - Supports `option` groups (optgroup)
- **Test Requirements:** RTL test: search, select, keyboard nav, accessibility

---

### T022 — CalendarView Component
- **Title:** `CalendarView.tsx` — react-big-calendar wrapper with Spanish locale
- **Description:** Wrap `react-big-calendar` with: Spanish locale (`es-ES`), week starts Monday, business hours 08:00–20:00, time slots 30min, `views: ['day', 'week', 'month', 'agenda']`, `min`/`max` date navigation bounds. Props: `events: AppointmentDto[]`, `onEventClick`, `onSlotSelect`, `selectedDate`, `therapistId` filter. Custom toolbar with view switcher, today button, date picker.
- **Dependencies:** T017 (AppointmentDto), T023 (EventComponent), T028 (status colors)
- **Effort:** L
- **Files to Create:** `src/components/agenda/CalendarView.tsx`
- **Acceptance Criteria:**
  - Renders 1000 events without jank (virtualized)
  - Spanish month/day names correct
  - Time format HH:mm (24h)
  - Click event → calls `onEventClick(appointment)`
  - Slot select → calls `onSlotSelect(start, end)`
- **Test Requirements:** RTL test: render, view switch, event click, slot select

---

### T023 — EventComponent (Custom Event Rendering)
- **Title:** `EventComponent.tsx` — custom event rendering with status colors
- **Description:** Custom `eventComponent` for react-big-calendar. Renders appointment block with: colored left border by status (Programada=Azul #1A73E8, Realizada=Verde Sage #2E7D32, Reagendada=Ámbar #F57F17, Cancelada=Rojo #C62828), patient name, session type icon, therapist initials, time range. Tooltip on hover with full details. Drag/resize disabled (handled via modal).
- **Dependencies:** T022, T028 (color tokens)
- **Effort:** M
- **Files to Create:** `src/components/agenda/EventComponent.tsx`
- **Acceptance Criteria:**
  - Colors match design tokens exactly
  - Truncates long patient names with ellipsis
  - Shows "🔄" icon for Reagendada, "✓" for Realizada, "✕" for Cancelada
  - Accessible: `role="button"`, `tabIndex=0`, `aria-label` with full info
- **Test Requirements:** RTL test: render per status, tooltip, accessibility

---

### T024 — SessionModal (Appointment Form)
- **Title:** `SessionModal.tsx` — appointment form with predictive patient combobox
- **Description:** Modal form (using `radix-ui` Dialog) for create/edit/finalize/reschedule/cancel. Fields: Patient (SearchDropdown from T021, predictive search via `usePatientSearch`), Therapist (select), SessionType (select), Date (date picker), Start Time (time picker), Duration (select 30/45/60/90/120), Notes (textarea). Validation: required fields, no overlap (client-side check via `listAppointments`), duration bounds. Submit calls appropriate command. Cancel reason required for cancel action.
- **Dependencies:** T021, T019 (hooks), T017 (types), T028 (tokens)
- **Effort:** XL
- **Files to Create:** `src/components/agenda/SessionModal.tsx`
- **Acceptance Criteria:**
  - Patient search debounced, shows top 10 matches
  - Overlap warning shown before submit (non-blocking)
  - Finalize action shows confirmation with revenue preview
  - Reschedule shows current vs new time comparison
  - Cancel requires reason (min 10 chars)
  - Form resets on close
- **Test Requirements:** RTL test: create, edit, finalize, reschedule, cancel flows

---

### T025 — KPIPanel Component
- **Title:** `KPIPanel.tsx` — Sage Green (#E5F1EE) reactive metrics (4 MetricCards)
- **Description:** Grid of 4 `MetricCard` components showing: Sessions Completed (today/week/month), Occupancy Rate (%), Revenue Realized, No-Show Rate. Background `#E5F1EE`, text `#1B5E20`, accent `#2E7D32`. Uses `useAgendaKpis` hook. Cards animate count-up on mount. Responsive: 1 col mobile, 2 tablet, 4 desktop. Refresh button triggers refetch.
- **Dependencies:** T019, T028 (color tokens)
- **Effort:** M
- **Files to Create:** `src/components/agenda/KPIPanel.tsx`, `src/components/agenda/MetricCard.tsx`
- **Acceptance Criteria:**
  - Colors exactly `#E5F1EE` bg, `#1B5E20` text, `#2E7D32` accent
  - Count-up animation 800ms ease-out
  - Updates reactively when appointments change (via query invalidation)
  - Loading skeleton while fetching
- **Test Requirements:** RTL test: render, loading, data, animation, responsive

---

### T026 — AgendaPage (Main Page, Code-Split)
- **Title:** `AgendaPage.tsx` — main page (code-split via React.lazy)
- **Description:** Page component combining: `KPIPanel` (top), `CalendarView` (center, 70% width), `SessionModal` (overlay). State: `selectedDate`, `view`, `modalMode`. Lazy-loaded via `React.lazy(() => import('./AgendaPage'))` with `Suspense` fallback. Feature flag check: if `import.meta.env.VITE_FEATURE_AGENDA !== 'true'` → render "Coming Soon".
- **Dependencies:** T022, T024, T025, T027 (feature flag)
- **Effort:** L
- **Files to Create:** `src/pages/AgendaPage.tsx`, `src/pages/index.ts` (export)
- **Acceptance Criteria:**
  - Code-split: separate chunk `agenda-[hash].js`
  - Loads only when navigating to `/agenda`
  - Feature flag gates entire page
  - Responsive layout: sidebar collapsible on mobile
- **Test Requirements:** E2E test (T033) covers page load

---

### T027 — Layout & App Routing Updates
- **Title:** Update `Layout.tsx` — Agenda nav item + Update `App.tsx` — /agenda route + feature flag
- **Description:** In `Layout.tsx`: add "Agenda" nav item with Calendar icon (`lucide-react`), conditional render based on feature flag. In `App.tsx`: add `/agenda` route with `React.lazy` import, wrap in `Suspense`. Define `VITE_FEATURE_AGENDA` env var (default false). Update `vite.config.ts` to define env var.
- **Dependencies:** T026
- **Effort:** M
- **Files to Modify:** `src/components/Layout.tsx`, `src/App.tsx`, `vite.config.ts`, `.env.example`
- **Acceptance Criteria:**
  - Nav item appears only when `VITE_FEATURE_AGENDA=true`
  - Route `/agenda` loads lazy chunk
  - 404 for `/agenda` when feature flag off
  - Icon: `Calendar` from `lucide-react`
- **Test Requirements:** E2E test: feature flag on/off

---

### T028 — Tailwind Config & CSS Brand Tokens
- **Title:** Update `tailwind.config.js` + `src/index.css` — brand tokens for status colors
- **Description:** Extend Tailwind theme with: `colors.brand.sage = { 50: '#E5F1EE', 100: '#C8E0D0', ..., 900: '#1B5E20' }`, `colors.status.scheduled = '#1A73E8'`, `colors.status.completed = '#2E7D32'`, `colors.status.rescheduled = '#F57F17'`, `colors.status.cancelled = '#C62828'`. In `index.css`: `@layer utilities { .bg-status-scheduled { @apply bg-status-scheduled; } ... }`. Add CSS variables for Sage Green palette.
- **Dependencies:** None (design system)
- **Effort:** M
- **Files to Modify:** `tailwind.config.js`, `src/index.css`
- **Acceptance Criteria:**
  - `bg-brand-sage-50` → `#E5F1EE`
  - `text-status-completed` → `#2E7D32`
  - `border-status-rescheduled` → `#F57F17`
  - All status colors accessible (WCAG AA on white)
- **Test Requirements:** Visual regression test (chromatic) or manual check

---

### T029 — Vite Code-Split Configuration
- **Title:** Update `vite.config.ts` — code-split configuration
- **Description:** Configure `build.rollupOptions.output.manualChunks` to split: `agenda` chunk (AgendaPage + components), `vendor` chunk (react-big-calendar, tanstack-query, zod), `ui` chunk (radix-ui, lucide-react). Set `build.chunkSizeWarningLimit: 1000`. Ensure `esbuild` target supports optional chaining.
- **Dependencies:** T026
- **Effort:** S
- **Files to Modify:** `vite.config.ts`
- **Acceptance Criteria:**
  - `pnpm build` produces `agenda-[hash].js` < 200KB gzipped
  - No duplicate modules across chunks
  - `react-big-calendar` not in main chunk
- **Test Requirements:** Build analysis via `vite-bundle-analyzer`

---

## Phase 6: Testing & Quality — 5 Tasks

### T030 — Domain Unit Tests
- **Title:** Domain unit tests: state machine, accounting trigger, reminder logic (proptest)
- **Description:** Comprehensive unit tests in `domain/tests/`: `appointment_state_machine.rs` (all transitions, invariants), `accounting_trigger.rs` (balanced entries, all session types), `reminder_logic.rs` (scheduling, deduplication). Use `proptest` for property-based tests: random valid appointments never overlap, accounting entries always balance, reminders fire once.
- **Dependencies:** T001–T005
- **Effort:** L
- **Files to Create:** `domain/tests/appointment_state_machine.rs`, `domain/tests/accounting_trigger.rs`, `domain/tests/reminder_logic.rs`
- **Acceptance Criteria:**
  - 100% coverage of domain logic (branches)
  - Proptest: 10,000 iterations pass
  - No flaky tests
- **Test Requirements:** `cargo test -p domain -- --test-threads=4`

---

### T031 — Command Integration Tests
- **Title:** Command integration tests: in-memory SQLite, atomic transaction verification
- **Description:** Integration tests in `commands/tests/`: `agenda_commands_test.rs` (all 7 commands), `calendar_commands_test.rs` (sync, list), `reminder_commands_test.rs` (schedule, process), `kpi_commands_test.rs` (metrics). Use `sqlx::SqlitePool::connect(":memory:")` with migrations. Verify: atomicity (rollback on error), concurrency (optimistic locking), domain events emitted.
- **Dependencies:** T006, T010, T012–T016
- **Effort:** XL
- **Files to Create:** `commands/tests/agenda_commands_test.rs`, `commands/tests/calendar_commands_test.rs`, `commands/tests/reminder_commands_test.rs`, `commands/tests/kpi_commands_test.rs`, `commands/tests/helpers.rs`
- **Acceptance Criteria:**
  - All 12 commands tested with happy + error paths
  - Concurrency test: 10 parallel reschedules → max 1 succeeds
  - Transaction test: finalize fails accounting → appointment not finalized
  - KPI test: known dataset → exact expected values
- **Test Requirements:** `cargo test -p commands -- --test-threads=1`

---

### T032 — Frontend Component Tests
- **Title:** Frontend component tests: Vitest + RTL (SearchDropdown, SessionModal, KPIPanel)
- **Description:** Component tests in `src/components/agenda/__tests__/`: `SearchDropdown.test.tsx` (search, select, keyboard, a11y), `SessionModal.test.tsx` (create, edit, finalize, reschedule, cancel flows), `KPIPanel.test.tsx` (render, loading, data, animation). Use `@testing-library/react`, `@testing-library/user-event`, `vitest`, `msw` for API mocking. Test with `QueryClientProvider` wrapper.
- **Dependencies:** T019, T021, T024, T025
- **Effort:** L
- **Files to Create:** `src/components/agenda/__tests__/SearchDropdown.test.tsx`, `src/components/agenda/__tests__/SessionModal.test.tsx`, `src/components/agenda/__tests__/KPIPanel.test.tsx`, `vitest.setup.ts` (if missing)
- **Acceptance Criteria:**
  - All user interactions tested
  - Accessibility assertions (axe-core)
  - Async waits for debounced search
  - Animation tested via `waitFor` + snapshot
- **Test Requirements:** `pnpm test -- --run`

---

### T033 — E2E Tests (Playwright)
- **Title:** E2E tests: Playwright — create→finalize→KPI update flow
- **Description:** Playwright tests in `e2e/agenda.spec.ts`: (1) Navigate to `/agenda`, verify KPI panel loads. (2) Click slot → create appointment via modal → verify appears on calendar. (3) Click appointment → finalize → verify status "Realizada", accounting entry created (check via API). (4) Verify KPI panel updates: sessions_completed +1, revenue_realized updated. (5) Reschedule flow. (6) Cancel flow. Run against `tauri dev` or built app.
- **Dependencies:** T016, T026, T027
- **Effort:** XL
- **Files to Create:** `e2e/agenda.spec.ts`, `e2e/fixtures/tauri.ts`, `playwright.config.ts` (if missing)
- **Acceptance Criteria:**
  - Full happy path passes in < 60s
  - Tests run in CI (GitHub Actions) with Tauri driver
  - Screenshots on failure
  - Feature flag tested: off → page hidden
- **Test Requirements:** `npx playwright test e2e/agenda.spec.ts`

---

### T034 — Reminder Scheduler Tests
- **Title:** Reminder scheduler tests: mock time, verify notification sent at T-30min
- **Description:** Unit tests in `infrastructure/tests/reminder_scheduler_test.rs`: use `tokio::time::pause()` to control time. Create reminder at `now + 30min`. Advance time 29min → not sent. Advance 1min → sent exactly once. Verify `ReminderNotifier::notify` called with correct reminder + appointment snapshot. Test batch processing (100 reminders due same tick). Test shutdown drains in-flight.
- **Dependencies:** T010
- **Effort:** L
- **Files to Create:** `infrastructure/tests/reminder_scheduler_test.rs`
- **Acceptance Criteria:**
  - Time-mocked tests deterministic
  - Notification called exactly once per reminder
  - Batch processing doesn't block tick
  - Shutdown waits for in-flight notifications
- **Test Requirements:** `cargo test -p infrastructure reminder_scheduler -- --test-threads=1`

---

## Summary

| Phase | Tasks | Total Effort |
|-------|-------|--------------|
| 1: Domain Layer | 5 | L |
| 2: Infrastructure | 6 | XL |
| 3: Commands | 5 | XL |
| 4: Frontend Types/API | 4 | L |
| 5: Frontend Components | 10 | XL |
| 6: Testing | 5 | XL |
| **Total** | **34** | **~6 XL + 4 L + 4 M + 2 S** |

## Execution Order

```
T001 → T002 → T003 → T004 → T005
    ↓
T010 ← T006 ← T011
    ↓     ↓
T007, T008, T009 (parallel, feature-gated)
    ↓
T012 → T013 → T014 → T015 → T016
    ↓
T017 → T018 → T019 → T020
    ↓
T021 → T022 → T023 → T024 → T025 → T026 → T027 → T028 → T029
    ↓
T030, T031, T032, T033, T034 (parallel)
```

## Feature Flags

| Feature | Cargo Feature | Vite Env | Components |
|---------|---------------|----------|------------|
| macOS Calendar | `os-calendar` | — | OSCalendarProvider |
| Google Calendar | `google-calendar` | — | GoogleCalendarProvider |
| Outlook Calendar | `outlook-calendar` | — | OutlookCalendarProvider |
| Agenda UI | — | `VITE_FEATURE_AGENDA` | AgendaPage, nav item |

## Definition of Done

- [ ] All 34 tasks completed
- [ ] `cargo test --workspace` passes
- [ ] `pnpm test` passes
- [ ] `npx playwright test` passes
- [ ] `cargo build --release --features os-calendar` succeeds
- [ ] `pnpm build` produces agenda chunk < 200KB gzipped
- [ ] Accessibility audit: axe-core 0 violations on AgendaPage
- [ ] Feature flag `VITE_FEATURE_AGENDA=false` hides Agenda completely
- [ ] Documentation updated: `README.md` (feature flags), `CHANGELOG.md`