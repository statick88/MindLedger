# Dependency Vulnerabilities Specification

## Purpose

Verify that all Rust and JavaScript dependencies are free of known critical/high CVEs, and that unsafe Rust code is documented and justified. Covers OWASP A06:2021.

## Requirements

### Requirement: Rust Supply Chain Audit

The Rust dependency tree MUST have 0 CRITICAL/HIGH CVEs. MEDIUM CVEs MUST be documented with mitigations.

#### Scenario: cargo-audit execution

- GIVEN: Cargo.lock in src-tauri/
- WHEN: `cargo-audit` is executed
- THEN: 0 CRITICAL CVEs found
- AND: 0 HIGH CVEs found
- AND: Any MEDIUM CVEs are documented with justification

### Requirement: JavaScript Supply Chain Audit

The JavaScript dependency tree MUST have 0 CRITICAL/HIGH CVEs.

#### Scenario: pnpm audit execution

- GIVEN: pnpm-lock.yaml
- WHEN: `pnpm audit` is executed
- THEN: 0 CRITICAL CVEs found
- AND: 0 HIGH CVEs found

### Requirement: Unsafe Code Documentation

All `unsafe` Rust blocks MUST be documented, audited, and justified. No undocumented unsafe code is permitted.

#### Scenario: cargo-geiger coverage

- GIVEN: All Rust source files in the workspace
- WHEN: `cargo-geiger` is executed
- THEN: All unsafe blocks are identified
- AND: Each unsafe block has a comment justifying its necessity
- AND: No undocumented unsafe code exists

### Requirement: Dead Dependency Cleanup

Dependencies listed in Cargo.toml but not imported MUST be removed or justified.

#### Scenario: Unused dependency check

- GIVEN: Cargo.toml dependency list
- WHEN: Cross-referenced with actual imports
- THEN: All dependencies are used or removed
- AND: `argon2` specifically is confirmed removed if unused
