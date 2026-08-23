use std::collections::HashSet;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use serde_json::json;
use crate::models::{AiDesktopItem, AiFolderPlan, AiModelInfo, AiPlan, AiPreviewItem, MoveOp};
use crate::state::AppState;
use crate::utils::is_excluded;

/// AIモデル一覧を取得するコマンド (LM Studio / OpenAI 互換)
#[tauri::command]
pub async fn get_ai_models(endpoint: String, api_key: String) -> Result<Vec<AiModelInfo>, String> {
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTPクライアント初期化エラー: {}", e))?;

    let auth_token = if api_key.trim().is_empty() {
        "lm-studio".to_string()
    } else {
        api_key.trim().to_string()
    };

    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .map_err(|e| format!("AIプロバイダへの接続に失敗しました ({}): {}", url, e))?;

    if !resp.status().is_success() {
        return Err(format!("モデル一覧の取得に失敗しました (ステータスコード: {})", resp.status()));
    }

    let json_val: serde_json::Value = resp.json().await
        .map_err(|e| format!("レスポンスの解析に失敗しました: {}", e))?;

    let mut models = Vec::new();
    if let Some(data) = json_val.get("data").and_then(|d| d.as_array()) {
        for item in data {
            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                models.push(AiModelInfo { id: id.to_string() });
            }
        }
    }

    Ok(models)
}

/// AI整理計画のプレビューを取得するコマンド
#[tauri::command]
pub async fn get_ai_organization_preview(
    endpoint: String,
    model: String,
    api_key: String,
    excluded: Vec<String>,
) -> Result<Vec<AiPreviewItem>, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let entries = fs::read_dir(&desktop).map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    let mut item_type_map = std::collections::HashMap::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if is_excluded(&path, &excluded) {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext == "lnk" || ext == "url" {
                continue;
            }
            items.push(AiDesktopItem {
                r#type: "file".to_string(),
                name: name.clone(),
            });
            item_type_map.insert(name, "file".to_string());
        } else if path.is_dir() {
            items.push(AiDesktopItem {
                r#type: "folder".to_string(),
                name: name.clone(),
            });
            item_type_map.insert(name, "folder".to_string());
        }
    }

    if items.is_empty() {
        return Ok(Vec::new());
    }

    let system_prompt = r#"You are an expert desktop file system taxonomy designer.
Analyze the user's desktop files and folders provided in JSON and group them into logical, semantic category folders.

STRICT RULES:
1. Respond ONLY with valid JSON strictly conforming to the following JSON schema:
   {
     "folders": [
       {
         "name": "CategoryName",
         "items": ["filename1.pdf", "foldername1"],
         "children": []
       }
     ]
   }
2. Item names in "items" MUST match EXACTLY with the input names. Do NOT rename, shorten, or modify item names.
3. Every item listed in "items" MUST be an item from the input list.
4. An item must appear at most ONCE across all folders.
5. Output MUST be raw JSON without any markdown formatting or explanation."#;

    let user_content = serde_json::to_string(&json!({ "items": items }))
        .map_err(|e| format!("リクエストデータの生成に失敗しました: {}", e))?;

    let payload = json!({
        "model": if model.is_empty() { "default" } else { &model },
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_content }
        ],
        "temperature": 0.2
    });

    let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("HTTPクライアント初期化エラー: {}", e))?;

    let auth_token = if api_key.trim().is_empty() {
        "lm-studio".to_string()
    } else {
        api_key.trim().to_string()
    };

    let resp = client.post(&url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("AIプロバイダとの通信に失敗しました: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("AIプロバイダがエラーを返しました (ステータスコード: {})", resp.status()));
    }

    let json_resp: serde_json::Value = resp.json().await
        .map_err(|e| format!("AIプロバイダからのレスポンスの解析に失敗しました: {}", e))?;

    let content = json_resp.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|m| m.get("message"))
        .and_then(|c| c.get("content"))
        .and_then(|s| s.as_str())
        .ok_or("AIレスポンスの本文を取得できませんでした")?;

    let cleaned_json = clean_json_response(content);
    let plan: AiPlan = serde_json::from_str(&cleaned_json)
        .map_err(|e| format!("AIのレスポンスが正しいJSON整理計画ではありません: {}\n出力: {}", e, content))?;

    // Validate plan
    let mut flattened_moves = Vec::new();
    for folder in &plan.folders {
        flatten_folder_plan("", folder, &mut flattened_moves);
    }

    let mut seen_items = HashSet::new();
    let mut previews = Vec::new();

    for (item_name, target_path) in flattened_moves {
        if !item_type_map.contains_key(&item_name) {
            return Err(format!("AIが存在しないアイテム '{}' を移動対象に指定しました", item_name));
        }

        if !seen_items.insert(item_name.clone()) {
            return Err(format!("アイテム '{}' が複数の整理先フォルダに指定されています", item_name));
        }

        let item_type = item_type_map.get(&item_name).cloned().unwrap_or_else(|| "file".to_string());
        previews.push(AiPreviewItem {
            item_name,
            item_type,
            target_path,
        });
    }

    Ok(previews)
}

/// AI整理計画を実行するコマンド
#[tauri::command]
pub fn execute_ai_organization(
    state: State<'_, AppState>,
    items: Vec<AiPreviewItem>,
    excluded: Vec<String>,
) -> Result<usize, String> {
    let desktop = dirs::desktop_dir().ok_or("デスクトップディレクトリが見つかりません")?;
    let mut moved_count = 0;
    let mut batch = Vec::new();

    for item in items {
        let source_path = desktop.join(&item.item_name);
        if !source_path.exists() || is_excluded(&source_path, &excluded) {
            continue;
        }

        let mut target_path = desktop.join(&item.target_path);

        // 自分自身または子孫ディレクトリへの移動を防止
        if source_path.is_dir() && target_path.starts_with(&source_path) {
            return Err(format!(
                "フォルダ '{}' を自身またはその子要素に移動することはできません",
                item.item_name
            ));
        }

        if source_path == target_path {
            continue;
        }

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }

        // 宛先が存在する場合は衝突回避用にタイムスタンプ付与
        if target_path.exists() {
            let stem = source_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let ext = source_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis();

            let parent = target_path.parent().unwrap_or(&desktop);
            let new_filename = if source_path.is_file() && !ext.is_empty() {
                format!("{}_{}.{}", stem, now_ms, ext)
            } else {
                format!("{}_{}", item.item_name, now_ms)
            };
            target_path = parent.join(new_filename);
        }

        let original_path = source_path.clone();
        fs::rename(&source_path, &target_path).map_err(|e| format!("'{}' の移動に失敗しました: {}", item.item_name, e))?;

        batch.push(MoveOp {
            from: original_path,
            to: target_path,
        });
        moved_count += 1;
    }

    if !batch.is_empty() {
        let mut history = state.history.lock().unwrap();
        history.batches.push(batch);
    }

    Ok(moved_count)
}

fn clean_json_response(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find("```json") {
        let content_after = &trimmed[start + 7..];
        if let Some(end) = content_after.find("```") {
            return content_after[..end].trim().to_string();
        }
    } else if let Some(start) = trimmed.find("```") {
        let content_after = &trimmed[start + 3..];
        if let Some(end) = content_after.find("```") {
            return content_after[..end].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn flatten_folder_plan(parent_path: &str, folder: &AiFolderPlan, out: &mut Vec<(String, String)>) {
    let folder_name = folder.name.trim();
    if folder_name.is_empty() {
        return;
    }

    let current_path = if parent_path.is_empty() {
        folder_name.to_string()
    } else {
        format!("{}/{}", parent_path, folder_name)
    };

    for item in &folder.items {
        let item_name = item.trim();
        if !item_name.is_empty() {
            let target_path = format!("{}/{}", current_path, item_name);
            out.push((item_name.to_string(), target_path));
        }
    }

    for child in &folder.children {
        flatten_folder_plan(&current_path, child, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json_response() {
        let raw_md = "```json\n{\"folders\": []}\n```";
        assert_eq!(clean_json_response(raw_md), "{\"folders\": []}");

        let raw_plain = "{\"folders\": []}";
        assert_eq!(clean_json_response(raw_plain), "{\"folders\": []}");
    }

    #[test]
    fn test_flatten_folder_plan() {
        let plan = AiFolderPlan {
            name: "郷土史・地域資料".to_string(),
            items: vec!["大和町史.pdf".to_string(), "佐賀市史.pdf".to_string()],
            children: vec![
                AiFolderPlan {
                    name: "メモ".to_string(),
                    items: vec!["町史メモ.txt".to_string()],
                    children: vec![],
                }
            ],
        };

        let mut out = Vec::new();
        flatten_folder_plan("", &plan, &mut out);

        assert_eq!(out.len(), 3);
        assert_eq!(out[0], ("大和町史.pdf".to_string(), "郷土史・地域資料/大和町史.pdf".to_string()));
        assert_eq!(out[1], ("佐賀市史.pdf".to_string(), "郷土史・地域資料/佐賀市史.pdf".to_string()));
        assert_eq!(out[2], ("町史メモ.txt".to_string(), "郷土史・地域資料/メモ/町史メモ.txt".to_string()));
    }
}
