use std::collections::HashSet;
use std::fs;
use std::path::Path;
use crate::models::CategoryRule;

/// 指定されたパスが除外リストに含まれているか判定
pub fn is_excluded(path: &Path, excluded_paths: &[String]) -> bool {
    let path_str = path.to_string_lossy().to_string();
    excluded_paths.iter().any(|p| {
        // 完全一致、または除外フォルダ配下のパスであるかを確認
        path_str == *p || path_str.starts_with(&(p.to_owned() + std::path::MAIN_SEPARATOR.to_string().as_str()))
    })
}

/// フォルダの中身を見てプロジェクトカテゴリを判定するヘルパー
pub fn get_category_for_folder(path: &std::path::Path, prefix: &str, rules: &[CategoryRule]) -> Result<String, String> {
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

    for rule in rules {
        if rule.extensions.iter().any(|e| exts.contains(&e.to_lowercase())) {
            return Ok(format!("{}{}", prefix, rule.name));
        }
    }

    Ok(format!("{}NoCategories", prefix))
}
