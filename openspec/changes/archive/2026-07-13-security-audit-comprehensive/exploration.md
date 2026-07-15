# Security Audit Exploration: MindLdger

## Exploration: Comprehensive Security Audit

### Current State

MindLdger is a Tauri 2.0 desktop application for clinical psychology practice management in Ecuador. It stores encrypted patient records, accounting data, CIE-10/DSM-5 diagnostics, and clinical notes using SQLCipher. The architecture follows hexagonal principles with clear separation: domain → application → infrastructure → commands (IPC).

**Security Architecture Map:**

```
┌─────────────────────────────────────────────────────────────┐
│                    IPC BRIDGE (27 commands)                 │
│  patient_commands (7) | accounting (5) | diagnostics (8)   │
│  age (3) | agenda (10) | tenant (1)                        │
├─────────────────────────────────────────────────────────────┤
│                 APPLICATION LAYER                           │
│  docx_parser (DOCX clinical notes)                         │
├─────────────────────────────────────────────────────────────┤
│                 DOMAIN LAYER                                │
│  value_objects | accounting | appointment | identifiers     │
│  accounting_trigger | patient | diagnostics                 │
├─────────────────────────────────────────────────────────────┤
│                 INFRASTRUCTURE LAYER                        │
│  database.rs (SQLCipher pool) | keyring.rs (key lifecycle) │
│  migrations.rs | repositories (SQLite)                     │
├─────────────────────────────────────────────────────────────┤
│                 STORAGE LAYER                               │
│  SQLCipher (encrypted SQLite) + keyring/file key storage   │
└─────────────────────────────────────────────────────────────┘
```

### Affected Areas — IPC Bridge (27 Tauri Commands)

| Module | Commands | Parameters | Validation |
|--------|----------|------------|------------|
| `patient_commands.rs` | `create_patient`, `get_patient`, `list_patients`, `update_patient`, `delete_patient`, `search_patients`, `get_patient_count` | PatientId (String→UUID), CreatePatientRequest (document_number, names, email, phone, address, medical data) | Domain value objects validate: DocumentNumber, FullName, Email, PhoneNumber, Address; UUID parsing; date parsing |
| `accounting_commands.rs` | `add_asiento`, `remove_asiento`, `list_asientos`, `generate_balance_general`, `generate_estado_resultados` | CreateAsientoRequest (fecha, descripcion, lineas with debito/credito), ListAsientosQuery | parse_date, parse_decimal, validate_linea_request; negative amount check (monto <= 0); balance validation in domain |
| `diagnostics_commands.rs` | `search_cie10`, `search_dsm5`, `get_cie10_by_codigo`, `get_dsm5_by_codigo`, `create_mapeo`, `list_mapeos`, `update_mapeo`, `delete_mapeo` | paciente_id (UUID), diagnostico_id, fuente, notas | UUID parsing, limit capping (min(100)), date parsing |
| `age_commands.rs` | `calculate_age`, `calculate_age_at`, `calculate_age_breakdown` | birth_date, at_date (String→NaiveDate) | Date format validation |
| `agenda_commands.rs` | `crear_cita_agenda`, `obtener_cita_agenda`, `listar_citas_agenda`, `finalizar_sesion_agenda`, `reagendar_cita`, `cancelar_cita`, `obtener_citas_paciente`, `obtener_recordatorios_pendientes`, `procesar_recordatorios_pendientes`, `obtener_kpis_agenda` | AppointmentId, PatientId, TherapistId (all UUID), CreateAppointmentRequest (start_at, end_at, modality, fee_cents, notes) | RFC 3339 parsing, UUID validation, DateTimeRange validation (15-120 min), overlap detection, state machine transitions, fee_cents >= 0 |
| `tenant.rs` | `get_tenant_config` | None | Compile-time embedded JSON config |

**Key Security Observations — IPC:**
- All UUID parameters are parsed via `Uuid::parse_str()` with proper error handling
- Domain value objects enforce validation (empty checks, length limits, format validation)
- Accounting: negative amounts rejected at domain layer (`monto <= Decimal::ZERO`)
- Accounting: balance enforced at domain layer (debit = credit within 0.01 epsilon)
- Pagination capped at 100 items per page
- Search limit capped at 100 results
- **CSP configured** in tauri.conf.json: `default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'`
- **No Tauri allowlist configured** — default Tauri 2.0 security (no filesystem/shell/HTTP access from frontend)

### Affected Areas — Cryptography & Secrets

**Key Lifecycle:**
```
derivation: 32-byte random key (rand::thread_rng) → hex-encoded (64 chars)
storage:    keyring (primary) → file fallback (mind-ledger.key with 0o600)
usage:      PRAGMA key = '{hex_key}' → SQLCipher encrypts/decrypts
zeroization: Zeroizing<Vec<u8>> + Zeroizing<String> during generation
```

**Files:**
- `src-tauri/infrastructure/src/keyring.rs` — SqlCipherKeyManager
- `src-tauri/infrastructure/src/database.rs` — create_pool_for_tenant, create_pool_with_key

**Key Security Observations — Crypto:**
1. **Key derivation**: 32-byte random key via `rand::thread_rng()` — NOT cryptographically strong CSPRNG (uses OS entropy but not explicitly `OsRng`)
2. **Key storage**: keyring (OS credential store) with file fallback at `0o600`
3. **Key zeroization**: `Zeroizing<Vec<u8>>` used during generation, but key is returned as plain `String` — no zeroization after use
4. **PRAGMA key injection**: `format!("PRAGMA key = '{}';", key)` — key is single-quoted; potential SQL injection if key contains `'` (unlikely with hex encoding but not enforced)
5. **No key rotation mechanism** — once created, key is permanent
6. **No key derivation function** (argon2 listed in Cargo.toml but NOT used for key derivation — only random generation)
7. **File fallback**: key written to `mind-ledger.key` in app data dir; `0o600` permissions set on Unix only
8. **Tenant isolation**: different keyring account + DB filename per tenant

### Affected Areas — Business Logic Invariants

**Accounting Equation (debit = credit):**
- `AsientoContable::new()` validates: lines not empty, each line has debit XOR credit, amounts > 0, total debits == total credits (within 0.01 epsilon)
- `validar_balance_general()` validates: Activos = Pasivos + Patrimonio
- `AccountingTrigger::validate_asiento_balance()` double-checks after generation

**Negative Amount Validation:**
- `LineaAsiento::new_debito/credito`: rejects `monto <= Decimal::ZERO`
- `accounting_commands.rs`: rejects `monto <= Decimal::ZERO` before domain call
- `Appointment::new()`: rejects `fee_cents < 0`

**Transaction State Machine:**
- `AppointmentStatus`: Programada → {Realizada, Reagendada, Cancelada}; Reagendada → {Realizada, Cancelada}; Realizada/Cancelada are terminal
- `can_transition_to()` enforced at domain level
- Cancellation requires non-empty reason

### Dependency Inventory (Security-Critical)

**Rust (Cargo.lock):**

| Crate | Version | Purpose | Notes |
|-------|---------|---------|-------|
| `tokio` | 1.52.3 | Async runtime | Well-maintained, no known vulns |
| `rusqlite` | 0.31.0 | SQLite bindings | With `bundled-sqlcipher` feature |
| `keyring` | 3.6.3 | OS credential store | Depends on `zeroize` |
| `zeroize` | 1.9.0 | Memory zeroization | Used in key generation |
| `argon2` | 0.5 (spec) | Password hashing | **NOT used in code** — listed in Cargo.toml but no imports found |
| `docx-rs` | 0.4.20 | DOCX parsing | Parses untrusted input |
| `rand` | 0.8.x | RNG | Uses `thread_rng()` (ChaCha) |

**TypeScript (pnpm-lock.yaml):**

| Package | Version | Purpose |
|---------|---------|---------|
| `@tauri-apps/api` | 2.11.1 | Tauri IPC |
| `@tanstack/react-query` | 5.101.2 | Server state |
| `zustand` | 5.0.5 | Client state |
| `react` | 18.3.1 | UI |
| `react-router-dom` | 7.6.1 | Routing |

### Identified Attack Surfaces

1. **DOCX Parser (docx-rs)**: Parses untrusted `.docx` files — potential for zip bombs, XML external entity (XXE) attacks, or memory exhaustion via crafted documents. No size limits or sanitization applied.

2. **SQL Injection via PRAGMA key**: Key is interpolated into `PRAGMA key = '{key}'` — while hex encoding prevents `'` characters, the pattern is fragile. If key generation ever changes, this could become exploitable.

3. **No Tauri Allowlist**: The `tauri.conf.json` has no `allowlist` configuration. In Tauri 2.0, this means default restrictions apply, but explicit deny-by-default would be more secure.

4. **CSP `unsafe-inline` for styles**: Allows inline CSS — potential for CSS injection attacks, though limited impact in desktop app context.

5. **Key File Fallback**: On systems without keyring (CI, containers, some Linux desktops), encryption key stored in plaintext file. `0o600` only on Unix — Windows/macOS have no equivalent permission enforcement.

6. **No Rate Limiting**: IPC commands have no rate limiting — potential for DoS via rapid command invocation.

7. **Error Messages Leak Internal State**: `AppError` variants expose database errors, validation details, and internal paths to the frontend.

8. **No Authentication/Authorization**: All IPC commands accessible without authentication — any code running in the webview can invoke them.

9. **Clinical Note Data Exposure**: DOCX parser extracts patient IDs, diagnosis codes, and clinical notes — sensitive PHI transmitted through IPC without additional protection.

10. **No Audit Trail**: While `created_at`/`updated_at` timestamps exist, there's no audit log for who accessed or modified patient records.

### Files to Audit (Specific Paths)

**IPC Bridge:**
- `src-tauri/commands/src/patient_commands.rs` — Patient CRUD, all patient data flows
- `src-tauri/commands/src/accounting_commands.rs` — Financial data, balance validation
- `src-tauri/commands/src/diagnostics_commands.rs` — PHI (diagnosis codes)
- `src-tauri/commands/src/agenda_commands.rs` — Appointment scheduling, state machine
- `src-tauri/commands/src/age_commands.rs` — Date parsing
- `src-tauri/commands/src/tenant.rs` — Tenant config, keyring account names
- `src-tauri/commands/src/error.rs` — Error leakage
- `src-tauri/app/src/main.rs` — Command registration, setup

**Cryptography:**
- `src-tauri/infrastructure/src/keyring.rs` — Key lifecycle (generation, storage, retrieval)
- `src-tauri/infrastructure/src/database.rs` — SQLCipher connection, PRAGMA key injection

**Business Logic:**
- `src-tauri/domain/src/accounting.rs` — Balance validation, amount checks
- `src-tauri/domain/src/accounting_trigger.rs` — Double-entry bookkeeping
- `src-tauri/domain/src/appointment.rs` — State machine, duration validation
- `src-tauri/domain/src/value_objects.rs` — Input validation (Email, PhoneNumber, etc.)

**Application:**
- `src-tauri/application/src/docx_parser.rs` — Untrusted input parsing

**Configuration:**
- `src-tauri/tauri.conf.json` — CSP, security settings
- `tenant-configs/default.json` — Crypto config, keyring account names
- `src-tauri/Cargo.toml` — Dependency versions
- `src-tauri/Cargo.lock` — Exact dependency versions

**Tests:**
- `src-tauri/commands/src/e2e_integration.rs` — E2E security-relevant tests

### Recommendations

1. **DOCX Parser**: Add file size limits (e.g., 10MB max), consider using a sandboxed parser or validating DOCX structure before full parsing.

2. **PRAGMA Key**: Use parameterized queries or at minimum validate key format (exactly 64 hex chars) before interpolation.

3. **Tauri Allowlist**: Explicitly configure `allowlist` in tauri.conf.json to deny all IPC except required commands.

4. **Key Zeroization**: Return keys as `Zeroizing<String>` throughout the key lifecycle, not just during generation.

5. **Argon2 Usage**: The `argon2` crate is listed but unused — either use it for key derivation from a master password, or remove the dependency.

6. **Rate Limiting**: Consider implementing command-level rate limiting for resource-intensive operations.

7. **Error Sanitization**: Strip internal paths and database details from errors returned to frontend.

8. **Audit Logging**: Add append-only audit trail for patient record access/modification.

9. **File Permissions**: Extend `0o600` enforcement to Windows/macOS using platform-specific APIs.

10. **CSP Hardening**: Remove `unsafe-inline` from style-src if possible, or use nonces.

### Ready for Proposal

**Yes** — the exploration is complete. The orchestrator should:
1. Present the attack surface list to the user for prioritization
2. Determine which vectors to address in this change
3. Proceed to `sdd-propose` with the scoped security improvements
