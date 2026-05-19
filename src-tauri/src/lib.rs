use chrono::Local;
use models::{AppState, ParseRequest, ParseResponse, SavePreviewRequest, Todo};

mod config;
mod date_rules;
mod db;
mod llm;
mod models;
mod parser;
mod startup;
mod window;

#[tauri::command]
fn get_app_state() -> Result<AppState, String> {
    Ok(AppState {
        today: Local::now().date_naive(),
        todos: Vec::new(),
    })
}

#[tauri::command]
async fn parse_todo_text(_request: ParseRequest) -> Result<ParseResponse, String> {
    Err("API key is empty".to_string())
}

#[tauri::command]
fn save_preview(_request: SavePreviewRequest) -> Result<Vec<Todo>, String> {
    Ok(Vec::new())
}

#[tauri::command]
fn suppress_today() -> Result<(), String> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--startup-check".into()]),
        ))
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            parse_todo_text,
            save_preview,
            suppress_today
        ])
        .setup(|app| {
            let is_startup_check = std::env::args().any(|arg| arg == "--startup-check");
            if !is_startup_check {
                window::open_main_window(app.handle())?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Todo Float");
}
