use std::collections::BTreeMap;

use tauri::{command, AppHandle, Emitter, Runtime, State};

use crate::PluginI18n;

// use crate::models::*;
// use crate::I18nExt;

///
/// Load translations
///
#[command]
pub(crate) fn load_translations<R: Runtime>(
    _app: AppHandle<R>,
    translations: State<'_, PluginI18n<R>>,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, crate::Error> {
    let value = translations.get_translations_data();
    Ok(value)
}

///
/// Gets the translated string according to the current locale
///
#[command]
pub(crate) fn translate<R: Runtime>(
    _app: AppHandle<R>,
    translations: State<'_, PluginI18n<R>>,
    key: &str,
) -> Result<Option<String>, crate::Error> {
    let value = translations.translate(key);
    Ok(value.map(|s| s.to_string()))
}

///
/// Changes the locale. eg: "zh-CN", "en-US"
///
#[command]
pub(crate) fn set_locale<R: Runtime>(
    app: AppHandle<R>,
    translations: State<'_, PluginI18n<R>>,
    locale: &str,
) -> Result<(), crate::Error> {
    translations.set_locale(locale);
    let _ = app.emit("i18n:locale_changed", locale);
    Ok(())
}

///
/// Gets the current locale
///
#[command]
pub(crate) fn get_locale<R: Runtime>(
    _app: AppHandle<R>,
    translations: State<'_, PluginI18n<R>>,
) -> Result<String, crate::Error> {
    Ok(translations.get_locale())
}

///
/// Gets the available locales
///
#[command]
pub(crate) fn get_available_locales<R: Runtime>(
    _app: AppHandle<R>,
    translations: State<'_, PluginI18n<R>>,
) -> Result<Vec<String>, crate::Error> {
    let locales = translations.available_locales();
    Ok(locales)
}
