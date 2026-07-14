//! Business Logic Abuse Security Tests
//!
//! Validates that accounting and appointment commands reject malformed DTOs,
//! negative/overflow amounts, imbalanced entries, and invalid state transitions
//! atomically — with no partial database writes.

#[cfg(test)]
mod business_logic_tests {
    use crate::accounting_commands::*;
    use crate::agenda_commands::*;
    use crate::error::AppError;
    use soft_mindledger_domain::appointment::{AppointmentStatus, Modality};
    use soft_mindledger_infrastructure::database::create_memory_pool;
    use soft_mindledger_infrastructure::DbPool;
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    /// Shared test pool with full schema for business logic tests.
    fn create_bl_pool() -> DbPool {
        let pool = create_memory_pool().expect("Failed to create memory pool");
        soft_mindledger_infrastructure::migrations::run_all_migrations(&pool)
            .expect("Failed to run migrations");
        pool
    }

    /// Create a test patient in the DB.
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
                    "BL-TEST-001",
                    "DNI",
                    "EC",
                    "Business",
                    "Logic",
                    "1990-01-01",
                    "Male",
                    1,
                ],
            ).unwrap();
        }).await.unwrap();
        patient_id
    }

    /// Count asientos in the database.
    fn count_asientos(pool: &DbPool) -> i64 {
        let conn = pool.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM asientos_contables",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
    }

    // ══════════════════════════════════════════════════════════════════════
    // Transaction Amount Validation
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: business-logic-abuse/negative-transaction-amount/scenario-1
    /// Negative transaction amount — DTO with negative `lineas[].monto`
    /// must be rejected atomically, no DB write.
    #[tokio::test]
    async fn test_negative_transaction_amount_rejected() {
        let pool = create_bl_pool();
        let asientos_before = count_asientos(&pool);

        let request = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Negative amount test".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "1110 Caja".to_string(),
                    debito: Some("-500".to_string()),
                    credito: None,
                },
                CreateLineaAsientoRequest {
                    cuenta: "4110 Capital".to_string(),
                    debito: None,
                    credito: Some("500".to_string()),
                },
            ],
        };

        let result = add_asiento_impl(&pool, request).await;

        // Must be rejected
        assert!(result.is_err(), "Negative amount must be rejected");
        match result.unwrap_err() {
            AppError::Validation(msg) => {
                assert!(
                    msg.contains("positive") || msg.contains("negative") || msg.contains("Invalid"),
                    "Error should mention amount validation, got: {}",
                    msg
                );
            }
            AppError::Accounting(msg) => {
                assert!(
                    msg.contains("inválido") || msg.contains("Invalid") || msg.contains("monto"),
                    "Accounting error should mention invalid amount, got: {}",
                    msg
                );
            }
            other => panic!("Expected Validation or Accounting error, got: {:?}", other),
        }

        // Verify no database write occurred
        let asientos_after = count_asientos(&pool);
        assert_eq!(
            asientos_before, asientos_after,
            "No database write should occur for rejected negative amount"
        );
    }

    /// Spec: business-logic-abuse/overflow-amount/scenario-1
    /// Overflow amount — amount exceeding i64::MAX must return validation
    /// error, DB unchanged.
    #[tokio::test]
    async fn test_overflow_amount_rejected() {
        let pool = create_bl_pool();
        let asientos_before = count_asientos(&pool);

        let overflow_str = "99999999999999999999999999999".to_string();
        let request = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Overflow amount test".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "1110 Caja".to_string(),
                    debito: Some(overflow_str.clone()),
                    credito: None,
                },
                CreateLineaAsientoRequest {
                    cuenta: "4110 Capital".to_string(),
                    debito: None,
                    credito: Some(overflow_str),
                },
            ],
        };

        let result = add_asiento_impl(&pool, request).await;

        // Must be rejected (Decimal parse overflow or validation)
        assert!(result.is_err(), "Overflow amount must be rejected");

        // Verify no database write
        let asientos_after = count_asientos(&pool);
        assert_eq!(
            asientos_before, asientos_after,
            "No database write should occur for overflow amount"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Accounting Equation Invariant
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: business-logic-abuse/debit-credit-imbalance/scenario-1
    /// Debit-credit imbalance — `sum(debito) != sum(credito)` must be
    /// rejected, accounting equation preserved, no partial writes.
    #[tokio::test]
    async fn test_debit_credit_imbalance_rejected() {
        let pool = create_bl_pool();
        let asientos_before = count_asientos(&pool);

        // Debit 1000, Credit 500 → imbalanced
        let request = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Imbalanced entry test".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "1110 Caja".to_string(),
                    debito: Some("1000".to_string()),
                    credito: None,
                },
                CreateLineaAsientoRequest {
                    cuenta: "4110 Capital".to_string(),
                    debito: None,
                    credito: Some("500".to_string()),
                },
            ],
        };

        let result = add_asiento_impl(&pool, request).await;

        // Must be rejected — AsientoContable::new validates balance
        assert!(result.is_err(), "Imbalanced asiento must be rejected");
        match result.unwrap_err() {
            AppError::Accounting(msg) => {
                assert!(
                    msg.contains("Balance") || msg.contains("balance") || msg.contains("desbalanceado") || msg.contains("不平衡"),
                    "Error should mention balance, got: {}",
                    msg
                );
            }
            other => panic!("Expected Accounting error for imbalance, got: {:?}", other),
        }

        // Verify no database write
        let asientos_after = count_asientos(&pool);
        assert_eq!(
            asientos_before, asientos_after,
            "No database write should occur for imbalanced asiento"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Appointment State Machine Integrity
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: business-logic-abuse/invalid-state-transition/scenario-1
    /// Invalid state transition — "Cancelada" → "Realizada" must be rejected.
    #[tokio::test]
    async fn test_invalid_transition_cancelada_to_realizada() {
        let pool = create_bl_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();

        // Create appointment
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };

        let appointment = crear_cita_agenda_impl(&pool, request).await.unwrap();

        // Cancel the appointment
        let cancelled = cancelar_cita_impl(
            &pool,
            appointment.id.clone(),
            "Paciente canceló".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(cancelled.status, AppointmentStatus::Cancelada);

        // Now try to finalize (Cancelada → Realizada) — must fail
        let result = finalizar_sesion_agenda_impl(&pool, appointment.id, None).await;

        assert!(
            result.is_err(),
            "Cannot transition from Cancelada to Realizada"
        );
        match result.unwrap_err() {
            AppError::Validation(msg) => {
                assert!(
                    msg.contains("Cannot transition") || msg.contains("transition") || msg.contains("Cancelada"),
                    "Error should mention invalid transition, got: {}",
                    msg
                );
            }
            other => panic!("Expected Validation error for invalid transition, got: {:?}", other),
        }
    }

    /// Spec: business-logic-abuse/terminal-state-reentry/scenario-1
    /// Terminal state re-entry — "Realizada" → any state must be rejected.
    #[tokio::test]
    async fn test_terminal_state_reentry_realizada() {
        let pool = create_bl_pool();
        let patient_id = create_test_patient(&pool).await;
        let therapist_id = Uuid::new_v4();

        // Create and finalize an appointment (→ Realizada)
        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);
        let request = CreateAppointmentRequest {
            patient_id: patient_id.to_string(),
            therapist_id: therapist_id.to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };

        let appointment = crear_cita_agenda_impl(&pool, request).await.unwrap();
        let _finalized = finalizar_sesion_agenda_impl(
            &pool,
            appointment.id.clone(),
            Some("Sesión completada".to_string()),
        )
        .await
        .unwrap();

        // Try to cancel (Realizada → Cancelada) — must fail (terminal state)
        let cancel_result = cancelar_cita_impl(
            &pool,
            appointment.id.clone(),
            "Trying to cancel after finalize".to_string(),
        )
        .await;
        assert!(
            cancel_result.is_err(),
            "Cannot transition from Realizada (terminal) to Cancelada"
        );

        // Try to reschedule (Realizada → Reagendada) — must fail
        let new_start = start + Duration::days(1);
        let new_end = end + Duration::days(1);
        let reschedule_result = reagendar_cita_impl(
            &pool,
            appointment.id.clone(),
            new_start.to_rfc3339(),
            new_end.to_rfc3339(),
            "Trying to reschedule after finalize".to_string(),
        )
        .await;
        assert!(
            reschedule_result.is_err(),
            "Cannot transition from Realizada (terminal) to Reagendada"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Missing Required Fields in DTOs
    // ══════════════════════════════════════════════════════════════════════

    /// Spec: business-logic-abuse/missing-required-fields/scenario-1
    /// Missing required fields — DTOs with empty/missing mandatory fields
    /// must return validation errors.
    #[tokio::test]
    async fn test_missing_required_fields_asiento() {
        let pool = create_bl_pool();
        let asientos_before = count_asientos(&pool);

        // Empty lineas
        let request_empty_lines = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Empty lines test".to_string(),
            lineas: vec![],
        };
        let result = add_asiento_impl(&pool, request_empty_lines).await;
        assert!(result.is_err(), "Empty lineas must be rejected");

        // Empty description
        let request_empty_desc = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "   ".to_string(), // whitespace only
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
        let result = add_asiento_impl(&pool, request_empty_desc).await;
        assert!(result.is_err(), "Empty description must be rejected");

        // Invalid date format
        let request_bad_date = CreateAsientoRequest {
            fecha: "not-a-date".to_string(),
            descripcion: "Bad date test".to_string(),
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
        let result = add_asiento_impl(&pool, request_bad_date).await;
        assert!(result.is_err(), "Invalid date must be rejected");

        // Empty account name
        let request_empty_account = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Empty account test".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest {
                    cuenta: "".to_string(),
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
        let result = add_asiento_impl(&pool, request_empty_account).await;
        assert!(result.is_err(), "Empty account name must be rejected");

        // Verify no database writes from any of the above
        let asientos_after = count_asientos(&pool);
        assert_eq!(
            asientos_before, asientos_after,
            "No database writes should occur for invalid DTOs"
        );
    }

    /// Spec: business-logic-abuse/missing-required-fields/scenario-2
    /// Appointment with missing/invalid patient_id must be rejected.
    #[tokio::test]
    async fn test_missing_required_fields_appointment() {
        let pool = create_bl_pool();

        let start = Utc::now() + Duration::hours(2);
        let end = start + Duration::minutes(50);

        // Invalid patient_id (not a UUID)
        let request_bad_patient = CreateAppointmentRequest {
            patient_id: "not-a-uuid".to_string(),
            therapist_id: Uuid::new_v4().to_string(),
            start_at: start.to_rfc3339(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };
        let result = crear_cita_agenda_impl(&pool, request_bad_patient).await;
        assert!(result.is_err(), "Invalid patient_id must be rejected");

        // Invalid date format
        let request_bad_date = CreateAppointmentRequest {
            patient_id: Uuid::new_v4().to_string(),
            therapist_id: Uuid::new_v4().to_string(),
            start_at: "not-a-date".to_string(),
            end_at: end.to_rfc3339(),
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };
        let result = crear_cita_agenda_impl(&pool, request_bad_date).await;
        assert!(result.is_err(), "Invalid start_at must be rejected");

        // end_at before start_at
        let request_reversed = CreateAppointmentRequest {
            patient_id: Uuid::new_v4().to_string(),
            therapist_id: Uuid::new_v4().to_string(),
            start_at: end.to_rfc3339(),
            end_at: start.to_rfc3339(), // before start
            modality: Modality::Presencial,
            fee_cents: Some(50000),
            notes: None,
        };
        let result = crear_cita_agenda_impl(&pool, request_reversed).await;
        assert!(result.is_err(), "Reversed dates must be rejected");
    }
}
