mod commands;
mod models;
mod rss;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::add_feed,
            commands::discover_feeds,
            commands::refresh_feed,
            commands::extract_abstract,
            commands::translate_text,
            commands::encrypt_value,
            commands::decrypt_value,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
