use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Manager, State, AppHandle, Runtime};
use serde::Serialize;
use chrono::{Local, Datelike};

/// ファイル整理のプレビュー用データ構造
#[derive(Serialize, Clone)]
pub struct FilePreview {
    pub filename: String,
    pub target_dir: String,
}

/// フォルダ分類のプレビュー用データ構造
#[derive(Serialize, Clone)]
pub struct FolderPreview {
    pub folder_name: String,
    pub category: String,
}

/// 単一の移動操作を記録する構造体（Undo用）
struct MoveOp {
    from: PathBuf,
    to: PathBuf,
}

/// 移動操作の履歴をバッチ単位で管理
struct History {
    batches: Vec<Vec<MoveOp>>,
}

/// アプリケーションの共有状態（履歴を保持）
pub struct AppState {
    pub history: Mutex<History>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(History { batches: Vec::new() }),
        }
    }
}

/// 指定されたパスが除外リストに含まれているか判定
fn is_excluded(path: &Path, excluded_paths: &[String]) -> bool {
    let path_str = path.to_string_lossy().to_string();
    excluded_paths.iter().any(|p| {
        // 完全一致、または除外フォルダ配下のパスであるかを確認
        path_str == *p || path_str.starts_with(&(p.to_owned() + std::path::MAIN_SEPARATOR.to_string().as_str()))
    })
}

/// 今日のファイルを特定のフォルダに整理するコマンド
#[tauri::command]
pub fn organize_today_files(state: State<'_, AppState>, today_prefix: String, folder_name: String, excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;

    // 今日の日付を YYMMDD 形式で取得
    let now = Local::now();
    let date_str = format!("{:02}{:02}{:02}", now.year() % 100, now.month(), now.day());

    // ターゲットフォルダ名を作成（例：▶260607-フォルダ名）
    let target_folder_name = format!("{}{}-{}", today_prefix, date_str, folder_name);
    let target_folder = desktop.join(&target_folder_name);

    // 今日の開始時刻（00:00:00）をSystemTimeで取得
    let start_of_today = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap()
        .and_local_timezone(Local).unwrap()
        .with_timezone(&chrono::Utc);
    let start_of_today_st = SystemTime::from(start_of_today);

    let mut moved_count = 0;
    let mut batch = Vec::new();

    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    // 移動対象のファイルを収集
    let mut to_move = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if is_excluded(&path, &excluded) {
                continue;
            }

            // ショートカットファイルは除外
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext == "lnk" || ext == "url" {
                continue;
            }

            // 更新日時を確認
            let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
            let modified = metadata.modified().map_err(|e| e.to_string())?;

            if modified >= start_of_today_st {
                to_move.push(path);
            }
        }
    }

    // フォルダは常に作成（ユーザーの要望：ファイルがなくても作成する）
    if !target_folder.exists() {
        fs::create_dir(&target_folder).map_err(|e| e.to_string())?;
    }

    // ファイルを移動
    if !to_move.is_empty() {
        for path in to_move {
            let file_name = path.file_name().ok_or("無効なファイル名")?;
            let mut target_path = target_folder.join(file_name);

            // 同名ファイルがある場合はタイムスタンプを付与
            if target_path.exists() {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
                let new_name = if ext.is_empty() {
                    format!("{}_{}", stem, now_ms)
                } else {
                    format!("{}_{}.{}", stem, now_ms, ext)
                };
                target_path = target_folder.join(new_name);
            }

            let original_path = path.clone();
            fs::rename(&path, &target_path).map_err(|e| e.to_string())?;
            batch.push(MoveOp { from: original_path, to: target_path });
            moved_count += 1;
        }

        // 履歴を保存
        let mut history = state.history.lock().unwrap();
        history.batches.push(batch);
    }

    Ok(moved_count)
}

/// Windowsのごみ箱を空にするコマンド（削除されたサイズを返す）
#[tauri::command]
pub fn empty_recycle_bin() -> Result<u64, String> {
    #[cfg(target_os = "windows")]
    {
        // PowerShellを使用してサイズ取得と削除を実行
        let ps_script = r#"
            $shell = New-Object -ComObject Shell.Application
            $bin = $shell.Namespace(0x0a)
            $size = ($bin.Items() | Measure-Object -Property Size -Sum).Sum
            if ($null -eq $size) { $size = 0 }
            Clear-RecycleBin -Confirm:$false -ErrorAction SilentlyContinue
            Write-Output $size
        "#;

        let output = Command::new("powershell")
            .args(["-Command", ps_script])
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let size = stdout.parse::<u64>().unwrap_or(0);

        Ok(size)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("ごみ箱を空にする機能はWindowsのみ対応しています。".to_string())
    }
}

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
            if is_excluded(&path, &excluded) {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            // ショートカットは整理対象外
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

            let file_name_os = path.file_name().ok_or("無効なファイル名")?;
            let mut target_path = target_folder.join(file_name_os);

            // 重複回避
            if target_path.exists() {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
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

/// フォルダの中身を見てプロジェクトカテゴリを判定するヘルパー
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

    // 分類ルール定義
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

/// フォルダ分類のプレビューを取得するコマンド
#[tauri::command]
pub fn get_classification_preview(prefix: String, excluded: Vec<String>) -> Result<Vec<FolderPreview>, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
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
            // 既に接頭辞が付いているフォルダはスルー
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

/// フォルダを内容に応じて分類するコマンド
#[tauri::command]
pub fn classify_folders(state: State<'_, AppState>, prefix: String, excluded: Vec<String>) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
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

/// 最後に実行した移動操作（整理・分類・日次整理）を元に戻すコマンド
#[tauri::command]
pub fn undo_last_operation(state: State<'_, AppState>) -> Result<usize, String> {
    let mut history = state.history.lock().unwrap();
    if let Some(batch) = history.batches.pop() {
        let mut undo_count = 0;
        for op in batch {
            if op.to.exists() {
                // 元の親フォルダが消えていた場合は再作成
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

/// アプリケーションのセットアップ（メニュー削除など）
pub fn setup<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.remove_menu();
        }
    }
    Ok(())
}
