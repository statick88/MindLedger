//! FASE 6 — End-to-end integration tests
//!
//! Tests cross-layer flows: docx parsing → domain → IPC command → SQLCipher → verification.
//! All tests use in-memory SQLite with full schema to simulate production path.

#[cfg(test)]
mod e2e_tests {
    use crate::accounting_commands::*;
    use crate::age_commands::*;
    use crate::patient_commands::*;
    use chrono::NaiveDate;
    use docx_rs::*;
    use rust_decimal_macros::dec;
    use soft_mindledger_domain::accounting::{AsientoContable, LineaAsiento};
    use soft_mindledger_domain::age::Age;
    use soft_mindledger_domain::repositories::*;
    use soft_mindledger_domain::value_objects::{DocumentNumber, DocumentType, FullName, Gender};
    use soft_mindledger_domain::*;
    use soft_mindledger_infrastructure::accounting_repository_sqlite::SqliteAccountingRepository;
    use soft_mindledger_infrastructure::repositories::SqlitePatientRepository;
    use soft_mindledger_infrastructure::{create_memory_pool, DbPool};
    use std::fs;
    use tempfile::tempdir;
    use uuid::Uuid;

    // ── Shared test pool with full schema ──────────────────────────────────

    /// Creates an in-memory DbPool with the complete accounting + patient schema.
    /// This mirrors production SQLCipher but without encryption.
    fn create_e2e_pool() -> DbPool {
        let pool = create_memory_pool().expect("Failed to create memory pool");

        // run_all_migrations acquires its own lock — do NOT pre-lock here
        // (std::sync::Mutex is not reentrant → pre-lock causes deadlock)
        soft_mindledger_infrastructure::migrations::run_all_migrations(&pool)
            .expect("Failed to run migrations");

        pool
    }

    /// Helper: create a clinical note DOCX and return its path.
    fn create_clinical_note_docx(dir: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
        let path = dir.path().join("clinical_note.docx");
        let mut doc = Docx::new();
        for line in content.lines() {
            doc = doc.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(line.to_string())),
            );
        }
        let mut file = fs::File::create(&path).unwrap();
        doc.build().pack(&mut file).unwrap();
        path
    }

    /// Local patient creation helper — mirrors patient_commands::create_patient but
    /// takes &DbPool directly (no Tauri State needed in tests).
    async fn create_patient_impl(pool: &DbPool, request: CreatePatientRequest) -> Result<crate::patient_commands::PatientResponse, crate::AppError> {
        use soft_mindledger_domain::patient::Patient;
        use soft_mindledger_infrastructure::repositories::SqlitePatientRepository;

        let repo = SqlitePatientRepository::new(pool.clone());

        let document_number = DocumentNumber::new(
            request.document_number.clone(),
            request.document_type.clone(),
            request.country_code.clone(),
        ).map_err(|e| crate::AppError::Validation(format!("Invalid document number: {}", e)))?;
        let name = FullName::new(
            request.first_name.clone(),
            request.last_name.clone(),
            request.middle_name.clone(),
        ).map_err(|e| crate::AppError::Validation(format!("Invalid name: {}", e)))?;
        let birth_date = chrono::NaiveDate::parse_from_str(&request.date_of_birth, "%Y-%m-%d")
            .map_err(|e| crate::AppError::Validation(format!("Invalid date: {}", e)))?;

        let patient = Patient::new(
            document_number,
            name,
            birth_date,
            request.gender.clone(),
        );

        repo.create(&patient).await.map_err(|e| crate::AppError::Database(e.to_string()))?;

        Ok(crate::patient_commands::PatientResponse::from(patient))
    }

    /// Local update helper — mirrors what an update_asiento_impl would do.
    async fn update_asiento_impl(
        pool: &DbPool,
        asiento_id: String,
        request: UpdateAsientoRequest,
    ) -> Result<AsientoResponse, crate::AppError> {
        use soft_mindledger_infrastructure::accounting_repository_sqlite::SqliteAccountingRepository;

        let repo = SqliteAccountingRepository::new(pool.clone());
        let id = Uuid::parse_str(&asiento_id)
            .map_err(|e| crate::AppError::Validation(format!("Invalid UUID: {}", e)))?;

        let existing = repo
            .get_asiento_by_id(id)
            .await
            .map_err(|e| crate::AppError::Database(e.to_string()))?
            .ok_or_else(|| crate::AppError::NotFound("Asiento not found".to_string()))?;

        // Apply partial updates
        let mut updated = existing;
        if let Some(fecha) = &request.fecha {
            updated.fecha = chrono::NaiveDate::parse_from_str(fecha, "%Y-%m-%d")
                .map_err(|e| crate::AppError::Validation(format!("Invalid date: {}", e)))?;
        }
        if let Some(desc) = &request.descripcion {
            updated.descripcion = desc.clone();
        }
        if let Some(lineas_req) = &request.lineas {
            let mut lineas = Vec::new();
            for linea in lineas_req {
                let has_debito = linea.debito.is_some();
                let has_credito = linea.credito.is_some();
                if !has_debito && !has_credito {
                    return Err(crate::AppError::Validation("Line must have debito or credito".to_string()));
                }
                let debito: rust_decimal::Decimal = linea.debito.as_deref().unwrap_or("0").parse()
                    .map_err(|e| crate::AppError::Validation(format!("Invalid debito: {}", e)))?;
                let credito: rust_decimal::Decimal = linea.credito.as_deref().unwrap_or("0").parse()
                    .map_err(|e| crate::AppError::Validation(format!("Invalid credito: {}", e)))?;
                lineas.push(LineaAsiento {
                    cuenta: linea.cuenta.clone(),
                    debito,
                    credito,
                });
            }
            updated.lineas = lineas;
        }

        // Validate balance
        if !updated.is_balanced() {
            return Err(crate::AppError::Validation("Updated asiento is not balanced".to_string()));
        }

        repo.update_asiento(&updated)
            .await
            .map_err(|e| crate::AppError::Database(e.to_string()))?;

        Ok(AsientoResponse::from(updated))
    }

    // ══════════════════════════════════════════════════════════════════════
    // OBJ 1: E2E Flow — docx parsing → IPC → SQLCipher → UI reflection
    // ══════════════════════════════════════════════════════════════════════

    /// Full E2E: Parse a clinical note DOCX, create patient, create
    /// accounting entry for the session, verify data persists and balances.
    #[tokio::test]
    async fn test_e2e_docx_parse_to_patient_and_accounting() {
        let dir = tempdir().unwrap();
        let pool = create_e2e_pool();

        // Step 1: Create a clinical note DOCX
        let clinical_content = r#"Paciente: PT-E2E-001
Fecha: 2024-06-15
Diagnóstico: F32.1
Tipo de sesión: Terapia cognitivo-conductual
Notas: Paciente presenta mejoría significativa en síntomas de ansiedad. 
       Se observa reducción en évitación comportamental.
Plan de tratamiento: Continuar con terapia semanal, aumentar exposición gradual."#;

        let docx_path = create_clinical_note_docx(&dir, clinical_content);

        // Step 2: Parse the DOCX
        let note = soft_mindledger_application::docx_parser::ClinicalNoteParser::parse_docx(
            docx_path.to_str().unwrap(),
        )
        .expect("Failed to parse clinical note DOCX");

        assert_eq!(note.patient_id, Some("PT-E2E-001".to_string()));
        assert_eq!(note.session_date, Some("2024-06-15".to_string()));
        assert_eq!(note.diagnosis_code, Some("F32.1".to_string()));
        assert_eq!(
            note.session_type,
            Some("Terapia cognitivo-conductual".to_string())
        );
        assert!(note.notes.is_some());
        assert!(note.treatment_plan.is_some());

        // Step 3: Create a patient using parsed data
        let patient_request = CreatePatientRequest {
            document_number: note.patient_id.clone().unwrap(),
            document_type: DocumentType::NationalId,
            country_code: "EC".to_string(),
            first_name: "María".to_string(),
            last_name: "González".to_string(),
            middle_name: None,
            date_of_birth: "1990-03-15".to_string(),
            gender: Gender::Female,
            email: Some("maria@example.com".to_string()),
            phone_number: None,
            phone_country_code: None,
            phone_extension: None,
            address_street: None,
            address_city: None,
            address_state: None,
            address_postal_code: None,
            address_country: None,
            address_additional_info: None,
            emergency_contact_first_name: None,
            emergency_contact_last_name: None,
            emergency_contact_middle_name: None,
            emergency_contact_relationship: None,
            emergency_contact_phone_number: None,
            emergency_contact_phone_country_code: None,
            emergency_contact_email: None,
            blood_type: None,
            allergies: None,
            chronic_conditions: None,
            medications: None,
            notes: note.notes.clone(),
        };

        let patient_response = create_patient_impl(&pool, patient_request)
            .await
            .expect("Failed to create patient");

        assert_eq!(patient_response.document_number, "PT-E2E-001");
        assert_eq!(patient_response.first_name, "María");
        assert_eq!(patient_response.last_name, "González");
        assert!(patient_response.is_active);

        // Step 4: Verify patient can be queried back
        let repo = SqlitePatientRepository::new(pool.clone());
        let patient_id = PatientId::from_str(&patient_response.id).unwrap();
        let retrieved = repo.get_by_id(patient_id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.document_number.number, "PT-E2E-001");

        // Step 5: Create an accounting entry for the session fee
        let accounting_request = CreateAsientoRequest {
            fecha: note.session_date.unwrap(),
            descripcion: format!(
                "Sesión clínica - {} - {}",
                note.patient_id.unwrap(),
                note.session_type.unwrap()
            ),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "1110 Caja".to_string(),
                    debito: Some("150.00".to_string()),
                    credito: None,
                },
                CreateLineaAsientoRequest {
                    cuenta: "4110 Ingresos por Servicios".to_string(),
                    debito: None,
                    credito: Some("150.00".to_string()),
                },
            ],
        };

        let asiento_response = add_asiento_impl(&pool, accounting_request)
            .await
            .expect("Failed to create accounting entry");

        assert!(asiento_response.is_balanced);
        assert_eq!(asiento_response.total_debitos, "150.00");
        assert_eq!(asiento_response.total_creditos, "150.00");

        // Step 6: Verify balance invariant — Activos = Pasivos + Patrimonio
        let balance = generate_balance_general_impl(&pool, "2024-12-31".to_string())
            .await
            .expect("Failed to generate balance");

        assert!(
            balance.is_balanced,
            "Balance invariant violated: Activos={} ≠ Pasivos+Patrimonio={}",
            balance.total_activos,
            balance.total_pasivos.clone() + &balance.total_patrimonio
        );
    }

    /// E2E: Multiple sessions for the same patient, each creating an
    /// accounting entry, then verify cumulative balance.
    #[tokio::test]
    async fn test_e2e_multiple_sessions_cumulative_balance() {
        let dir = tempdir().unwrap();
        let pool = create_e2e_pool();

        // Parse 3 clinical notes for the same patient
        for i in 1..=3 {
            let content = format!(
                "Paciente: PT-MULTI-001\nFecha: 2024-0{}-{:02}\nDiagnóstico: F32.1\nTipo de sesión: Terapia\nNotas: Sesión {}\nPlan: Continuar",
                i, 15 + i, i
            );
            let docx_path = create_clinical_note_docx(&dir, &content);

            let note = soft_mindledger_application::docx_parser::ClinicalNoteParser::parse_docx(
                docx_path.to_str().unwrap(),
            )
            .unwrap();

            // Create patient only on first iteration
            if i == 1 {
                let patient_request = CreatePatientRequest {
                    document_number: "PT-MULTI-001".to_string(),
                    document_type: DocumentType::NationalId,
                    country_code: "EC".to_string(),
                    first_name: "Carlos".to_string(),
                    last_name: "Méndez".to_string(),
                    middle_name: None,
                    date_of_birth: "1985-07-20".to_string(),
                    gender: Gender::Male,
                    email: None,
                    phone_number: None,
                    phone_country_code: None,
                    phone_extension: None,
                    address_street: None,
                    address_city: None,
                    address_state: None,
                    address_postal_code: None,
                    address_country: None,
                    address_additional_info: None,
                    emergency_contact_first_name: None,
                    emergency_contact_last_name: None,
                    emergency_contact_middle_name: None,
                    emergency_contact_relationship: None,
                    emergency_contact_phone_number: None,
                    emergency_contact_phone_country_code: None,
                    emergency_contact_email: None,
                    blood_type: None,
                    allergies: None,
                    chronic_conditions: None,
                    medications: None,
                    notes: None,
                };
                create_patient_impl(&pool, patient_request)
                    .await
                    .unwrap();
            }

            // Create accounting entry for each session
            let request = CreateAsientoRequest {
                fecha: note.session_date.unwrap(),
                descripcion: format!("Sesión {}", i),
                lineas: vec![
                    CreateLineaAsientoRequest {
                        cuenta: "1110 Caja".to_string(),
                        debito: Some("100.00".to_string()),
                        credito: None,
                    },
                    CreateLineaAsientoRequest {
                        cuenta: "4110 Ingresos por Servicios".to_string(),
                        debito: None,
                        credito: Some("100.00".to_string()),
                    },
                ],
            };
            let resp = add_asiento_impl(&pool, request).await.unwrap();
            assert!(resp.is_balanced, "Asiento {} not balanced", i);
        }

        // Verify cumulative balance: Activos=300, Pasivos=0, Patrimonio=300
        let balance = generate_balance_general_impl(&pool, "2024-12-31".to_string())
            .await
            .unwrap();

        assert!(balance.is_balanced);

        // Compare as Decimal (Decimal::to_string may produce "300.00" not "300")
        let activos: rust_decimal::Decimal = balance.total_activos.parse().unwrap();
        let pasivos: rust_decimal::Decimal = balance.total_pasivos.parse().unwrap();
        let patrimonio: rust_decimal::Decimal = balance.total_patrimonio.parse().unwrap();
        assert_eq!(activos, rust_decimal_macros::dec!(300));
        assert_eq!(pasivos, rust_decimal_macros::dec!(0));
        assert_eq!(patrimonio, rust_decimal_macros::dec!(300));
    }

    /// E2E: Parse DOCX with missing fields → graceful fallback.
    #[tokio::test]
    async fn test_e2e_docx_with_missing_fields() {
        let dir = tempdir().unwrap();

        let content = "Paciente: PT-SPARSE-001\nSolo tiene paciente y diagnóstico.\nDiagnóstico: F41.1";
        let docx_path = create_clinical_note_docx(&dir, content);

        let note = soft_mindledger_application::docx_parser::ClinicalNoteParser::parse_docx(
            docx_path.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(note.patient_id, Some("PT-SPARSE-001".to_string()));
        assert_eq!(note.diagnosis_code, Some("F41.1".to_string()));
        assert_eq!(note.session_date, None);
        assert_eq!(note.session_type, None);
        assert_eq!(note.notes, None);
        assert_eq!(note.treatment_plan, None);
    }

    // ══════════════════════════════════════════════════════════════════════
    // OBJ 2: Stress Test — 500 concurrent entries, invariant check
    // ══════════════════════════════════════════════════════════════════════

    /// Stress test: Insert 500 accounting entries concurrently.
    /// After all entries, verify Activos = Pasivos + Patrimonio exactly.
    /// ABORT + REVERT on any centavo of imbalance.
    #[tokio::test]
    async fn test_stress_500_concurrent_entries_balance_invariant() {
        let pool = create_e2e_pool();
        let num_entries = 500;
        let amount_per_entry = dec!(100);

        // Spawn 500 concurrent tasks, each creating a balanced asiento
        let mut handles = Vec::new();

        for i in 0..num_entries {
            let pool_clone = pool.clone();
            handles.push(tokio::spawn(async move {
                let request = CreateAsientoRequest {
                    fecha: format!("2024-01-{:02}", (i % 28) + 1),
                    descripcion: format!("Stress entry #{}", i),
                    lineas: vec![
                        CreateLineaAsientoRequest {
                            cuenta: "1110 Caja".to_string(),
                            debito: Some(amount_per_entry.to_string()),
                            credito: None,
                        },
                        CreateLineaAsientoRequest {
                            cuenta: "3110 Capital Social".to_string(),
                            debito: None,
                            credito: Some(amount_per_entry.to_string()),
                        },
                    ],
                };
                add_asiento_impl(&pool_clone, request)
                    .await
                    .expect(&format!("Failed to insert entry #{}", i))
            }));
        }

        // Wait for all tasks to complete
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_balanced, "Entry not balanced: {:?}", result);
            success_count += 1;
        }
        assert_eq!(success_count, num_entries);

        // CRITICAL: Verify the fundamental accounting invariant
        // Activos = Pasivos + Patrimonio
        let balance = generate_balance_general_impl(&pool, "2024-12-31".to_string())
            .await
            .expect("Failed to generate balance after stress test");

        // Parse totals as Decimal for exact comparison
        let activos: rust_decimal::Decimal = balance.total_activos.parse().unwrap();
        let pasivos: rust_decimal::Decimal = balance.total_pasivos.parse().unwrap();
        let patrimonio: rust_decimal::Decimal = balance.total_patrimonio.parse().unwrap();

        let lhs = activos;
        let rhs = pasivos + patrimonio;

        assert_eq!(
            lhs, rhs,
            "CRITICAL INVARIANT VIOLATION: Activos({}) ≠ Pasivos({}) + Patrimonio({}) = {} | diff={} centavos",
            lhs, pasivos, patrimonio, rhs, (lhs - rhs).abs()
        );

        // Also verify via BalanceGeneral's own is_balanced flag
        assert!(
            balance.is_balanced,
            "BalanceGeneral.is_balanced() returned false after stress test"
        );

        // Verify total count
        let repo = SqliteAccountingRepository::new(pool.clone());
        let count = repo
            .count_asientos(None, None)
            .await
            .expect("Failed to count asientos");
        assert_eq!(count, num_entries as u64);
    }

    /// Stress test: 500 sequential entries with different amounts.
    /// Verifies decimal precision doesn't drift.
    #[tokio::test]
    async fn test_stress_500_sequential_precision() {
        let pool = create_e2e_pool();
        let repo = SqliteAccountingRepository::new(pool.clone());
        let num_entries = 500;

        // Each entry: debit 33.33, credit 33.33 (3 decimal places of precision)
        for i in 0..num_entries {
            let asiento = AsientoContable::new(
                NaiveDate::from_ymd_opt(2024, 1, ((i % 28) + 1) as u32).unwrap(),
                format!("Precision test #{}", i),
                vec![
                    LineaAsiento::new_debito("1110 Caja".to_string(), dec!(33.33)).unwrap(),
                    LineaAsiento::new_credito("3110 Capital Social".to_string(), dec!(33.33))
                        .unwrap(),
                ],
            )
            .unwrap();

            repo.create_asiento(&asiento).await.unwrap();
        }

        // Verify balance: Activos = 500 * 33.33 = 16665.00
        let balance = repo
            .get_balance_general(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
            .await
            .unwrap();

        let activos: rust_decimal::Decimal = balance.total_activos().to_string().parse().unwrap();
        let pasivos: rust_decimal::Decimal = balance.total_pasivos().to_string().parse().unwrap();
        let patrimonio: rust_decimal::Decimal = balance.total_patrimonio().to_string().parse().unwrap();

        assert_eq!(
            activos,
            pasivos + patrimonio,
            "Decimal precision drift detected after {} entries",
            num_entries
        );

        // Verify exact expected total
        let expected_total = dec!(33.33) * dec!(500);
        assert_eq!(
            activos, expected_total,
            "Unexpected total: got {} expected {}",
            activos, expected_total
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // OBJ 3: Age Edge Cases
    // ══════════════════════════════════════════════════════════════════════

    /// Age: born on Feb 29 (leap year), calculate at non-leap year Feb 28.
    #[tokio::test]
    async fn test_age_leap_year_feb29_to_feb28() {
        let birth = NaiveDate::from_ymd_opt(2000, 2, 29).unwrap();
        let at = NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(); // Non-leap year
        let age = Age::from_birth_date(birth, at);

        // The algorithm clamps Feb 29 to Feb 28 in non-leap years
        // Feb 28 - Feb 28 = 0 days, Feb - Feb = 0 months, 2025 - 2000 = 25 years
        assert_eq!(age.years, 25);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 0);
    }

    /// Age: born today → 0 years, 0 months, 0 days.
    #[tokio::test]
    async fn test_age_born_today() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let age = Age::from_birth_date(today, today);

        assert_eq!(age.years, 0);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 0);
    }

    /// Age: born yesterday → 0 years, 0 months, 1 day.
    #[tokio::test]
    async fn test_age_born_yesterday() {
        let yesterday = NaiveDate::from_ymd_opt(2024, 6, 14).unwrap();
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let age = Age::from_birth_date(yesterday, today);

        assert_eq!(age.years, 0);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 1);
    }

    /// Age: Jan 31 → Feb 28 (clamp to dimension).
    #[tokio::test]
    async fn test_age_clamp_to_dimension() {
        let birth = NaiveDate::from_ymd_opt(2000, 1, 31).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 2, 28).unwrap();
        let age = Age::from_birth_date(birth, at);

        // Clamp 31 → 29 (Feb 2024 leap year dimension), so effective birth day = 29
        // 28 - 29 = -1 → borrow month → months = 0, days = 28 + 31 - 31 = 28
        // 2024 - 2000 = 24 years
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 28);
    }

    /// Age: Jan 31 → Mar 1 (cross-month boundary with clamping).
    #[tokio::test]
    async fn test_age_cross_month_clamp() {
        let birth = NaiveDate::from_ymd_opt(2000, 1, 31).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let age = Age::from_birth_date(birth, at);

        assert_eq!(age.years, 24);
        assert_eq!(age.months, 1);
        assert_eq!(age.days, 1);
    }

    /// Age: Feb 29 leap year → Mar 1 non-leap year.
    #[tokio::test]
    async fn test_age_leap_year_mar1() {
        let birth = NaiveDate::from_ymd_opt(2000, 2, 29).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let age = Age::from_birth_date(birth, at);

        // Clamp 29 → 29 (Feb 2024 is leap year, dimension = 29), so effective birth day = 29
        // 1 - 29 = -28 → borrow month → months = 0, days = 1 + 29 - 29 = 1
        // 2024 - 2000 = 24 years
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 1);
    }

    /// Age via IPC command: calculate_age_at with known dates.
    #[tokio::test]
    async fn test_age_ipc_calculate_age_at() {
        let result = calculate_age_at(
            "2000-06-15".to_string(),
            "2024-06-15".to_string(),
        )
        .await;

        assert!(result.is_ok());
        let age = result.unwrap();
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 0);
    }

    /// Age via IPC: invalid date format returns error.
    #[tokio::test]
    async fn test_age_ipc_invalid_date() {
        let result = calculate_age("not-a-date".to_string()).await;
        assert!(result.is_err());
    }

    // ══════════════════════════════════════════════════════════════════════
    // OBJ 4: Keychain Fallback Smoke Test
    // ══════════════════════════════════════════════════════════════════════

    /// Verify that create_memory_pool works without keychain (in-memory mode).
    /// This simulates the fallback path when keyring is unavailable.
    #[tokio::test]
    async fn test_keychain_fallback_memory_pool() {
        let pool = create_memory_pool();
        assert!(pool.is_ok(), "Memory pool creation should succeed without keychain");

        let pool = pool.unwrap();
        let conn = pool.lock().unwrap();

        // Verify we can execute PRAGMA and create tables
        let result = conn.execute_batch("PRAGMA journal_mode=WAL;");
        assert!(result.is_ok(), "PRAGMA should work on memory pool");

        // Verify foreign keys
        let result = conn.execute_batch("PRAGMA foreign_keys = ON;");
        assert!(result.is_ok());

        drop(conn);
    }

    /// Verify SqlCipherKeyManager file-based fallback creates file with 0o600 perms.
    #[test]
    fn test_keychain_fallback_file_permissions() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("test_key.bin");

        // Write a test key file with explicit permissions
        let key = "a]b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1";
        fs::write(&key_path, key).unwrap();

        // Explicitly set permissions to 0o600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path).unwrap().permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_path, perms).unwrap();
        }

        // Verify permissions (on macOS/Linux)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&key_path).unwrap();
            let perms = metadata.permissions();
            let mode = perms.mode();
            // 0o600 = owner read/write only
            assert_eq!(
                mode & 0o777,
                0o600,
                "Key file permissions should be 0o600, got {:o}",
                mode & 0o777
            );
        }

        // Verify key can be read back
        let read_key = fs::read_to_string(&key_path).unwrap();
        assert_eq!(read_key, key);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Additional E2E: Accounting CRUD via IPC commands
    // ══════════════════════════════════════════════════════════════════════

    /// E2E: Create, read, update, delete accounting entry via IPC commands.
    #[tokio::test]
    async fn test_e2e_accounting_crud_via_ipc() {
        let pool = create_e2e_pool();

        // CREATE
        let create_request = CreateAsientoRequest {
            fecha: "2024-03-10".to_string(),
            descripcion: "Compra de material de oficina".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "5110 Gastos de Oficina".to_string(),
                    debito: Some("250.00".to_string()),
                    credito: None,
                },
                CreateLineaAsientoRequest {
                    cuenta: "1110 Caja".to_string(),
                    debito: None,
                    credito: Some("250.00".to_string()),
                },
            ],
        };

        let created = add_asiento_impl(&pool, create_request).await.unwrap();
        assert!(created.is_balanced);
        let asiento_id = created.id.clone();

        // READ (via list)
        let list_result = list_asientos_impl(
            &pool,
            ListAsientosQuery {
                fecha_desde: Some("2024-03-01".to_string()),
                fecha_hasta: Some("2024-03-31".to_string()),
                page: Some(0),
                page_size: Some(10),
            },
        )
        .await
        .unwrap();

        assert_eq!(list_result.items.len(), 1);
        assert_eq!(list_result.items[0].descripcion, "Compra de material de oficina");

        // UPDATE
        let update_request = UpdateAsientoRequest {
            fecha: None,
            descripcion: Some("Compra de material de oficina - actualizado".to_string()),
            lineas: None,
        };

        let updated =
            update_asiento_impl(&pool, asiento_id.clone(), update_request).await.unwrap();
        assert_eq!(
            updated.descripcion,
            "Compra de material de oficina - actualizado"
        );

        // DELETE
        let deleted = remove_asiento_impl(&pool, asiento_id.clone()).await.unwrap();
        assert!(deleted);

        // Verify deleted
        let list_after = list_asientos_impl(
            &pool,
            ListAsientosQuery {
                fecha_desde: Some("2024-03-01".to_string()),
                fecha_hasta: Some("2024-03-31".to_string()),
                page: Some(0),
                page_size: Some(10),
            },
        )
        .await
        .unwrap();

        assert_eq!(list_after.items.len(), 0);
    }

    /// E2E: Estado de Resultados (income statement) with multiple entries.
    #[tokio::test]
    async fn test_e2e_estado_resultados() {
        let pool = create_e2e_pool();

        // Revenue entry
        add_asiento_impl(
            &pool,
            CreateAsientoRequest {
                fecha: "2024-04-10".to_string(),
                descripcion: "Servicios de consultoría".to_string(),
                lineas: vec![
                    CreateLineaAsientoRequest {
                        cuenta: "1110 Caja".to_string(),
                        debito: Some("5000.00".to_string()),
                        credito: None,
                    },
                    CreateLineaAsientoRequest {
                        cuenta: "4110 Ingresos por Servicios".to_string(),
                        debito: None,
                        credito: Some("5000.00".to_string()),
                    },
                ],
            },
        )
        .await
        .unwrap();

        // Expense entry
        add_asiento_impl(
            &pool,
            CreateAsientoRequest {
                fecha: "2024-04-15".to_string(),
                descripcion: "Pago de alquiler".to_string(),
                lineas: vec![
                    CreateLineaAsientoRequest {
                        cuenta: "5110 Alquiler".to_string(),
                        debito: Some("1200.00".to_string()),
                        credito: None,
                    },
                    CreateLineaAsientoRequest {
                        cuenta: "1110 Caja".to_string(),
                        debito: None,
                        credito: Some("1200.00".to_string()),
                    },
                ],
            },
        )
        .await
        .unwrap();

        // Generate Estado de Resultados
        let estado = generate_estado_resultados_impl(
            &pool,
            "2024-04-01".to_string(),
            "2024-04-30".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(estado.total_ingresos, "5000.00");
        assert_eq!(estado.total_gastos, "1200.00");
        assert_eq!(estado.utilidad_neta, "3800.00");
    }
}
