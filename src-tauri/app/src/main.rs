#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use soft_mindledger_commands::patient_commands::*;
use soft_mindledger_commands::accounting_commands::*;
use soft_mindledger_commands::diagnostics_commands::*;
use soft_mindledger_commands::age_commands::*;
use soft_mindledger_commands::agenda_commands::*;
use soft_mindledger_commands::tenant::*;
use soft_mindledger_infrastructure::{create_pool_for_tenant, run_all_migrations};
use std::sync::Arc;
use tauri::Manager;

/// Marker file to detect first-run bootstrap completion.
/// Created after successful database initialization + migrations.
const FIRST_RUN_MARKER: &str = ".mindledger_initialized";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set TENANT_CONFIG_PATH as a process env var so commands crate can read it at runtime.
    // build.rs sets this via cargo:rustc-env (compile-time only for mind-ledger crate),
    // but commands crate can't see cargo:rustc-env from another crate.
    if let Ok(path) = std::env::var("TENANT_CONFIG_PATH") {
        // Already set as process env (e.g. from bundle-tenant.py or manual export)
    } else if let Some(compile_time_path) = option_env!("TENANT_CONFIG_PATH") {
        std::env::set_var("TENANT_CONFIG_PATH", compile_time_path);
    }

    // Propagate the embedded tenant config CONTENT (not just the path) so the commands
    // crate can fall back to it when the path points to a non-existent file (e.g. in
    // release builds where the path references the CI build machine's filesystem).
    if let Some(config_json) = option_env!("TENANT_CONFIG_JSON") {
        if std::env::var("TENANT_CONFIG_JSON").is_err() {
            std::env::set_var("TENANT_CONFIG_JSON", config_json);
        }
    }

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Get tenant config at startup (sync, compiled into binary)
            let tenant_config = get_tenant_config_cached().map_err(|e| {
                #[cfg(debug_assertions)]
                eprintln!("[MindLedger] Failed to load tenant config: {}", e);
                e
            })?;
            
            let tenant_id = tenant_config.tenant.id.clone();
            let keyring_account = tenant_config.crypto.keyringAccount.clone();
            let db_filename = tenant_config.crypto.dbFileName.clone();
            
            // Derive tenant-specific data directory
            let base_data_dir = app.path().app_data_dir().map_err(|e| {
                #[cfg(debug_assertions)]
                eprintln!("[MindLedger] Failed to get app data dir: {}", e);
                e
            })?;
            
            let data_dir = base_data_dir.join(format!("mind-ledger-{}", tenant_id));
            std::fs::create_dir_all(&data_dir).map_err(|e| {
                #[cfg(debug_assertions)]
                eprintln!("[MindLedger] Failed to create tenant data dir: {}", e);
                e
            })?;
            
            // Check if first-run bootstrap has already completed
            let first_run_marker = data_dir.join(FIRST_RUN_MARKER);
            let is_first_run = !first_run_marker.exists();
            
            tauri::async_runtime::block_on(async move {
                // Initialize database pool (creates DB file, generates/retrieves encryption key)
                let db = create_pool_for_tenant(&data_dir, &keyring_account, &db_filename)
                    .map_err(|e| {
                        #[cfg(debug_assertions)]
                        eprintln!("[MindLedger] Failed to initialize database: {}", e);
                        e
                    })?;
                
                // Run all migrations (idempotent - safe to run on every startup)
                // Uses run_all_migrations which executes all schema migrations in order
                run_all_migrations(&db).map_err(|e| {
                    #[cfg(debug_assertions)]
                    eprintln!("[MindLedger] Failed to run migrations: {}", e);
                    e
                })?;
                
                // If first run, create marker file to signal bootstrap completion
                if is_first_run {
                    std::fs::write(&first_run_marker, "initialized")
                        .map_err(|e| {
                            #[cfg(debug_assertions)]
                            eprintln!("[MindLedger] Warning: failed to write first-run marker: {}", e);
                            e
                        })
                        .ok(); // Non-critical, log only in debug
                    
                    #[cfg(debug_assertions)]
                    eprintln!("[MindLedger] First-run bootstrap completed: DB deployed, keys generated, migrations applied");
                }
                
                app_handle.manage(Arc::new(db));
                Ok::<(), Box<dyn std::error::Error>>(())
            }).unwrap_or_else(|e| {
                #[cfg(debug_assertions)]
                eprintln!("[MindLedger] Critical error during setup: {}", e);
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_patient,
            get_patient,
            list_patients,
            update_patient,
            delete_patient,
            search_patients,
            get_patient_count,
            // Accounting commands
            add_asiento,
            remove_asiento,
            list_asientos,
            generate_balance_general,
            generate_estado_resultados,
            // Diagnostics commands
            search_cie10,
            search_dsm5,
            get_cie10_by_codigo,
            get_dsm5_by_codigo,
            create_mapeo,
            list_mapeos,
            update_mapeo,
            delete_mapeo,
            // Age commands
            calculate_age,
            calculate_age_at,
            calculate_age_breakdown,
            // Agenda commands
            crear_cita_agenda,
            obtener_cita_agenda,
            listar_citas_agenda,
            finalizar_sesion_agenda,
            reagendar_cita,
            cancelar_cita,
            obtener_citas_paciente,
            obtener_recordatorios_pendientes,
            procesar_recordatorios_pendientes,
            obtener_kpis_agenda,
            // Tenant config command
            get_tenant_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
