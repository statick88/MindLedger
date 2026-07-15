# SQLCipher Resistance Specification

## Purpose

Verify that the SQLCipher database layer resists cold-dump analysis, key extraction, and unauthorized decryption. Covers OWASP A02:2021 and NIST SP 800-175B requirements.

## Requirements

### Requirement: Cold Dump Indistinguishability

The persisted .db file MUST be indistinguishable from random noise when analyzed with standard forensics tools.

#### Scenario: strings/hexdump analysis

- GIVEN: The persisted .db file in $APPDATA
- WHEN: Analyzed with `strings`, `hexdump`, `xxd`, `foremost`, `binwalk`
- THEN: Output contains no readable schema, table names, or PHI
- AND: Output is indistinguishable from random noise

### Requirement: Key Zeroization

The encryption key MUST be zeroized from process memory after use. MITRE T1003 resistance is required.

#### Scenario: SIGKILL memory dump

- GIVEN: Application running with encryption key loaded in memory
- WHEN: Process is killed (SIGKILL)
- THEN: Memory dump shows no plaintext key bytes
- AND: `Zeroizing` types are used for all key material end-to-end

### Requirement: Key File Fallback Security

When keyring is unavailable, the fallback key file MUST be protected by filesystem permissions and store only hex-encoded key material.

#### Scenario: Key file permissions

- GIVEN: System without keyring support
- WHEN: Fallback key file is created
- THEN: File permissions are 0o600
- AND: Content is hex-encoded key (64 chars), not plaintext

### Requirement: PRAGMA Key Injection Safety

The PRAGMA key format MUST prevent SQL injection through the key string itself.

#### Scenario: Key format validation

- GIVEN: Database initialization with PRAGMA key
- WHEN: Key is processed
- THEN: Key is validated as exactly 64 hex characters
- AND: No single quotes or SQL metacharacters are present
- AND: Malformed key produces error, not SQL execution
