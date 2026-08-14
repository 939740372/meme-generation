use serde::Serialize;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Welcome back, {}.", name)
}

#[derive(Debug, Serialize)]
struct AppInfo {
    version: String,
    platform: String,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
