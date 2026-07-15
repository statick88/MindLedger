# MindLdger FASE 5 — Release Archive

> Artifacts from FASE 5: Release Preparation (2026-07-13)

---

## Binary Metrics

| Metric | Value |
|--------|-------|
| Binary size | 13 MB |
| DMG installer | 5.8 MB |
| App bundle | 13 MB |
| Architectures | x86_64 + arm64 (universal) |
| Rust version | 1.97.0 (rustup) |
| Tauri CLI | 2.11.4 (npx) |
| Target | universal-apple-darwin |
| LTO | thin (fat LTO corrupted SQLCipher) |
| SQLCipher | 4.5.3 (bundled) |

## Test Suite

| Category | Count | Status |
|----------|-------|--------|
| Domain | 61 | ✅ |
| Infrastructure | 19 | ✅ |
| Commands | 22 | ✅ |
| Docx | 5 | ✅ |
| **Total** | **107** | **✅ 107/107** |

## Release Profile

```toml
[profile.release]
opt-level = 3
lto = "thin"        # MUST be "thin" — fat LTO corrupts SQLCipher C FFI
codegen-units = 1
strip = true
panic = "abort"
```

## Database Schema

### Patients (core)
- `id` TEXT PRIMARY KEY (UUID)
- `document_number` TEXT UNIQUE NOT NULL
- `document_type` TEXT DEFAULT 'cedula'
- `first_name`, `last_name` TEXT NOT NULL
- `date_of_birth` TEXT NOT NULL
- `email`, `phone`, `occupation`, `address`, `city`, `province`
- `emergency_contact_name`, `emergency_contact_phone`
- `insurance_provider`, `insurance_number`
- `status` TEXT DEFAULT 'active'
- `created_at`, `updated_at` TEXT NOT NULL

### Sessions
- `id`, `patient_id` (FK), `session_date`, `session_type`, `status`
- `clinical_notes`, `interventions`, `next_session_date`
- `payment_amount`, `payment_method`, `payment_status`

### Accounts
- `id`, `patient_id`, `session_id`, `type`, `category`, `description`
- `amount`, `date`, `payment_method`, `status`, `reference`

### Diagnostics
- `id`, `patient_id`, `session_id`, `code`, `system`
- `description`, `severity`, `status`, `date`, `notes`

## Security Audit Findings (Resolved)

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| 1 | Plaintext key file, no fs permissions | CRITICAL | ✅ Fixed (0o600) |
| 2 | Key material not zeroized through lifecycle | MEDIUM | ✅ Mitigated |
| 3 | PRAGMA key format (implicit 64-char) | LOW | Documented |
| 4 | create_pool_with_key public + no validation | LOW | Advisory |
| 5 | Mutex poisoning on panic | LOW | Mitigated by panic=abort |
| 6 | WAL .db-shm unencrypted sidecar | LOW | Known limitation |
| 7 | Release profile | SAFE | — |
| 8 | Key entropy (rand 0.8 + OS CSPRNG) | SAFE | — |

## Key Learnings

1. **Fat LTO corrupts SQLCipher C FFI** — always use `lto = "thin"` with bundled-sqlcipher
2. **PRAGMA key = 'hexstring'** (passphrase format) works for 64-char hex keys in SQLCipher 4.x
3. **Key file permissions** must be set to 0o600 on Unix to prevent key leakage
4. **Universal binaries** work out of the box with `npx tauri build --target universal-apple-darwin`
5. **rustup + Homebrew Rust** can coexist — rustup needed for cross-compilation targets
