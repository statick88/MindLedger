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
            
            tauri::async_runtime::block_on(async move {
                let db = create_pool(std::path::Path::new("soft_gloria.db")).expect("Failed to initialize database");
                run_migrations(&db).expect("Failed to run migrations");
                run_accounting_migrations(&db).expect("Failed to run accounting migrations");
                run_diagnostics_migrations(&db).expect("Failed to run diagnostics migrations");
                
                app_handle.manage(Arc::new(db));
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
