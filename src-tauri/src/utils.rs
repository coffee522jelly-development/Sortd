use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// 指定されたパスが除外リストに含まれているか判定
pub fn is_excluded(path: &Path, excluded_paths: &[String]) -> bool {
    let path_str = path.to_string_lossy().to_string();
    excluded_paths.iter().any(|p| {
        // 完全一致、または除外フォルダ配下のパスであるかを確認
        path_str == *p || path_str.starts_with(&(p.to_owned() + std::path::MAIN_SEPARATOR.to_string().as_str()))
    })
}

/// フォルダの中身を見てプロジェクトカテゴリを判定するヘルパー
pub fn get_category_for_folder(path: &std::path::Path, prefix: &str) -> Result<String, String> {
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
