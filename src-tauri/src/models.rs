use serde::{Deserialize, Serialize};
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

/// カスタム分類ルール用データ構造
#[derive(Serialize, serde::Deserialize, Clone)]
pub struct CategoryRule {
    pub name: String,
    pub extensions: Vec<String>,
}

/// AI整理の入力アイテム構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiDesktopItem {
    pub r#type: String,
    pub name: String,
}

/// AI整理計画のフォルダ構造体（再帰的）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiFolderPlan {
    pub name: String,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub children: Vec<AiFolderPlan>,
}

/// AI整理計画のルート構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiPlan {
    #[serde(default)]
    pub folders: Vec<AiFolderPlan>,
}

/// AI整理のプレビュー表示用データ構造
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiPreviewItem {
    pub item_name: String,
    pub item_type: String,
    pub target_path: String,
}

/// AIモデル情報
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiModelInfo {
    pub id: String,
}
