// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sortd_lib::state::AppState;
use sortd_lib::setup::setup;
use sortd_lib::commands::organize::*;
use sortd_lib::commands::preview::*;
use sortd_lib::commands::system::*;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            delete_new_folders,
            get_organization_preview,
            organize_files,
            get_classification_preview,
            classify_folders,
            undo_last_operation,
            empty_recycle_bin,
            organize_today_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
