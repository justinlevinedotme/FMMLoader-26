# 🧩 FMMLoader26 — Football Manager 2026 Mod Manager

**Cross-platform mod manager** for *Football Manager 2026*, built to make installing, organizing, and troubleshooting mods easy — on both **macOS** and **Windows**.  

Developed by [Justin Levine](https://justinlevine.me) / notJalco and presented with the great help of **The FM Match Lab Team**.

Join us on [Discord](https://discord.gg/QCW7QhWdAs) to discuss, ask questions, or contribute.

## NOTE: This applicaiton involves modifying your FM game files. Some mods work, some mods don't. It is not a substitute or plug-and-play solution. It simply makes the distrubtion and installation of mods simpler. You should know how to verify your game files, and undo any changes if need be. 

---

## ✨ Features

- 🖥️ **Cross-platform GUI** (Tkinter + TkinterDnD2)
- 🔍 **Automatic FM folder detection**
- 📦 **Drag & drop** import of `.zip` or folder-based mods
- ✅ **Enable / Disable mods** with one click
- 📚 **Load order control** (last-write-wins system)
- ⚔️ **Conflict Manager** — detects overlapping files before applying mods
- 💾 **Restore points & one-click rollback**
- 🧱 **Safe Finder / Explorer operations** (no macOS crashes)
- 🧠 **FM process detection** — warns if the game is open
- 🗂️ **Type-aware installs**
  - Automatically installs mods to the right folders:
    - Bundles → game data folder  
    - Tactics → `Documents/Sports Interactive/Football Manager 26/tactics`  
    - Graphics → `/graphics`
- 📜 **Detailed logs** stored per run
- 💡 **No install required** — standalone `.exe` or `.app`

---

## 🚀 Installation

### macOS

1. Download the latest **`FMMLoader26-mac.zip`** from the [Releases](../../releases) page.
2. Extract and drag `FMMLoader26.app` into `/Applications` or your preferred folder.
3. The first time you open it, **Control-click → Open** to bypass Gatekeeper.

### Windows

1. Download **`FMMLoader26.exe`** from [Releases](../../releases).
2. Run it directly — no installation required.

---

## 🧠 How It Works

Each mod lives in your `mods` directory and contains a small JSON manifest (`manifest.json`) describing what files to copy and where to copy them.  

When you enable mods and click **Apply Order**:

- The app creates a restore point (backup).
- Mods are applied **in load order** (bottom overrides top).
- If any conflicts exist (mods editing the same file), you’ll be shown the **Conflict Manager** before proceeding.
- If Football Manager is open, the app will prompt you to close it first.

---

## 🕹️ Usage Guide

### 1️⃣ Detect or Set Your Target Folder

- Click **Detect Target** — the app finds your FM data folder automatically.
- If that fails, click **Set Target…** and choose it manually (EPIC games installs also auto detect):
  - **macOS (Steam):**  
    `~/Library/Application Support/Steam/steamapps/common/Football Manager 26/fm.app/Contents/Resources/Data/StreamingAssets/aa/StandaloneOSX`
  - **Windows (Steam):**  
    `C:\Program Files (x86)\Steam\steamapps\common\Football Manager 26\fm_Data\StreamingAssets\aa\StandaloneWindows64`

---

### 2️⃣ Import a Mod

- Drag a `.zip` file or folder directly into the app window,  
  **or** click **Import Mod…** and pick your file.
- The app automatically extracts `.zip` archives and places the mod in your `mods` folder.

---

### 3️⃣ Enable / Disable

- Select a mod and click **Enable (mark)** or **Disable (unmark)**.  
- Enabled mods show `yes` in the “Enabled” column.

---

### 4️⃣ Adjust Load Order

- Select a mod and use **Up (Order)** or **Down (Order)**.  
- The last mod in the list wins if multiple mods modify the same file.

---

### 5️⃣ Apply Mods

- Click **Apply Order** to write changes to the FM folder.
- You’ll get a warning if:
  - Football Manager is running.
  - There are file conflicts between mods.

---

### 6️⃣ Rollback

- Use **Rollback…** to restore a previous backup created automatically before each apply.

---

### 7️⃣ Open Folders / Logs

- **Open Mods Folder** — view your installed mods.
- **Open Logs Folder** — view detailed logs for each run.
- **Copy Log Path** — copies the current log’s file path to your clipboard.

---

````markdown
## 🧩 Mod Manifest Format (for Mod Makers)

Every mod must include a `manifest.json` file in its root folder.  
This file tells FMMLoader26 where to install the mod’s files and for which platform (Windows or macOS).

### 📄 Example Manifest

```json
{
  "name": "FM26 UI Pack",
  "version": "1.0.0",
  "type": "ui",
  "author": "FM Match Lab",
  "description": "Custom UI panels for FM26.",
  "files": [
    { "source": "ui-panelids_assets_all Mac.bundle", "target_subpath": "ui-panelids_assets_all.bundle", "platform": "mac" },
    { "source": "ui-panelids_assets_all Windows.bundle", "target_subpath": "ui-panelids_assets_all.bundle", "platform": "windows" }
  ]
}
````

---

### 🔧 Supported Fields

| Key           | Type   | Description                                                                                                        |
| ------------- | ------ | ------------------------------------------------------------------------------------------------------------------ |
| `name`        | string | Display name for the mod. Must be unique within the `mods` folder.                                                 |
| `version`     | string | Mod version (e.g. `1.0.2`). Used for update comparison.                                                            |
| `type`        | string | Determines **where the mod installs**. See table below.                                                            |
| `author`      | string | The mod creator or team name.                                                                                      |
| `homepage`    | string | *(Optional)* A website, forum, or download link.                                                                   |
| `description` | string | *(Optional)* Summary text displayed in the manager.                                                                |
| `files`       | array  | A list of objects describing which files to copy and where. Each entry must include `source` and `target_subpath`. |
| `platform`    | string | `"mac"`, `"windows"`, or omit to apply to both platforms.                                                          |

---

### 📁 File Entries Explained

Each file entry in the `files` array should look like this:

```json
{ "source": "ui-panelids_assets_all Windows.bundle", "target_subpath": "ui-panelids_assets_all.bundle", "platform": "windows" }
```

| Key              | Description                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------------- |
| `source`         | The file or folder path **relative to the mod’s root directory**. Example: `"data/training.fmf"`. |
| `target_subpath` | The filename or subpath relative to the game’s **target install folder** (depending on `type`).   |
| `platform`       | Optional — limits installation to mac or Windows only. Leave blank to support both.               |

---

### 🗂️ Install Locations by Type

| Type            | Installs To                                                                                                                                                                  | Notes                                                                                           |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `bundle` / `ui` | Game installation directory → `fm_Data/StreamingAssets/aa/Standalone[Platform]`                                                                                              | For UI and game data bundles (.bundle, .fmf, etc). **Use this for UI mods instead of `skins`.** |
| `tactics`       | User folder → `Documents/Sports Interactive/Football Manager 26/tactics` (Windows) or `~/Library/Application Support/Sports Interactive/Football Manager 26/tactics` (macOS) | For `.fmf` tactic or formation files.                                                           |
| `graphics`      | User folder → `.../Football Manager 26/graphics`                                                                                                                             | For facepacks, logos, or kits.                                                                  |
| `editor-data`   | User folder → `.../Football Manager 26/editor data`                                                                                                                          | For `.fmf` database mods.                                                                       |
| `schedules`     | User folder → `.../Football Manager 26/schedules`                                                                                                                            | For training schedule packs.                                                                    |
| `misc`          | User folder root (`Football Manager 26/`)                                                                                                                                    | For any other files not covered above.                                                          |

> 🧠 **UI mods note:**
> In FM26, **UI and interface bundles no longer belong in `/skins/`**.
> Instead, use `"type": "ui"` (or `"bundle"`) so the manager installs them directly into the game’s internal StreamingAssets folder.

---

### 🪶 Relative Paths and How They Work

* All paths in your manifest are **relative to your mod’s root folder**.
  For example, if your folder looks like this:

  ```
  MyCoolMod/
  ├─ manifest.json
  ├─ data/
  │  └─ training.fmf
  └─ ui/
     └─ ui-panelids_assets_all Windows.bundle
  ```

  Then your entries might be:

  ```json
  "files": [
    { "source": "data/training.fmf", "target_subpath": "training.fmf" },
    { "source": "ui/ui-panelids_assets_all Windows.bundle", "target_subpath": "ui-panelids_assets_all.bundle", "platform": "windows" }
  ]
  ```

* `target_subpath` defines the final file name inside the destination folder determined by `type`.
  For example:

  * If `type = "tactics"` → the file installs into your FM *tactics folder*.
  * If `type = "ui"` → the file installs into your FM *StreamingAssets* data folder.

---

### ✅ Good Practices

* Always **include platform entries** if your mod uses different builds for Windows and macOS.
* Keep names short and unique; folder name = mod name.
* Test using **Enable → Apply Order → Disable → Apply Order** to ensure clean installation and removal.
* Do **not** put huge archives (like graphics megapacks) directly in one file — split them by category for faster load and conflict detection.

---

### ⚠️ Conflict Rules

Two mods conflict if they both define the same `target_subpath`.
If you try to apply such mods simultaneously, FMMLoader26 will show a **Conflict Manager** prompt before writing any files.

---

## ⚙️ Configuration and Data Paths

| Component      | macOS Path                                                 | Windows Path                           |
| -------------- | ---------------------------------------------------------- | -------------------------------------- |
| Config         | `~/Library/Application Support/FMMLoader26/config.json`    | `%AppData%\FMMLoader26\config.json`    |
| Mods           | `~/Library/Application Support/FMMLoader26/mods`           | `%AppData%\FMMLoader26\mods`           |
| Backups        | `~/Library/Application Support/FMMLoader26/backups`        | `%AppData%\FMMLoader26\backups`        |
| Restore Points | `~/Library/Application Support/FMMLoader26/restore_points` | `%AppData%\FMMLoader26\restore_points` |
| Logs           | `~/Library/Application Support/FMMLoader26/logs`           | `%AppData%\FMMLoader26\logs`           |

---

## 🔍 Troubleshooting

**⚠️ Finder crashes on macOS:**
Finder-safe mode is already built in (no recursive reveals). If it still hangs, open the log folder manually:
`~/Library/Application Support/FMMLoader26/logs`

**🚫 “Permission denied” when writing mods:**
Ensure Football Manager isn’t running, and that your Steam or Epic installation folder isn’t sandboxed (right-click → Get Info → unlock permissions), or run as administrator.

**🧩 FM not detected automatically:**
Use **Set Target…** and navigate manually (see above paths).

**💡 No drag-and-drop?**
Install TkinterDnD2:

```bash
pip install tkinterdnd2
```

---

## 🛠️ Building from Source

```bash
git clone https://github.com/<yourname>/FMMLoader26.git
cd FMMLoader26
pip install -r requirements.txt
python src/fm26_mod_manager_gui.py
```

To package executables:

```bash
pyinstaller --noconfirm --onefile --windowed src/fm26_mod_manager_gui.py
```

or use the GitHub Actions workflow provided in `.github/workflows/build.yml`.

---

## 🧩 Requirements

- Python 3.10+
- Packages:

  - `psutil`
  - `tkinterdnd2` *(optional for drag-and-drop)*

---

## 🧠 Development Notes

- Mods live in per-platform `mods` directory under app data.
- The app logs every enable/disable and file copy.
- Conflicts are detected by comparing `target_subpath` across all manifests.
- Load order defines overwrite priority.

---

## 🤝 Contributing

All contributions welcome!

1. Fork this repo
2. Create a feature branch (`feature/add-theme-toggle`)
3. Commit your changes
4. Submit a Pull Request

Use the included GitHub issue templates for:

- 🐞 Bug reports
- 💡 Feature requests
- ❓ Questions

---

## 🧾 License

This project is licensed under the **Creative Commons Attribution–NonCommercial 4.0 International License (CC BY-NC 4.0)**.  

You are free to:

- **Share** — copy and redistribute the material in any medium or format  
- **Adapt** — remix, transform, and build upon the material  

Under the following terms:

- **Attribution** — You must give appropriate credit and link to this repository.  
- **NonCommercial** — You may not use the material for commercial purposes.  

See the full license here:  
[https://creativecommons.org/licenses/by-nc/4.0/](https://creativecommons.org/licenses/by-nc/4.0/)
