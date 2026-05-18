// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashSet;
use std::fs;
use std::time::SystemTime;
use tauri::Manager;

#[tauri::command]
fn delete_new_folders() -> Result<String, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut deleted_count = 0;

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("新しいフォルダー") {
                    fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
                    deleted_count += 1;
                }
            }
        }
    }
    Ok(format!("{}個のフォルダを削除しました。", deleted_count))
}

#[tauri::command]
fn organize_files() -> Result<String, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut moved_count = 0;

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            // Skip shortcuts
            if ext == "lnk" || ext == "url" {
                continue;
            }

            let folder_name = if ext.is_empty() {
                "◇NO_EXTENSION".to_string()
            } else {
                format!("◇{}", ext.to_uppercase())
            };

            let target_folder = desktop.join(&folder_name);
            if !target_folder.exists() {
                fs::create_dir(&target_folder).map_err(|e| e.to_string())?;
            }

            let file_name = path.file_name().ok_or("Invalid file name")?;
            let mut target_path = target_folder.join(file_name);

            if target_path.exists() {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
                let new_name = if ext.is_empty() {
                    format!("{}_{}", stem, now)
                } else {
                    format!("{}_{}.{}", stem, now, ext)
                };
                target_path = target_folder.join(new_name);
            }

            fs::rename(&path, &target_path).map_err(|e| e.to_string())?;
            moved_count += 1;
        }
    }
    Ok(format!("{}個のファイルを整理しました。", moved_count))
}

#[tauri::command]
fn classify_folders() -> Result<String, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut moved_count = 0;

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('◇') {
                continue;
            }

            let mut exts = HashSet::new();
            let sub_entries = fs::read_dir(&path).map_err(|e| e.to_string())?;
            for sub_entry in sub_entries {
                let sub_entry = sub_entry.map_err(|e| e.to_string())?;
                let sub_path = sub_entry.path();
                if sub_path.is_file() {
                    if let Some(ext) = sub_path.extension().and_then(|e| e.to_str()) {
                        exts.insert(ext.to_lowercase());
                    }
                }
            }

            let web_exts = ["html", "htm", "css", "js", "ts", "jsx", "tsx", "php", "vue", "scss"];
            let unity_exts = ["unity", "prefab", "asset"];
            let python_exts = ["py", "ipynb"];
            let design_exts = ["psd", "ai", "xd", "fig", "sketch"];
            let doc_exts = ["docx", "pptx", "pdf", "csv"];
            let prog_exts = ["c", "cpp", "h", "hpp", "cs", "java", "go", "rs", "rb"];

            let has_ext = |list: &[&str]| list.iter().any(|e| exts.contains(*e));

            let category = if has_ext(&web_exts) {
                "◇WebProject"
            } else if has_ext(&unity_exts) {
                "◇UnityProject"
            } else if has_ext(&python_exts) {
                "◇PythonProject"
            } else if has_ext(&design_exts) {
                "◇DesignProject"
            } else if has_ext(&doc_exts) {
                "◇DocumentProject"
            } else if has_ext(&prog_exts) {
                "◇ProgrammingProject"
            } else if (exts.contains("xlsx") || exts.contains("xls")) && exts.contains("png") {
                "◇ProjectWorking"
            } else if exts.contains("txt") && exts.len() == 1 {
                "◇Memo"
            } else {
                "◇NoCategories"
            };

            let target_folder = desktop.join(category);
            if !target_folder.exists() {
                fs::create_dir(&target_folder).map_err(|e| e.to_string())?;
            }

            let mut target_path = target_folder.join(name);
            if target_path.exists() {
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
                target_path = target_folder.join(format!("{}_{}", name, now));
            }

            fs::rename(&path, &target_path).map_err(|e| e.to_string())?;
            moved_count += 1;
        }
    }
    Ok(format!("{}個のフォルダを分類しました。", moved_count))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_menu(None);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            delete_new_folders,
            organize_files,
            classify_folders
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
