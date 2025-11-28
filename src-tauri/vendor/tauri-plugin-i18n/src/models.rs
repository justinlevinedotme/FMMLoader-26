use rust_i18n_support::load_locales;
use std::{collections::BTreeMap, sync::Mutex};

use tauri::{AppHandle, Emitter, Manager, Runtime};

#[derive(Debug)]
pub struct PluginI18n<R: Runtime> {
    pub app: AppHandle<R>,
    pub data: BTreeMap<String, BTreeMap<String, String>>,
    pub locale: Mutex<String>,
}

impl<R: Runtime> PluginI18n<R> {
    ///
    /// Initialize the data using the locale and path provided.
    ///
    pub fn new(app: tauri::AppHandle<R>, path: String, locale: String) -> Self {
        // load_locales returns HashMap; convert to BTreeMap for stable ordering
        let raw = load_locales(&path, |_| false);
        let data: BTreeMap<String, BTreeMap<String, String>> = raw
            .into_iter()
            .map(|(locale, map)| {
                let inner: BTreeMap<String, String> = map.into_iter().collect();
                (locale, inner)
            })
            .collect();
        Self {
            app,
            data,
            locale: Mutex::new(locale),
        }
    }

    ///
    /// Gets the available locales
    ///
    pub fn available_locales(&self) -> Vec<String> {
        self.data.keys().map(|k| k.to_string()).collect()
    }

    ///
    /// Gets the translated string according to the current locale
    ///
    pub fn translate(&self, key: &str) -> Option<&str> {
        let locale = self.locale.lock().ok()?;

        self.data
            .get(&locale.to_string())?
            .get(key)
            .map(|k| k.as_str())
    }

    ///
    /// Returns the data used for translations
    ///
    pub fn get_translations_data(&self) -> BTreeMap<String, BTreeMap<String, String>> {
        self.data.clone()
    }

    ///
    /// Update the locale.
    /// eg: "zh-CN", "en-US"
    ///
    pub fn set_locale(&self, locale: &str) {
        let mut l = self.locale.lock().unwrap();
        *l = locale.to_string();
        let _ = self.app.emit("i18n:locale_changed", locale);
    }

    ///
    /// Get the current locale.
    /// eg: "zh-CN", "en-US"
    /// Default locale is "en".
    ///
    pub fn get_locale(&self) -> String {
        let locale = self.locale.lock();
        if let Ok(l) = locale {
            l.to_string()
        } else {
            "en".to_string()
        }
    }
}

pub trait PluginI18nExt<R: Runtime> {
    fn i18n(&self) -> &PluginI18n<R>;
}

impl<R: Runtime, T: Manager<R>> PluginI18nExt<R> for T {
    fn i18n(&self) -> &PluginI18n<R> {
        self.state::<PluginI18n<R>>().inner()
    }
}
