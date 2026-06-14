use tauri::Runtime;
use tauri::Manager;

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
