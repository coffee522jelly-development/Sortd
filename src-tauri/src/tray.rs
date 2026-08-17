use tauri::{AppHandle, Manager, Runtime, Emitter};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, MouseButton, TrayIconEvent};

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let delete_i = MenuItem::with_id(app, "delete", "一括削除", true, None::<&str>)?;
    let organize_i = MenuItem::with_id(app, "organize", "拡張子ごとに整理", true, None::<&str>)?;
    let classify_i = MenuItem::with_id(app, "classify", "内容で分類", true, None::<&str>)?;
    let ai_organize_i = MenuItem::with_id(app, "ai-organize", "AIで整理", true, None::<&str>)?;
    let today_i = MenuItem::with_id(app, "today", "今日のファイルを整理", true, None::<&str>)?;
    let empty_i = MenuItem::with_id(app, "empty", "ゴミ箱を空にする", true, None::<&str>)?;
    let duplicates_i = MenuItem::with_id(app, "duplicates", "重複ファイルを削除", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "設定", true, None::<&str>)?;
    let toggle_i = MenuItem::with_id(app, "toggle", "表示/非表示", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &delete_i, &organize_i, &classify_i, &ai_organize_i, &today_i, &empty_i, &duplicates_i,
        &settings_i, &toggle_i, &quit_i
    ])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app, event| {
            let action = event.id.as_ref();
            match action {
                "toggle" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {
                    // Make sure window is visible if we want to run commands
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit(format!("tray-action-{}", action).as_str(), ());
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == MouseButton::Left {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
