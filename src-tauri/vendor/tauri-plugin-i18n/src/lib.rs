use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

/// Initializes the plugin.
pub fn init<R: Runtime>(locale_path: &'static str, locale: Option<String>) -> TauriPlugin<R> {
    Builder::new("i18n")
        .invoke_handler(tauri::generate_handler![
            commands::load_translations,
            commands::translate,
            commands::set_locale,
            commands::get_locale,
            commands::get_available_locales,
        ])
        .setup(|app, _api| {
            let path = locale_path.to_string();
            app.manage(PluginI18n::new(
                app.clone(),
                path,
                locale.unwrap_or("en".to_string()),
            ));

            Ok(())
        })
        .build()
}
