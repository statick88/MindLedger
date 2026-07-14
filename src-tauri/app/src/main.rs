#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use soft_gloria_commands::patient_commands::*;
use soft_gloria_commands::accounting_commands::*;
use soft_gloria_commands::diagnostics_commands::*;
use soft_gloria_commands::age_commands::*;
use soft_gloria_commands::agenda_commands::*;
use soft_gloria_commands::tenant::*;
use soft_gloria_infrastructure::{create_pool_for_tenant, run_migrations, run_accounting_migrations, run_diagnostics_migrations, run_agenda_migrations};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Get tenant config at startup (sync, compiled into binary)
            let tenant_config = get_tenant_config_cached().map_err(|e| {
                eprintln!("[MindLedger] Failed to load tenant config: {}", e);
                e
            })?;
            
            let tenant_id = tenant_config.tenant.id.clone();
            let keyring_account = tenant_config.crypto.keyringAccount.clone();
            let db_filename = tenant_config.crypto.dbFileName.clone();
            
            // Derive tenant-specific data directory
            let base_data_dir = app.path().app_data_dir().map_err(|e| {
                eprintln!("[MindLedger] Failed to get app data dir: {}", e);
                e
            })?;
            
            let data_dir = base_data_dir.join(format!("mind-ledger-{}", tenant_id));
            std::fs::create_dir_all(&data_dir).map_err(|e| {
                eprintln!("[MindLedger] Failed to create tenant data dir: {}", e);
                e
            })?;
            
            tauri::async_runtime::block_on(async move {
                let db = create_pool_for_tenant(&data_dir, &keyring_account, &db_filename)
                    .map_err(|e| {
                        eprintln!("[MindLedger] Failed to initialize database: {}", e);
                        e
                    })?;
                
                // Run migrations...
                run_migrations(&db).map_err(|e| {
                    eprintln!("[MindLedger] Failed to run migrations: {}", e);
                    e
                })?;
                run_accounting_migrations(&db).map_err(|e| {
                    eprintln!("[MindLedger] Failed to run accounting migrations: {}", e);
                    e
                })?;
                run_diagnostics_migrations(&db).map_err(|e| {
                    eprintln!("[MindLedger] Failed to run diagnostics migrations: {}", e);
                    e
                })?;
                run_agenda_migrations(&db).map_err(|e| {
                    eprintln!("[MindLedger] Failed to run agenda migrations: {}", e);
                    e
                })?;
                
                app_handle.manage(Arc::new(db));
                Ok::<(), Box<dyn std::error::Error>>(())
            }).unwrap_or_else(|e| {
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
