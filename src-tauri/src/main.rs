// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;
use tauri::{Manager, State};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct FilePreview {
    pub filename: String,
    pub target_dir: String,
}

#[derive(Serialize, Clone)]
pub struct FolderPreview {
    pub folder_name: String,
    pub category: String,
}

struct MoveOp {
    from: PathBuf,
    to: PathBuf,
}

struct History {
    batches: Vec<Vec<MoveOp>>,
}

pub struct AppState {
    history: Mutex<History>,
}

fn is_excluded(path: &Path, excluded_paths: &[String]) -> bool {
    let path_str = path.to_string_lossy().to_string();
    excluded_paths.iter().any(|p| {
        path_str == *p || path_str.starts_with(&(p.to_owned() + std::path::MAIN_SEPARATOR.to_string().as_str()))
    })
}

#[tauri::command]
fn delete_new_folders(excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut deleted_count = 0;

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, &excluded) {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("新しいフォルダー") {
                    fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
                    deleted_count += 1;
                }
            }
        }
    }
    Ok(deleted_count)
}

#[tauri::command]
fn get_organization_preview(prefix: String, excluded: Vec<String>) -> Result<Vec<FilePreview>, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut preview = Vec::new();

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if is_excluded(&path, &excluded) {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            if ext == "lnk" || ext == "url" {
                continue;
            }

            let folder_name = if ext.is_empty() {
                format!("{}NO_EXTENSION", prefix)
            } else {
                format!("{}{}", prefix, ext.to_uppercase())
            };
            preview.push(FilePreview {
                filename: file_name.to_string(),
                target_dir: folder_name,
            });
        }
    }
    Ok(preview)
}

#[tauri::command]
fn organize_files(state: State<'_, AppState>, prefix: String, excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut moved_count = 0;
    let mut batch = Vec::new();

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if is_excluded(&path, &excluded) {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            if ext == "lnk" || ext == "url" {
                continue;
            }

            let folder_name = if ext.is_empty() {
                format!("{}NO_EXTENSION", prefix)
            } else {
                format!("{}{}", prefix, ext.to_uppercase())
            };

            let target_folder = desktop.join(&folder_name);
            if !target_folder.exists() {
                fs::create_dir(&target_folder).map_err(|e| e.to_string())?;
            }

            let file_name_os = path.file_name().ok_or("Invalid file name")?;
            let mut target_path = target_folder.join(file_name_os);

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

            let original_path = path.clone();
            fs::rename(&path, &target_path).map_err(|e| e.to_string())?;
            batch.push(MoveOp { from: original_path, to: target_path });
            moved_count += 1;
        }
    }

    if !batch.is_empty() {
        let mut history = state.history.lock().unwrap();
        history.batches.push(batch);
    }

    Ok(moved_count)
}

fn get_category_for_folder(path: &std::path::Path, prefix: &str) -> Result<String, String> {
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
        format!("{}WebProject", prefix)
    } else if has_ext(&unity_exts) {
        format!("{}UnityProject", prefix)
    } else if has_ext(&python_exts) {
        format!("{}PythonProject", prefix)
    } else if has_ext(&design_exts) {
        format!("{}DesignProject", prefix)
    } else if has_ext(&doc_exts) {
        format!("{}DocumentProject", prefix)
    } else if has_ext(&prog_exts) {
        format!("{}ProgrammingProject", prefix)
    } else if (exts.contains("xlsx") || exts.contains("xls")) && exts.contains("png") {
        format!("{}ProjectWorking", prefix)
    } else if exts.contains("txt") && exts.len() == 1 {
        format!("{}Memo", prefix)
    } else {
        format!("{}NoCategories", prefix)
    };
    Ok(category)
}

#[tauri::command]
fn get_classification_preview(prefix: String, excluded: Vec<String>) -> Result<Vec<FolderPreview>, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut preview = Vec::new();

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, &excluded) {
                continue;
            }

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with(&prefix) {
                continue;
            }
            let category = get_category_for_folder(&path, &prefix)?;
            preview.push(FolderPreview {
                folder_name: name.to_string(),
                category,
            });
        }
    }
    Ok(preview)
}

#[tauri::command]
fn classify_folders(state: State<'_, AppState>, prefix: String, excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("Could not find desktop directory")?;
    let mut moved_count = 0;
    let mut batch = Vec::new();

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, &excluded) {
                continue;
            }

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with(&prefix) {
                continue;
            }

            let category = get_category_for_folder(&path, &prefix)?;
            let target_folder = desktop.join(category);
            if !target_folder.exists() {
                fs::create_dir(&target_folder).map_err(|e| e.to_string())?;
            }

            let mut target_path = target_folder.join(name);
            if target_path.exists() {
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
                target_path = target_folder.join(format!("{}_{}", name, now));
            }

            let original_path = path.clone();
            fs::rename(&path, &target_path).map_err(|e| e.to_string())?;
            batch.push(MoveOp { from: original_path, to: target_path });
            moved_count += 1;
        }
    }

    if !batch.is_empty() {
        let mut history = state.history.lock().unwrap();
        history.batches.push(batch);
    }

    Ok(moved_count)
}

#[tauri::command]
fn undo_last_operation(state: State<'_, AppState>) -> Result<usize, String> {
    let mut history = state.history.lock().unwrap();
    if let Some(batch) = history.batches.pop() {
        let mut undo_count = 0;
        for op in batch {
            if op.to.exists() {
                if let Some(parent) = op.from.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                }
                fs::rename(&op.to, &op.from).map_err(|e| e.to_string())?;
                undo_count += 1;
            }
        }
        Ok(undo_count)
    } else {
        Err("元に戻す履歴がありません。".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            history: Mutex::new(History { batches: Vec::new() }),
        })
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.remove_menu();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            delete_new_folders,
            get_organization_preview,
            organize_files,
            get_classification_preview,
            classify_folders,
            undo_last_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
