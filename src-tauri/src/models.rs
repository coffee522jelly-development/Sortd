use serde::Serialize;
use std::path::PathBuf;

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
pub struct MoveOp {
    pub from: PathBuf,
    pub to: PathBuf,
}
