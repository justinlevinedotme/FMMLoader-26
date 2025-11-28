# Tauri Plugin i18n (A simple wrapper around rust_i18n)

> This plugin does not support macros from rust_i18n.

> rust_i18n is only used for parsing the locales.

![i18n usage](i18n_usage.gif)

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | Untested  |
| iOS      | Untested  |

## Install

_This plugin requires a Rust version of at least **1.77.2**_

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
# Point this to your fork's repository and branch/tag/rev
# Example using a GitHub repo:
[dependencies.tauri-plugin-i18n]
git = "https://github.com/razein97/tauri-plugin-i18n"

# Or use a local path if developing locally:
# path = "../path/to/your/fork/tauri-plugin-i18n"
```

The package can also be installed by using cargo:

```sh
cargo add tauri-plugin-i18n
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

Install the JavaScript bindings using your preferred package manager:

```bash
# Using pnpm
pnpm add @razein97/tauri-plugin-i18n

# Using npm
npm install @razein97/tauri-plugin-i18n

# Using yarn
yarn add @razein97/tauri-plugin-i18n
```

## Locale file [rust_i18n implementation]

You can use `_version` key to specify the version (This version is the locale file version, not the rust-i18n version) of the locale file, and the default value is `1`.

rust-i18n supports two style of config file, and those versions will always be keeping.

- `_version: 1` - Split each locale into difference files, it is useful when your project wants to split to translate work.
- `_version: 2` - Put all localized text into same file, it is easy to translate quickly by AI (e.g.: GitHub Copilot). When you write original text, just press Enter key, then AI will suggest you the translation text for other languages.

You can choose as you like.

### Split Localized Texts into Difference Files

> \_version: 1

You can also split the each language into difference files, and you can choose (YAML, JSON, TOML), for example: `en.json`:

```bash
.
├── Cargo.lock
├── Cargo.toml
├── locales
│   ├── zh-CN.yml
│   ├── en.yml
└── src
│   └── main.rs
```

```yml
_version: 1
hello: 'Hello world'
mello: 'Mello world'
```

Or use JSON or TOML format, just rename the file to `en.json` or `en.toml`, and the content is like this:

```json
{
  "_version": 1,
  "hello": "Hello world",
  "mello": "Mello world"
}
```

```toml
hello = "Hello world"
mello = "Mello world"
```

### All Localized Texts in One File

> \_version: 2

Make sure all localized files (containing the localized mappings) are located in the `locales/` folder of the project root directory:

```bash
.
├── Cargo.lock
├── Cargo.toml
├── locales
│   ├── app.yml
│   ├── some-module.yml
└── src
│   └── main.rs
└── sub_app
│   └── locales
│   │   └── app.yml
│   └── src
│   │   └── main.rs
│   └── Cargo.toml
```

In the localized files, specify the localization keys and their corresponding values, for example, in `app.yml`:

```yml
_version: 2
hello:
  en: Hello world
  zh-CN: 你好世界
```

This is useful when you use [GitHub Copilot](https://github.com/features/copilot), after you write a first translated text, then Copilot will auto generate other locale's translations for you.

<img src="https://user-images.githubusercontent.com/5518/262332592-7b6cf058-7ef4-4ec7-8dea-0aa3619ce6eb.gif" width="446" />

## Usage

First you need to register the core plugin with Tauri:

The init method takes two args:

- The location of the locales dir
- Default locale (eg: "en")

<br/>

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_i18n::init("./locales", None))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings and also via `tauri::AppHandle`:

#### JS

```javascript
import I18n from '@razein97/tauri-plugin-i18n';

// Load translations
await I18n.getInstance().load();

// Example
//Get available locales
const locales = await I18n.getAvailableLocales();
console.log('Locales:', locales);
```

#### Rust

```rust
#[tauri::command]
fn get_locales(app: tauri::AppHandle)-> Vec<String> {
  app.i18n().available_locales()
}
```

---

### Updating the locale

#### JS

```javascript
import I18n from '@razein97/tauri-plugin-i18n';

// Example
// Set the locale
await I18n.setLocale('en');
```

#### Rust

```rust
#[tauri::command]
fn update_locale(app: tauri::AppHandle, locale: &str) {
  app.i18n().set_locale(locale);
}
```

---

### Using the locale

**Note: The i18n class must be instantiated in order to translate in javascript.**

#### JS

```javascript
// In javascript, it uses the [data-i18n] attribute to remain framework agnostic

import I18n from '@razein97/tauri-plugin-i18n';

// Load translations
await I18n.getInstance().load();

// Example
// Use a translation
<p data-i18n="hello"> </p>;
```

#### Rust

```rust
  let translated: Option<&str> = app.i18n().translate("hello");
```

**Call the destroy method in javascript when done to cleanup.**

> See the examples folder for a working app.
