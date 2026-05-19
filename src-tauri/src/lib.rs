use chrono::Local;
use models::{AppState, ParseRequest, ParseResponse, SavePreviewRequest, Todo};
use std::path::PathBuf;

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
    let date = today();
    let conn = db::open_database(&database_path()?).map_err(|err| err.to_string())?;
    let todos = db::active_todos_for_date(&conn, date).map_err(|err| err.to_string())?;

    Ok(AppState { today: date, todos })
}

#[tauri::command]
async fn parse_todo_text(request: ParseRequest) -> Result<ParseResponse, String> {
    let config_path = config::config_path()?;
    let config = config::load_config_from(&config_path)?;
    if config.llm.api_key.trim().is_empty() {
        return Err(format!(
            "主模型 API Key 为空，请在配置文件 {} 中填写 api_key",
            config_path.display()
        ));
    }

    let date = today();
    let prompt = parser::build_prompt(date);
    let primary_request = llm::ProviderRequest {
        provider: config.llm.provider.clone(),
        api_key: config.llm.api_key.clone(),
        model: config.llm.model.clone(),
        prompt: prompt.clone(),
        user_text: request.text.clone(),
    };

    let raw_content = match llm::call_provider(primary_request).await {
        Ok(content) => content,
        Err(primary_error) => {
            let Some(fallback) = config.llm.fallback else {
                return Err(primary_error);
            };
            if fallback.api_key.trim().is_empty() {
                return Err(primary_error);
            }

            let fallback_request = llm::ProviderRequest {
                provider: fallback.provider,
                api_key: fallback.api_key,
                model: fallback.model,
                prompt,
                user_text: request.text.clone(),
            };
            llm::call_provider(fallback_request)
                .await
                .map_err(|_| "主模型和备用模型均调用失败，请稍后重试或检查配置。".to_string())?
        }
    };

    let entries = llm::parse_llm_entries(&request.text, &raw_content)?;
    let entries = parser::apply_local_overrides(entries, &request.text, date);
    Ok(ParseResponse { entries })
}

#[tauri::command]
fn save_preview(request: SavePreviewRequest) -> Result<Vec<Todo>, String> {
    let date = today();
    let conn = db::open_database(&database_path()?).map_err(|err| err.to_string())?;
    for entry in &request.entries {
        db::insert_preview_entry(&conn, entry).map_err(|err| err.to_string())?;
    }
    db::active_todos_for_date(&conn, date).map_err(|err| err.to_string())
}

#[tauri::command]
fn suppress_today() -> Result<(), String> {
    let date = today();
    let conn = db::open_database(&database_path()?).map_err(|err| err.to_string())?;
    let todos = db::active_todos_for_date(&conn, date).map_err(|err| err.to_string())?;
    let max_created_at = todos.iter().map(|todo| todo.created_at.as_str()).max();
    db::record_popup(&conn, date, max_created_at, true).map_err(|err| err.to_string())
}

fn run_startup_check(app: &tauri::AppHandle) -> Result<(), String> {
    let date = today();
    let conn = db::open_database(&database_path()?).map_err(|err| err.to_string())?;
    let todos = db::active_todos_for_date(&conn, date).map_err(|err| err.to_string())?;
    let snapshot = db::latest_popup_snapshot(&conn, date).map_err(|err| err.to_string())?;

    if startup::should_show_popup(&todos, snapshot) {
        let max_created_at = todos.iter().map(|todo| todo.created_at.as_str()).max();
        window::open_main_window(app).map_err(|err| err.to_string())?;
        db::record_popup(&conn, date, max_created_at, false).map_err(|err| err.to_string())?;
    }

    Ok(())
}

fn database_path() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("TODO_FLOAT_DB") {
        return Ok(PathBuf::from(path));
    }

    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let dir = exe
        .parent()
        .ok_or_else(|| "Cannot find executable directory".to_string())?;
    Ok(dir.join("todo-float.sqlite3"))
}

fn today() -> chrono::NaiveDate {
    Local::now().date_naive()
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
            let handle = app.handle();
            if is_startup_check {
                let _ = run_startup_check(handle);
            } else {
                window::open_main_window(handle)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Todo Float");
}
