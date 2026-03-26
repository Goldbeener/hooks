use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                window.set_shadow(false)?;
                if let Some(monitor) = window.current_monitor()? {
                    let screen_size = monitor.size();
                    let scale = monitor.scale_factor();
                    let menu_bar = (24.0 * scale) as u32;
                    let win_width = (400.0 * scale) as u32;
                    let full_height = screen_size.height.saturating_sub(menu_bar + (70.0 * scale) as u32);
                    let win_height = (full_height as f64 * 0.8) as u32;
                    window.set_size(tauri::PhysicalSize::new(win_width, win_height))?;
                    let x = (screen_size.width as i32) - (win_width as i32);
                    let y = menu_bar as i32;
                    window.set_position(tauri::PhysicalPosition::new(x, y))?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}