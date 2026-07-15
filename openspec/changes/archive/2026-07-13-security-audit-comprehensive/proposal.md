# Proposal: Comprehensive Security Audit

## Intent

MindLdger handles PHI (patient records, CIE-10/DSM-5 diagnostics, clinical notes) and financial accounting data. Before production deployment, a systematic penetration test is required to identify exploitable vulnerabilities across IPC, cryptography, dependencies, and business logic. This is an audit, not a feature — it produces findings and remediation tasks, not new capabilities.

## Scope

### In Scope
- IPC bridge fuzzing and injection testing (27 Tauri commands)
- SQLCipher cold-dump analysis and key lifecycle audit
- Dependency vulnerability scanning (cargo-audit, pnpm audit)
- Business logic abuse testing (accounting equation, state machine)
- Static analysis (clippy, cargo-geiger, error leakage)
- DOCX parser attack surface assessment (highest-risk vector)

### Out of Scope
- Network MITM attacks (app is local-first, no network surface)
- Social engineering and physical access attacks
- Frontend XSS/CSRF (Tauri webview sandbox handles this)
- Compliance certification (HIPAC, ISO 27001 — separate engagement)

## Capabilities

### New Capabilities
None — audit produces findings, not new behaviors.

### Modified Capabilities
None — audit recommendations will be implemented as separate changes after findings are triaged.

## Approach

### Vector 1: IPC Bridge & Access Control (OWASP A01:2021 / MITRE T1480)

| Test | Tool | Target |
|------|------|--------|
| Fuzz 27 IPC commands with malformed payloads | `cargo-fuzz`, custom harness | `src-tauri/commands/src/*.rs` |
| SQL injection via PatientId and clinical notes | `sqlmap` on extracted queries | `patient_commands.rs`, `docx_parser.rs` |
| DOCX parser XXE/zip-bomb | Crafted `.docx` files, `strings`, memory profiler | `application/src/docx_parser.rs` |
| Tauri allowlist audit | Manual review + `tauri.conf.json` validation | `src-tauri/tauri.conf.json` |
| Error message leakage scan | Grep for `AppError` variants exposed to IPC | `commands/src/error.rs` |

### Vector 2: Cryptography at Rest & Secrets (OWASP A02:2021 / NIST SP 800-175B)

| Test | Tool | Target |
|------|------|--------|
| Cold dump of `.db` file | `strings`, `hexdump`, `xxd`, `binwalk` | SQLCipher database files |
| Key zeroization audit | `volatility`, memory dump after process close | `keyring.rs` |
| PRAGMA key injection safety | Regex validation of hex key format | `database.rs` |
| File fallback key permissions | Cross-platform permission check | `keyring.rs` (file fallback path) |
| Key rotation absence | Manual review | `keyring.rs` lifecycle |

### Vector 3: Dependency & Business Logic (OWASP A06:2021 / A08:2021)

| Test | Tool | Target |
|------|------|--------|
| Rust SCA | `cargo audit` on `Cargo.lock` | `src-tauri/Cargo.lock` |
| JS/TS SCA | `pnpm audit` on `pnpm-lock.yaml` | `pnpm-lock.yaml` |
| Unsafe blocks audit | `cargo-geiger` | All Rust crates |
| Negative amount injection | Crafted DTOs with negative values | `accounting_commands.rs` |
| State machine bypass | Invalid transition sequences | `appointment.rs` |
| Accounting equation imbalance | Manipulated line items | `accounting.rs`, `accounting_trigger.rs` |

### Vector 4: SAST

| Test | Tool | Target |
|------|------|--------|
| Clippy security flags | `cargo clippy -- -W clippy::all` | All Rust workspace |
| Dead dependency check | `cargo-geiger` + manual review | `argon2` (listed but unused) |
| Error path sanitization | Grep for internal state in error messages | `error.rs`, all `Result` paths |

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/commands/src/*.rs` | Reviewed | IPC fuzzing, injection testing |
| `src-tauri/infrastructure/src/keyring.rs` | Reviewed | Key lifecycle audit |
| `src-tauri/infrastructure/src/database.rs` | Reviewed | PRAGMA key injection |
| `src-tauri/application/src/docx_parser.rs` | Reviewed | Highest-risk attack surface |
| `src-tauri/domain/src/accounting.rs` | Reviewed | Balance invariant testing |
| `src-tauri/domain/src/appointment.rs` | Reviewed | State machine bypass |
| `src-tauri/tauri.conf.json` | Reviewed | CSP, allowlist validation |
| `src-tauri/Cargo.lock` | Reviewed | Dependency audit |
| `pnpm-lock.yaml` | Reviewed | JS dependency audit |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| DOCX parser exploitable with crafted files | High | Add size limits before remediation; test in sandboxed env |
| Key zeroization incomplete — PHI in process memory | Medium | Verify `Zeroizing` types end-to-end; dump memory post-exec |
| `argon2` dead dependency masks unused security feature | Low | Confirm no import; decide use-or-remove |
| PRAGMA key format string could inject SQL | Low | Hex encoding prevents `'`; enforce 64-char validation |

## Rollback Plan

This is a READ-ONLY audit — no code changes are made during the audit phase. All findings are documented in a verify-report. Remediation tasks are created as separate SDD changes after triage. If any audit step risks data corruption (e.g., memory dump), run against test fixtures only, never production databases.

## Dependencies

- Kali Linux tools: `sqlmap`, `strings`, `hexdump`, `xxd`, `binwalk`
- Rust tooling: `cargo-audit`, `cargo-geiger`, `cargo-clippy`, `cargo-fuzz`
- Node tooling: `pnpm audit`
- Memory analysis: `volatility` or equivalent

## Success Criteria

- [ ] All 27 IPC commands fuzzed with at least 100 malformed inputs each
- [ ] DOCX parser tested with zip-bomb, XXE, and oversized files
- [ ] SQLCipher cold dump confirms no plaintext PHI in raw bytes
- [ ] Key memory zeroization verified via post-process dump
- [ ] `cargo-audit` and `pnpm audit` results documented with CVE IDs
- [ ] `cargo-geiger` unsafe coverage reported
- [ ] All findings classified by OWASP/MITRE reference and severity (Critical/High/Medium/Low)
- [ ] Remediation tasks created for all Critical and High findings
