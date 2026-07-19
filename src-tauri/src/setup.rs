use tauri::{Runtime, Manager, WindowEvent};
use crate::tray;

/// アプリケーションのセットアップ（メニュー削除など）
pub fn setup<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.remove_menu();
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window_clone.hide();
            }
        });
    }

    // トレイアイコンの作成
    tray::create_tray(app.handle())?;

    Ok(())
}
