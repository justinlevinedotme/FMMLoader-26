// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use crate::custom_menu::{custom_menu_receiver, open_custom_menu};

mod custom_menu;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_i18n::init("./locales", None))
        .invoke_handler(tauri::generate_handler![open_custom_menu])
        .setup(move |app| {
            app.on_menu_event(|app_handle: &tauri::AppHandle, event| {
                //handle the menu button events
                if event.id().0.contains("custom_menu:") {
                    custom_menu_receiver(app_handle, event);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
