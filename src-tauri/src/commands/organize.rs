use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use chrono::{Local, Datelike};
use crate::models::{MoveOp, CategoryRule};
use crate::state::AppState;
use crate::utils::{is_excluded, get_category_for_folder};

/// 今日のファイルを特定のフォルダに整理するコマンド
#[tauri::command]
pub fn organize_today_files(state: State<'_, AppState>, today_prefix: String, folder_name: String, excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let now = Local::now();
    let date_str = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());
    let target_folder_name = format!("{}{}-{}", today_prefix, date_str, folder_name);
    let target_folder = desktop.join(&target_folder_name);

    let start_of_today = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap()
        .and_local_timezone(Local).unwrap()
        .with_timezone(&chrono::Utc);
    let start_of_today_st = SystemTime::from(start_of_today);

    let mut moved_count = 0;
    let mut batch = Vec::new();
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    let mut to_move = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if is_excluded(&path, &excluded) { continue; }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext == "lnk" || ext == "url" { continue; }
            let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
            let modified = metadata.modified().map_err(|e| e.to_string())?;
            if modified >= start_of_today_st { to_move.push(path); }
        }
    }

    if !target_folder.exists() {
        fs::create_dir(&target_folder).map_err(|e| e.to_string())?;
    }

    if !to_move.is_empty() {
        for path in to_move {
            let file_name = path.file_name().ok_or("無効なファイル名")?;
            let mut target_path = target_folder.join(file_name);
            if target_path.exists() {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
                let new_name = if ext.is_empty() { format!("{}_{}", stem, now_ms) } else { format!("{}_{}.{}", stem, now_ms, ext) };
                target_path = target_folder.join(new_name);
            }
            let original_path = path.clone();
            fs::rename(&path, &target_path).map_err(|e| e.to_string())?;
            batch.push(MoveOp { from: original_path, to: target_path });
            moved_count += 1;
        }
        let mut history = state.history.lock().unwrap();
        history.batches.push(batch);
    }
    Ok(moved_count)
}

/// 拡張子ごとにファイルを整理するコマンド
#[tauri::command]
pub fn organize_files(state: State<'_, AppState>, prefix: String, excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let mut moved_count = 0;
    let mut batch = Vec::new();
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if is_excluded(&path, &excluded) { continue; }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext == "lnk" || ext == "url" { continue; }
            let folder_name = if ext.is_empty() { format!("{}NO_EXTENSION", prefix) } else { format!("{}{}", prefix, ext.to_uppercase()) };
            let target_folder = desktop.join(&folder_name);
            if !target_folder.exists() { fs::create_dir(&target_folder).map_err(|e| e.to_string())?; }
            let file_name_os = path.file_name().ok_or("無効なファイル名")?;
            let mut target_path = target_folder.join(file_name_os);
            if target_path.exists() {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
                let new_name = if ext.is_empty() { format!("{}_{}", stem, now) } else { format!("{}_{}.{}", stem, now, ext) };
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

/// フォルダを内容に応じて分類するコマンド
#[tauri::command]
pub fn classify_folders(state: State<'_, AppState>, prefix: String, excluded: Vec<String>, rules: Vec<CategoryRule>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let mut moved_count = 0;
    let mut batch = Vec::new();
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, &excluded) { continue; }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with(&prefix) { continue; }
            let category = get_category_for_folder(&path, &prefix, &rules)?;
            let target_folder = desktop.join(category);
            if !target_folder.exists() { fs::create_dir(&target_folder).map_err(|e| e.to_string())?; }
            let mut target_path = target_folder.join(name);
            if target_path.exists() {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
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
