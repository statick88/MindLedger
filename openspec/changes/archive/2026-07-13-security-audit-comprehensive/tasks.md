# Tasks: Comprehensive Security Audit

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 1,500–2,000 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 |
| Delivery strategy | auto-forecast |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | IPC fuzzing + business logic abuse tests | PR 1 (~550 lines) | `cargo test --package soft-gloria-commands -- ipc_fuzz business_logic` | In-memory SQLite + mock DB | `src-tauri/commands/src/security_audit/` directory |
| 2 | SQLCipher crypto verification tests | PR 2 (~400 lines) | `cargo test --package soft-gloria-infrastructure -- sqlcipher` | Temp files + mock keyring | `src-tauri/infrastructure/src/security_audit/` directory |
| 3 | Dependency audit, SAST scripts, report generator | PR 3 (~600 lines) | `bash scripts/dep-audit.sh && bash scripts/sast-clippy.sh` | Shell scripts on Kali Linux | `scripts/` directory + `sdd-archive/SECURITY-AUDIT-REPORT.md` |

## Phase 1: Foundation & Infrastructure

- [x] 1.1 Create shared types module `src-tauri/commands/src/security_audit/mod.rs` — define `SecurityFinding`, `Severity` enum, `SecurityTest` trait per design interfaces
- [x] 1.2 Create `src-tauri/infrastructure/src/security_audit/mod.rs` — module declarations for sqlcipher tests
- [x] 1.3 Add `[dev-dependencies]` to `src-tauri/Cargo.toml`: `assert_cmd`, `predicates` (for CLI assertion helpers)

## Phase 2: IPC Fuzzing Tests

- [x] 2.1 Create `src-tauri/commands/src/security_audit/ipc_fuzz_tests.rs` — test module skeleton with `use` imports
- [x] 2.2 Write RED test: DROP TABLE via PatientId — payload `"'; DROP TABLE patients; --"`, assert error returned, DB unchanged. Ref: spec ipc-injection/Scenario: DROP TABLE via PatientId
- [x] 2.3 Write RED test: UNION-based extraction — payload `"1 UNION SELECT * FROM asientos --"`, assert validation error, no data extracted. Ref: spec ipc-injection/Scenario: UNION-based extraction
- [x] 2.4 Write RED test: XSS/escape injection in clinical notes — DOCX with `<script>` tags, assert sanitized output, no panic. Ref: spec ipc-injection/Scenario: XSS/escape injection
- [x] 2.5 Write RED test: Zip bomb resilience — crafted .docx >100MB, assert parser rejects/limits, no OOM. Ref: spec ipc-injection/Scenario: Zip bomb resilience
- [x] 2.6 Write RED test: Tauri allowlist audit — parse `tauri.conf.json`, assert no `*` wildcards, no global capabilities. Ref: spec ipc-injection/Scenario: Allowlist audit
- [x] 2.7 Write RED test: Error leakage check — invoke error paths, assert no file paths/SQL/stack traces in messages. Ref: spec ipc-injection/Scenario: Error leakage check
- [x] 2.8 Write RED test: Escape character injection in clinical notes — fuzz `\n`, `\r`, `\0`, Unicode escapes in note fields, assert safe handling

## Phase 3: Business Logic Abuse Tests

- [x] 3.1 Create `src-tauri/commands/src/security_audit/business_logic_tests.rs` — test module skeleton
- [x] 3.2 Write RED test: Negative transaction amount — DTO with negative `lineas[].monto`, assert atomic rejection, no DB write. Ref: spec business-logic-abuse/Scenario: Negative transaction amount
- [x] 3.3 Write RED test: Overflow amount — amount exceeding `i64::MAX`, assert validation error, DB unchanged. Ref: spec business-logic-abuse/Scenario: Overflow amount
- [x] 3.4 Write RED test: Debit-credit imbalance — `sum(debito) != sum(credito)`, assert rejection, equation preserved. Ref: spec business-logic-abuse/Scenario: Debit-credit imbalance
- [x] 3.5 Write RED test: Invalid state transition — appointment in "Cancelada" → "Realizada", assert rejection. Ref: spec business-logic-abuse/Scenario: Invalid state transition
- [x] 3.6 Write RED test: Terminal state re-entry — appointment in "Realizada" → any state, assert rejection. Ref: spec business-logic-abuse/Scenario: Terminal state re-entry
- [x] 3.7 Write RED test: Missing required fields in DTOs — send DTOs with null/missing mandatory fields, assert validation errors

## Phase 4: SQLCipher Crypto Tests

- [x] 4.1 Create `src-tauri/infrastructure/src/security_audit/sqlcipher_tests.rs` — test module skeleton with tempfile + std::process imports
- [x] 4.2 Write RED test: Cold dump analysis — create test DB with PHI, run `strings`/`hexdump`, assert no readable schema/table names/PHI. Ref: spec sqlcipher-resistance/Scenario: strings/hexdump analysis
- [x] 4.3 Write RED test: Key zeroization — create key material with `Zeroizing<Vec<u8>>`, drop and verify zeroized. Ref: spec sqlcipher-resistance/Scenario: SIGKILL memory dump
- [x] 4.4 Write RED test: Key file fallback permissions — create temp key file, assert `0o600` permissions on Unix, content is 64-char hex. Ref: spec sqlcipher-resistance/Scenario: Key file permissions
- [x] 4.5 Write RED test: PRAGMA key injection safety — attempt malformed keys (quotes, SQL metacharacters), assert error not execution. Ref: spec sqlcipher-resistance/Scenario: Key format validation

## Phase 5: Dependency Audit Scripts

- [x] 5.1 Create `scripts/dep-audit.sh` — runs `cargo audit --json`, `cargo geiger --output-format json`, `pnpm audit --json`, outputs to `audit-output/`
- [x] 5.2 Create `scripts/dep-audit-report.sh` — parses JSON outputs, cross-references with CVE blocklist, produces `audit-output/dep-audit.json`
- [x] 5.3 Create `openspec/changes/security-audit-comprehensive/cve-blocklist.json` — empty initial blocklist with schema: `{"blocklisted_cves": [], "justifications": {}}`

## Phase 6: SAST Scripts

- [x] 6.1 Create `scripts/sast-clippy.sh` — runs `cargo clippy --workspace -- -D warnings`, captures output as JSON
- [x] 6.2 Create `scripts/sast-unsafe-audit.sh` — parses `cargo geiger` output, lists undocumented unsafe blocks
- [x] 6.3 Create `scripts/sast-error-leakage.sh` — greps `AppError` variants for file paths, SQL fragments, memory addresses

## Phase 7: Report Generator & Documentation

- [x] 7.1 Create `scripts/generate-audit-report.sh` — aggregates all `audit-output/*.json` into `SECURITY-AUDIT-REPORT.md`
- [x] 7.2 Create `sdd-archive/SECURITY-AUDIT-REPORT.md` template with CVSS v4.0 matrix, severity buckets, per-finding sections
- [x] 7.3 Write PoC documentation template in report: for each finding, include repro steps, affected file, OWASP/MITRE ref, CVSS score

## Phase 8: Integration & Verification

- [x] 8.1 Run full audit pipeline on Kali Linux: `bash scripts/dep-audit.sh && bash scripts/sast-clippy.sh && bash scripts/sast-unsafe-audit.sh && bash scripts/sast-error-leakage.sh`
- [x] 8.2 Run `cargo test --workspace` — verify all new tests compile and pass
- [x] 8.3 Verify all 25 spec scenarios: 6 IPC injection + 4 SQLCipher + 4 dependency + 5 business logic + 3 SAST = 22 (plus 3 additional from orchestrator epic totals)
- [x] 8.4 Generate final `SECURITY-AUDIT-REPORT.md` with all findings classified by OWASP ref and CVSS v4.0 score
- [x] 8.5 Document sign-off criteria: all CRITICAL/HIGH findings must have remediation plan before production deploy
