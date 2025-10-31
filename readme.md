
# 🧩 FM26 Mod Manager

**FM26 Mod Manager** is a cross-platform utility for **Football Manager 2026**, designed to safely manage, enable, disable, and organize mods for both **Windows** and **macOS** users.
Developed by the **FM Match Lab Team**, it provides a simple, safe, and reliable way to work with `.bundle` and `.fmf`-style mod files used by FM26.

---

## 🚀 Features

### 🗂 Mod Management

* **Import Mods** directly from folders or `.zip` archives — no manual extraction needed.
* Each mod includes a `manifest.json` file that defines:

  * Mod name, author, version, and type (`ui`, `graphics`, `database`, etc.)
  * File mappings (source → game target path)
  * Platform (Windows / Mac specific builds)
  * Optional metadata like `homepage`, `license`, and `dependencies`.

### 🧠 Enable / Disable Mods

* Toggle mods on or off with a single click.
* The **Enabled Mods** column lets you visually see active mods.
* Disabled mods are automatically skipped during “Apply Order.”

### 🪜 Load Order Control

* Easily move mods **up or down** in priority.
* FM26 Mod Manager uses a “last-write-wins” system — the lowest mod in the list overwrites conflicting files last.
* The order is stored persistently and applied whenever you hit **Apply Order**.

### ⚠️ Conflict Detection

* Detects **file conflicts** between mods that overwrite the same target file(s).
* Automatically opens a **Conflict Manager** window when overlaps exist among enabled mods.
* Shows which mod “wins” based on load order and allows you to selectively disable conflicting ones.

### 💾 Backup & Restore Points

* Every applied mod creates automatic **backups** of replaced files.
* You can also create manual **restore points** before applying new mods.
* Restore points let you roll back the game’s data folder to a clean, previous state with one click.

### 🧱 Cross-Platform Game Detection

FM26 Mod Manager automatically detects your installation path:

**Windows (Steam):**

```
C:\Program Files (x86)\Steam\steamapps\common\Football Manager 26\fm_Data\StreamingAssets\aa\StandaloneWindows64
```

**Windows (Epic Games):**

```
C:\Program Files\Epic Games\Football Manager 26\fm_Data\StreamingAssets\aa\StandaloneWindows64
```

**macOS (Steam):**

```
~/Library/Application Support/Steam/steamapps/common/Football Manager 26/fm.app/Contents/Resources/Data/StreamingAssets/aa/StandaloneOSX
```

**macOS (Epic Games):**

```
~/Library/Application Support/Epic/Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneOSXUniversal
```

If not detected automatically, you can set it manually with **Set Target**.

---

## 🪶 Safety Features

* **Finder-safe logging:** avoids revealing active log files to prevent Finder crashes on macOS.
* **Cross-platform storage:**

  * macOS: `~/Library/Application Support/FMMLoader26`
  * Windows: `%APPDATA%\FMMLoader26`
* **Auto-backup on overwrite** — every replaced `.bundle` or `.fmf` file is backed up first.

---

## 💻 Interface Overview

| Area              | Description                                                       |
| ----------------- | ----------------------------------------------------------------- |
| **Top Bar**       | Detect or manually set your FM26 target folder                    |
| **Mod List**      | Displays mod name, type, author, order, and status                |
| **Right Panel**   | Buttons for enabling, disabling, moving, and applying mods        |
| **Details Panel** | Shows `manifest.json` contents for the selected mod               |
| **Log Panel**     | Real-time event log showing applied operations and backup actions |
| **Footer**        | “Presented by the FM Match Lab Team”                              |

---

## 🧩 Manifest Format

Every mod folder (or `.zip`) must include a `manifest.json` file like this:

```json
{
  "name": "UI Bundle Patch",
  "version": "1.0.0",
  "type": "ui",
  "author": "FM Match Lab",
  "homepage": "https://example.com",
  "description": "Replaces FM26 UI panel bundle for dark theme support.",
  "files": [
    {
      "source": "ui-panelids_assets_all Mac.bundle",
      "target_subpath": "ui-panelids_assets_all.bundle",
      "platform": "mac"
    },
    {
      "source": "ui-panelids_assets_all Windows.bundle",
      "target_subpath": "ui-panelids_assets_all.bundle",
      "platform": "windows"
    }
  ]
}
```

---

## ⚙️ Installation & Usage

### 1. Download or Build

You can either download prebuilt versions from the [Releases](../../releases) page or build locally:

```bash
pip install -r requirements.txt
pyinstaller --windowed --onefile src/fm26_mod_manager_gui.py
```

### 2. Launch the App

* On **macOS**, run `FM26 Mod Manager.app`
* On **Windows**, run `FM26 Mod Manager.exe`

### 3. Point to Your FM26 Game Folder

* Click **Detect Target** or use **Set Target…** to manually locate your Standalone folder.

### 4. Import Mods

* Click **Import Mod…**
* Choose a `.zip` file or folder containing a valid `manifest.json`.

### 5. Enable Mods & Apply

* Enable or disable mods, then click **Apply Order** to install them.
* All file replacements are automatically backed up.

### 6. Handle Conflicts

* If mods edit the same file, a **Conflict Manager** window opens automatically.
* You can choose which mods to disable directly from that window.

### 7. Roll Back Anytime

* Use **Rollback…** to restore to any previous restore point.

---

## 🧱 Build Instructions (GitHub Actions)

This repository includes a ready-to-go workflow in `.github/workflows/build.yml` that:

* Builds `.exe` on **Windows**
* Builds `.app` on **macOS**
* Uploads both as GitHub Action artifacts

To trigger builds, tag a release:

```bash
git tag v0.4.0
git push origin v0.4.0
```

---

## 🛡️ Logging & Storage

* Logs are stored in:

  * macOS: `~/Library/Application Support/FMMLoader26/logs`
  * Windows: `%APPDATA%\FMMLoader26\logs`
* Each run creates a file named like:

  ```
  run_20251030-153000.log
  ```

* The latest log pointer is always `last_run.log`.

---

## 🧰 Developer Notes

### File Locations

* **Backups:** `backups/`
* **Restore Points:** `restore_points/`
* **Mods:** `mods/`
* **Config File:** `config.json`

### Config Keys

```json
{
  "target_path": "/path/to/StandaloneOSX",
  "enabled_mods": ["DarkUI", "PlayerStats"],
  "load_order": ["DarkUI", "PlayerStats"]
}
```

### Supported Platforms

✅ macOS 12+ (Intel / Apple Silicon)
✅ Windows 10 / 11

---

## 🧡 Credits

Developed and maintained by the **FM Match Lab Team**.
Special thanks to everyone in the Football Manager modding community for testing and feedback.

---

## 📄 License

This project is open source under the **MIT License**.
You are free to modify and distribute the code with proper attribution.

---
