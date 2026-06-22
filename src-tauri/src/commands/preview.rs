use std::fs;
use crate::models::{FilePreview, FolderPreview, CategoryRule};
use crate::utils::{is_excluded, get_category_for_folder};

/// 拡張子ごとの整理のプレビューを取得するコマンド
#[tauri::command]
pub fn get_organization_preview(prefix: String, excluded: Vec<String>) -> Result<Vec<FilePreview>, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let mut preview = Vec::new();
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if is_excluded(&path, &excluded) { continue; }
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext == "lnk" || ext == "url" { continue; }
            let folder_name = if ext.is_empty() { format!("{}NO_EXTENSION", prefix) } else { format!("{}{}", prefix, ext.to_uppercase()) };
            preview.push(FilePreview { filename: file_name.to_string(), target_dir: folder_name });
        }
    }
    Ok(preview)
}

/// フォルダ分類のプレビューを取得するコマンド
#[tauri::command]
pub fn get_classification_preview(prefix: String, excluded: Vec<String>, rules: Vec<CategoryRule>) -> Result<Vec<FolderPreview>, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let mut preview = Vec::new();
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if is_excluded(&path, &excluded) { continue; }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with(&prefix) { continue; }
            let category = get_category_for_folder(&path, &prefix, &rules)?;
            preview.push(FolderPreview { folder_name: name.to_string(), category });
        }
    }
    Ok(preview)
}
