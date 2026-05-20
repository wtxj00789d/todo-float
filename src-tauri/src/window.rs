use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn open_main_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("main").is_some() {
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Todo Float")
        .inner_size(300.0, 470.0)
        .min_inner_size(300.0, 360.0)
        .max_inner_size(480.0, 720.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(false)
        .build()?;

    Ok(())
}
