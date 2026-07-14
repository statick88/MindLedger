# SAST & Static Analysis Specification

## Purpose

Verify that the Rust codebase passes static analysis without security-related warnings and that error messages do not leak internal state.

## Requirements

### Requirement: Clippy Clean Workspace

The workspace MUST pass `cargo clippy` with zero warnings, especially no unsafe-related warnings.

#### Scenario: Clippy security audit

- GIVEN: All Rust source code in the workspace
- WHEN: `cargo clippy --workspace -- -D warnings` is executed
- THEN: 0 warnings produced
- AND: No unsafe-related warnings exist
- AND: No clippy::all violations exist

### Requirement: Error Message Non-Leakage

AppError variants MUST NOT expose database paths, SQL fragments, or internal state in any user-visible or log output.

#### Scenario: Error path sanitization

- GIVEN: Any `AppError` variant returned from a command
- WHEN: Error message is logged or displayed
- THEN: No database file paths are present
- AND: No SQL fragments or query text are present
- AND: No internal state (struct fields, memory addresses) is present
- AND: Message contains only a user-safe description string
