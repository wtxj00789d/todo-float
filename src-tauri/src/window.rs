use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const WINDOW_WIDTH: f64 = 300.0;
const WINDOW_HEIGHT: f64 = 470.0;
const SCREEN_MARGIN: f64 = 16.0;

pub fn open_main_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("main").is_some() {
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Todo Float")
        .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .min_inner_size(300.0, 360.0)
        .max_inner_size(480.0, 720.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(false);

    if let Some((x, y)) = top_right_position(app)? {
        builder = builder.position(x, y);
    }

    builder.build()?;

    Ok(())
}

fn top_right_position(app: &AppHandle) -> tauri::Result<Option<(f64, f64)>> {
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(None);
    };

    let work_area = monitor.work_area();
    let scale_factor = monitor.scale_factor();
    let left = work_area.position.x as f64 / scale_factor;
    let x = left + work_area.size.width as f64 / scale_factor - WINDOW_WIDTH - SCREEN_MARGIN;
    let y = work_area.position.y as f64 / scale_factor + SCREEN_MARGIN;

    Ok(Some((x.max(left + SCREEN_MARGIN), y)))
}
