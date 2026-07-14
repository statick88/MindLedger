# Design: Comprehensive Security Audit

## Technical Approach

Add a security audit test suite covering five domains: IPC injection, SQLCipher resistance, dependency vulnerabilities, business logic abuse, and SAST. The suite will be implemented as Rust `#[cfg(test)]` modules with external tool integration, producing a structured report.

## Architecture Decisions

| Decision | Choice | Alternatives | Rationale |
|----------|--------|--------------|-----------|
| **Test Framework** | Rust `#[cfg(test)]` + `rstest` | pytest, custom harness | Existing project uses rstest; keeps test infrastructure consistent |
| **Property Testing** | `proptest` | quickcheck, bolero | Already in project; good for edge-case generation |
| **Mocking** | `mockall` | manual mocks | Already in project; auto-generated mocks for keyring |
| **External Tool Integration** | Shell scripts calling `cargo-audit`, `cargo-geiger`, `pnpm audit`, `strings`, `hexdump` | Rust crates (cargo-audit-rs) | Tools already available; JSON parsing via `serde_json` |
| **Report Format** | Markdown with CVSS v4.0 scoring | JSON, HTML | Human-readable; integrates with existing documentation |
| **Test Isolation** | In-memory SQLite for unit tests; temp directories for file system tests | Shared database | Prevents side effects; enables parallel test execution |

## Data Flow

```
Test Harness → IPC Commands → SQLite (in-memory) → Assertions
       ↓
External Tools (cargo-audit, etc.) → JSON Output → Report Generator
       ↓
Security Report (SECURITY-AUDIT-REPORT.md)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/commands/src/security_audit/ipc_fuzz.rs` | Create | IPC fuzzing harness with SQL/escape injection tests |
| `src-tauri/infrastructure/src/security_audit/sqlcipher.rs` | Create | Cold dump and key zeroization tests |
| `src-tauri/security_audit/dependency_audit.sh` | Create | Script for cargo-audit + pnpm audit |
| `src-tauri/commands/src/security_audit/business_logic.rs` | Create | Malformed DTO tests for accounting commands |
| `src-tauri/security_audit/sast.sh` | Create | clippy + cargo-geiger + custom lint |
| `src-tauri/security_audit/report_generator.rs` | Create | Aggregates results into SECURITY-AUDIT-REPORT.md |
| `openspec/changes/security-audit-comprehensive/design.md` | Create | This design document |
| `Cargo.toml` | Modify | Add dev-dependencies: `cargo-audit`, `cargo-geiger` (optional) |

## Interfaces / Contracts

```rust
// Security test result structure
#[derive(Serialize)]
struct SecurityFinding {
    id: String,
    severity: Severity, // Critical, High, Medium, Low, Info
    category: String,
    description: String,
    evidence: String,
    remediation: String,
    cvss_score: Option<f64>,
}

// Test harness trait
trait SecurityTest {
    fn run(&self) -> Vec<SecurityFinding>;
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | IPC injection, error sanitization | `#[tokio::test]` with mock DB, property-based fuzzing |
| Integration | SQLCipher cold dump, key lifecycle | Temp files, mock keyring, memory analysis |
| E2E | Business logic abuse | TypeScript harness sending malformed DTOs via Tauri IPC |
| SAST | Clippy warnings, unsafe code, error leakage | Shell scripts with JSON output parsing |
| Dependency | CVE scanning | cargo-audit + pnpm audit with blocklist |

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary.

## Migration / Rollout

No migration required. Audit tests are additive; existing test suite remains unchanged.

## Open Questions

- [ ] Should we add `cargo-audit` and `cargo-geiger` as dev-dependencies or install externally?
- [ ] What CVSS v4.0 scoring methodology should be used for findings?
- [ ] Should the audit report be generated automatically on CI or manually?
