#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use soft_gloria_commands::patient_commands::*;
use soft_gloria_commands::accounting_commands::*;
use soft_gloria_commands::diagnostics_commands::*;
use soft_gloria_commands::age_commands::*;
use soft_gloria_infrastructure::{create_pool, run_migrations, run_accounting_migrations, run_diagnostics_migrations};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let data_dir = app.path().app_data_dir().map_err(|e| {
                eprintln!("[MindLedger] Failed to get app data dir: {}", e);
                e
            })?;
            std::fs::create_dir_all(&data_dir).map_err(|e| {
                eprintln!("[MindLedger] Failed to create app data dir: {}", e);
                e
            })?;
            let db_path = data_dir.join("mind_ledger.db");
            
            tauri::async_runtime::block_on(async move {
                let db = create_pool(&db_path, &data_dir).map_err(|e| {
                    eprintln!("[MindLedger] Failed to initialize database: {}", e);
                    e
                })?;
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
