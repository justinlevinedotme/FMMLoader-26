use tauri::{
    menu::{IconMenuItem, Menu, MenuBuilder, MenuEvent, MenuId, MenuItem, NativeIcon, Submenu},
    Wry,
};
use tauri_plugin_i18n::PluginI18nExt;

#[tauri::command]
pub fn open_custom_menu(app_handle: tauri::AppHandle, window: tauri::Window) {
    let m = menu(&app_handle);
    window.popup_menu(&m).unwrap();
}

// Use the MenuEvent::receiver to listen to click events on the menu items
pub fn custom_menu_receiver(app: &tauri::AppHandle, event: MenuEvent) {
    // Define a trait to convert the MenuId to a string
    impl MenuIdString for MenuId {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
    trait MenuIdString {
        fn to_string(&self) -> String;
    }

    let event_id_string: &str = &event.id().to_string();

    if event_id_string == "custom_menu:text_menu" {
        let cur_locale = app.i18n().get_locale();
        let all_locales = app.i18n().available_locales();
        for locale in all_locales {
            if locale != cur_locale {
                app.i18n().set_locale(&locale);
                break;
            }
        }
    } else {
        println!("{event_id_string:?} -- Not handled")
    }
}

fn menu(manager: &tauri::AppHandle) -> Menu<Wry> {
    MenuBuilder::new(manager)
        .item(
            &IconMenuItem::with_id_and_native_icon(
                manager,
                "custom_menu:native_icon",
                manager
                    .i18n()
                    .translate("menu.icon_menu")
                    .unwrap_or_default(),
                true,
                Some(NativeIcon::Folder),
                None::<&str>,
            )
            .unwrap(),
        )
        .item(
            &MenuItem::with_id(
                manager,
                "custom_menu:text_menu",
                manager
                    .i18n()
                    .translate("menu.text_menu")
                    .unwrap_or_default(),
                true,
                None::<&str>,
            )
            .unwrap(),
        )
        .separator()
        .item(
            &Submenu::with_id_and_items(
                manager,
                "submenu",
                manager.i18n().translate("menu.submenu").unwrap_or_default(),
                true,
                &[
                    &MenuItem::with_id(
                        manager,
                        "custom_menu:submenu_item_1",
                        manager
                            .i18n()
                            .translate("menu.submenu_item_1")
                            .unwrap_or_default(),
                        true,
                        None::<&str>,
                    )
                    .unwrap(),
                    &MenuItem::with_id(
                        manager,
                        "custom_menu:submenu_item_2",
                        manager
                            .i18n()
                            .translate("menu.submenu_item_2")
                            .unwrap_or_default(),
                        true,
                        None::<&str>,
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        )
        .build()
        .unwrap()
}
