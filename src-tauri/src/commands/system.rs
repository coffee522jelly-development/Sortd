use std::fs;
use std::process::Command;
use regex::Regex;
use tauri::State;
use crate::state::AppState;
use crate::utils::is_excluded;

/// 「新しいフォルダー」という名前を含むフォルダを一括削除するコマンド
#[tauri::command]
pub fn delete_new_folders(excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let mut deleted_count = 0;
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, &excluded) { continue; }
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

/// Windowsのごみ箱を空にするコマンド（削除されたサイズを返す）
#[tauri::command]
pub fn empty_recycle_bin() -> Result<u64, String> {
    #[cfg(target_os = "windows")]
    {
        let ps_script = r#"
            $shell = New-Object -ComObject Shell.Application
            $bin = $shell.Namespace(0x0a)
            $size = ($bin.Items() | Measure-Object -Property Size -Sum).Sum
            if ($null -eq $size) { $size = 0 }
            Clear-RecycleBin -Confirm:$false -ErrorAction SilentlyContinue
            Write-Output $size
        "#;
        let output = Command::new("powershell").args(["-Command", ps_script]).output().map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let size = stdout.parse::<u64>().unwrap_or(0);
        Ok(size)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("ごみ箱を空にする機能はWindowsのみ対応しています。".to_string())
    }
}

/// 重複ファイル（「〜のコピー」「〜 (1)」など）を削除するコマンド
#[tauri::command]
pub fn delete_duplicate_files(excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let mut deleted_count = 0;
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    // パターン1: "ファイル名 - コピー.ext" や "ファイル名 のコピー.ext"
    // パターン2: "ファイル名 (1).ext" など
    let re_copy = Regex::new(r"^(.*?)(?:\s-\sコピー|\sのコピー|\s\(\d+\))(\.[^.]+)$").unwrap();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() {
            if is_excluded(&path, &excluded) { continue; }

            if let Some(file_name_os) = path.file_name() {
                if let Some(file_name) = file_name_os.to_str() {
                    if let Some(caps) = re_copy.captures(file_name) {
                        let base_name = caps.get(1).map_or("", |m| m.as_str());
                        let extension = caps.get(2).map_or("", |m| m.as_str());
                        let original_file_name = format!("{}{}", base_name, extension);
                        let original_path = desktop.join(&original_file_name);

                        // オリジナルのファイルが存在する場合のみ重複とみなして削除
                        if original_path.exists() && original_path.is_file() {
                            if fs::remove_file(&path).is_ok() {
                                deleted_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(deleted_count)
}

/// 最後に実行した移動操作を元に戻すコマンド
#[tauri::command]
pub fn undo_last_operation(state: State<'_, AppState>) -> Result<usize, String> {
    let mut history = state.history.lock().unwrap();
    if let Some(batch) = history.batches.pop() {
        let mut undo_count = 0;
        for op in batch {
            if op.to.exists() {
                if let Some(parent) = op.from.parent() {
                    if !parent.exists() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
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
