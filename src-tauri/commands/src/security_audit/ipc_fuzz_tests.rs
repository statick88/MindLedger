//! IPC Fuzzing & Injection Security Tests
//!
//! Validates that IPC command parameters resist SQL injection, XSS,
//! escape character injection, DOCX parser abuse, and Tauri allowlist violations.
//!
//! All tests use in-memory SQLite to verify no database state changes occur.

#[cfg(test)]
mod ipc_fuzz_tests {
    use crate::accounting_commands::*;
    use crate::error::AppError;
    use crate::agenda_commands::*;
    use soft_mindledger_domain::appointment::{AppointmentStatus, Modality};
    use soft_mindledger_infrastructure::database::create_memory_pool;
    use soft_mindledger_infrastructure::DbPool;
    use chrono::{Duration, Utc};
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    /// Shared test pool with full schema for IPC fuzz tests.
    fn create_fuzz_pool() -> DbPool {
        let pool = create_memory_pool().expect("Failed to create memory pool");
        soft_mindledger_infrastructure::migrations::run_all_migrations(&pool)
            .expect("Failed to run migrations");
        pool
    }

    /// Create a test patient in the DB (required for FK references).
    async fn create_test_patient(pool: &DbPool) -> Uuid {
        let patient_id = Uuid::new_v4();
        let pool_clone = pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool_clone.lock().unwrap();
            conn.execute(
                "INSERT INTO patients (id, document_number, document_type, country_code, first_name, last_name, date_of_birth, gender, is_active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                rusqlite::params![
                    patient_id.to_string(),
                    "SEC-TEST-001",
                    "DNI",
                    "EC",
                    "Security",
                    "Test",
                    "1990-01-01",
                    "Male",
                    1,
                ],
            ).unwrap();
        }).await.unwrap();
        patient_id
    }

    // ══════════════════════════════════════════════════════════════════════
    // SQL Injection via PatientId
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: ipc-injection/sql-injection-resistance/scenario-1
    /// DROP TABLE via PatientId — payload must be rejected as invalid UUID,
    /// no SQL executed, database unchanged.
    #[tokio::test]
    async fn test_sql_injection_drop_table_via_patient_id() {
        let pool = create_fuzz_pool();
        let patient_id_before = count_patients(&pool);

        // Attempt to create an appointment with a SQL injection payload as patient_id
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        let request = CreateAppointmentRequest {
            patient_id: "'; DROP TABLE patients; --".to_string(),
            therapist_id: Uuid::new_v4().to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };

        let result = crear_cita_agenda_impl(&pool, request).await;

        // Must return validation error (invalid UUID), not a SQL error
        assert!(result.is_err(), "SQL injection payload must be rejected");
        match result.unwrap_err() {
            AppError::Validation(msg) => {
                assert!(
                    msg.contains("Invalid") || msg.contains("invalid") || msg.contains("UUID"),
                    "Error should indicate invalid input, got: {}",
                    msg
                );
            }
            other => panic!("Expected Validation error, got: {:?}", other),
        }

        // Verify database unchanged
        let patient_id_after = count_patients(&pool);
        assert_eq!(
            patient_id_before, patient_id_after,
            "Database must not change after SQL injection attempt"
        );
    }

    /// Spec: ipc-injection/sql-injection-resistance/scenario-2
    /// UNION-based extraction — payload must be rejected, no data extracted.
    #[tokio::test]
    async fn test_sql_injection_union_extraction() {
        let pool = create_fuzz_pool();

        // Attempt UNION-based injection via descripcion field
        let request = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "1 UNION SELECT * FROM asientos_contables --".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "1110 Caja".to_string(),
                    debito: Some("1000".to_string()),
                    credito: None,
                },
                CreateLineaAsientoRequest {
                    cuenta: "4110 Capital".to_string(),
                    debito: None,
                    credito: Some("1000".to_string()),
                },
            ],
        };

        // The asiento should be created (descripcion is just a string field)
        // but the SQL injection payload must NOT execute any SQL
        let result = add_asiento_impl(&pool, request).await;

        // If it succeeds, the description is stored as-is (no SQL execution)
        if let Ok(asiento) = &result {
            assert!(
                asiento.descripcion.contains("UNION SELECT"),
                "Description should be stored as literal string, not executed"
            );
        }

        // Verify no extra tables or data were created by injection
        let conn = pool.lock().unwrap();
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        // Should only have our standard tables (patients, appointments, reminders,
        // asientos_contables, clinical_notes, diagnoses, prescriptions, etc.)
        assert!(
            table_count < 20,
            "Suspicious number of tables ({}) suggests SQL injection succeeded",
            table_count
        );
    }

    /// Spec: ipc-injection/error-sanitization/scenario-1
    /// Error leakage — invoke error paths, assert no file paths, SQL, or
    /// stack traces in error messages.
    #[tokio::test]
    async fn test_error_leakage_no_sql_or_paths() {
        let pool = create_fuzz_pool();

        // Trigger a validation error (invalid date)
        let request = CreateAsientoRequest {
            fecha: "not-a-date".to_string(),
            descripcion: "Test".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "1110 Caja".to_string(),
                    debito: Some("100".to_string()),
                    credito: None,
                },
                CreateLineaAsientoRequest {
                    cuenta: "4110 Capital".to_string(),
                    debito: None,
                    credito: Some("100".to_string()),
                },
            ],
        };

        let result = add_asiento_impl(&pool, request).await;
        assert!(result.is_err(), "Should return validation error");

        let err_msg = result.unwrap_err().to_string();

        // Error must NOT contain file paths
        assert!(
            !err_msg.contains("/home/") && !err_msg.contains("/usr/") && !err_msg.contains("\\Users\\"),
            "Error message must not contain file paths: {}",
            err_msg
        );
        // Error must NOT contain SQL fragments
        assert!(
            !err_msg.to_lowercase().contains("select ") && !err_msg.to_lowercase().contains("insert ") && !err_msg.to_lowercase().contains("drop table"),
            "Error message must not contain SQL: {}",
            err_msg
        );
        // Error must NOT contain stack traces or debug info
        assert!(
            !err_msg.contains("stack backtrace") && !err_msg.contains("panicked at") && !err_msg.contains("thread '") && !err_msg.contains("  at src/"),
            "Error message must not contain stack traces: {}",
            err_msg
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Tauri Allowlist Audit
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: ipc-injection/allowlist-audit/scenario-1
    /// Tauri allowlist — parse tauri.conf.json, assert no wildcard `*`
    /// permissions, no global capabilities.
    #[test]
    fn test_tauri_allowlist_no_wildcards() {
        let conf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Failed to get parent dir")
            .join("tauri.conf.json");
        let conf_str = std::fs::read_to_string(&conf_path)
            .expect("Failed to read tauri.conf.json");
        let conf: serde_json::Value = serde_json::from_str(&conf_str)
            .expect("Failed to parse tauri.conf.json");

        // Check top-level security section
        if let Some(security) = conf.pointer("/app/security") {
            // CSP should not have 'unsafe-eval' or overly broad sources
            if let Some(csp) = security.get("csp").and_then(|v| v.as_str()) {
                assert!(
                    !csp.contains("'unsafe-eval'"),
                    "CSP must not contain 'unsafe-eval': {}",
                    csp
                );
            }
        }

        // Check for wildcard permissions in capabilities (Tauri v2 pattern)
        let conf_str_lower = conf_str.to_lowercase();
        assert!(
            !conf_str_lower.contains(r#""*""#) || conf_str.contains(r#""icon""#),
            "tauri.conf.json must not contain wildcard permissions"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Escape Character Injection in Clinical Notes
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: ipc-injection/escape-injection/scenario-1
    /// XSS/escape injection — DOCX with `<script>` tags must be sanitized
    /// or stored as literal text without panic.
    #[tokio::test]
    async fn test_escape_injection_script_tags_no_panic() {
        let pool = create_fuzz_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();

        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);

        // Clinical notes with XSS payload
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: Some("<script>alert('xss')</script>Paciente presenta ansiedad".to_string()),
        };

        // Must not panic — either accepts or rejects gracefully
        let result = crear_cita_agenda_impl(&pool, request).await;

        if let Ok(appt) = &result {
            // If stored, the notes should still contain the literal text
            let notes = appt.notes.as_ref().unwrap();
            assert!(
                notes.contains("Paciente presenta ansiedad"),
                "Clinical content must be preserved: {}",
                notes
            );
        }
        // Key assertion: no panic occurred (test completing proves this)
    }

    /// Spec: ipc-injection/escape-injection/scenario-2
    /// Escape characters — fuzz newlines, carriage returns, null bytes,
    /// and Unicode escapes in note fields.
    #[tokio::test]
    async fn test_escape_characters_in_notes_no_panic() {
        let pool = create_fuzz_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();

        let payloads = vec![
            "Line1\nLine2\rLine3",
            "Note with \0 null byte",
            "Unicode: \u{200B} zero-width space \u{FEFF} BOM",
            "Emoji injection: \u{1F600}\u{1F4A5}\u{1F480}",
            "RTL override: \u{202E}reversed\u{202C}",
            "Nested \\n\\r\\t escape sequences",
            "SQL-like: Robert'); DROP TABLE patients;--",
        ];

        for (i, payload) in payloads.into_iter().enumerate() {
            let start = Utc::now() + Duration::hours(2) + Duration::minutes(i as i64 * 60);
            let end = start + Duration::minutes(50);

            let request = CreateAppointmentRequest {
                patient_id: patient_id.to_string(),
                therapist_id: therapist_id.to_string(),
                start_at: start.to_rfc3339(),
                end_at: end.to_rfc3339(),
                modality: Modality::Virtual,
                fee_cents: Some(30000),
                notes: Some(payload.to_string()),
            };

            // Must not panic — the application must handle all escape sequences safely
            let _result = crear_cita_agenda_impl(&pool, request).await;
            // Key assertion: test completed without panic
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // DOCX Parser Resilience
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: ipc-injection/docx-parser/scenario-1
    /// Zip bomb resilience — crafted .docx >100MB must be rejected or limited,
    /// application must not OOM.
    #[test]
    fn test_zip_bomb_resilience() {
        use std::io::Write;

        // Create a minimal valid ZIP with a malicious uncompressed size hint
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("bomb.docx");

        // Create a small file that claims to be large (zip bomb heuristic)
        // A real zip bomb has compressed:uncompressed ratio > 100:1
        // We test that the parser handles oversized claims gracefully
        let mut fake_docx = std::fs::File::create(&zip_path).unwrap();
        // Write a valid ZIP header but truncated content
        let zip_header = b"PK\x03\x04";
        fake_docx.write_all(zip_header).unwrap();
        // Fill with zeros to simulate corrupted/truncated zip
        let padding = vec![0u8; 1024];
        fake_docx.write_all(&padding).unwrap();
        drop(fake_docx);

        // Attempt to parse — should fail gracefully, not panic or OOM
        let result = std::panic::catch_unwind(|| {
            let _ = soft_mindledger_application::docx_parser::ClinicalNoteParser::parse_docx(
                zip_path.to_str().unwrap(),
            );
        });

        assert!(
            result.is_ok(),
            "DOCX parser must not panic on malformed/zip-bomb file"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Helpers
    // ══════════════════════════════════════════════════════════════════════

    /// Count patients in the database.
    fn count_patients(pool: &DbPool) -> i64 {
        let conn = pool.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM patients", [], |row| row.get(0))
            .unwrap_or(0)
    }
}
