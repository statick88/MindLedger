#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use soft_gloria_commands::patient_commands::*;
use soft_gloria_infrastructure::{create_pool, run_migrations};
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
