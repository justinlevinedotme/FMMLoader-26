const COMMANDS: &[&str] = &[
    "load_translations",
    "translate",
    "set_locale",
    "get_locale",
    "get_available_locales",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        // .global_api_script_path("./api-iife.js")
        .build();
}
