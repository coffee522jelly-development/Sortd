pub mod models;
pub mod state;
pub mod utils;
pub mod setup;
pub mod commands;
pub mod tray;

use state::AppState;
use setup::setup;
use commands::organize::*;
use commands::preview::*;
use commands::system::*;
use commands::ai::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            organize_today_files,
            delete_duplicate_files,
            get_ai_models,
            get_ai_organization_preview,
            execute_ai_organization
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
