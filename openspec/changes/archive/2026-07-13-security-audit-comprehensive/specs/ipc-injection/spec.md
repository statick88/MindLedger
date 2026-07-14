# IPC Injection & Sanitization Specification

## Purpose

Verify that all 27 Tauri IPC commands resist injection attacks, malformed payloads, and file-based exploits. This spec defines PASS/FAIL criteria for the OWASP A01:2021 attack vector.

## Requirements

### Requirement: SQL Injection Resistance

The system MUST reject all SQL injection payloads passed through IPC command parameters without executing any SQL or modifying database state.

#### Scenario: DROP TABLE via PatientId

- GIVEN: A Tauri command accepting `patient_id: String`
- WHEN: Payload `"'; DROP TABLE patients; --"` is sent
- THEN: Command returns error, no SQL executed, database unchanged
- AND: No error message exposes SQL fragments or table names

#### Scenario: UNION-based extraction

- GIVEN: A Tauri command accepting patient name field
- WHEN: Payload `"1 UNION SELECT * FROM asientos --"` is sent
- THEN: Command returns validation error, no data extracted
- AND: Database state is unchanged

### Requirement: DOCX Parser Injection Resistance

The system MUST sanitize embedded scripts, escape characters, and malicious XML in DOCX files without panicking or exposing internal state.

#### Scenario: XSS/escape injection in clinical notes

- GIVEN: DOCX file with `<script>` tags or escape sequences in body text
- WHEN: File is parsed by docx-rs
- THEN: Content is sanitized, no panic, no buffer overflow
- AND: Returned text is safe for display

#### Scenario: Zip bomb resilience

- GIVEN: Malicious .docx with nested compressed layers >100MB uncompressed
- WHEN: File is opened by the parser
- THEN: Parser rejects or limits decompression
- AND: Application remains responsive, process does not OOM

### Requirement: Tauri Allowlist Compliance

The system MUST follow least-privilege for all Tauri capabilities — no global permissions or overly broad IPC scopes.

#### Scenario: Allowlist audit

- GIVEN: tauri.conf.json security configuration
- WHEN: Inspected
- THEN: No global capabilities exist
- AND: All permissions follow least-privilege principle
- AND: No `*` wildcard permissions are present

### Requirement: Error Message Sanitization

IPC error responses MUST NOT expose database paths, SQL fragments, or internal state to the frontend.

#### Scenario: Error leakage check

- GIVEN: Any IPC command returning an error
- WHEN: Error is logged or displayed in frontend
- THEN: Message contains only user-safe description
- AND: No file paths, SQL, or stack traces are exposed
