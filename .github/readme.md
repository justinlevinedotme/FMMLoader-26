<p align="center">
  <img width="460" height="300" src="imgs/fmmloaderheader.png">
</p>
<div align="center">
<h1>FMMLoader 26</h1>

<a href="https://creativecommons.org/licenses/by-nc-sa/4.0/"><img src="https://img.shields.io/badge/license-CC%20BY--NC--SA%204.0-fdfdfd?style=flat&labelColor=black&logo=creativecommons&logoColor=white&logoSize=auto&link=https%3A%2F%2Fcreativecommons.org%2Flicenses%2Fby-nc-sa%2F4.0%2F" alt="CC BY NC SA 4.0"></a>
<a href=""><img alt= "Discord Link" src="https://img.shields.io/badge/Discord-7289da?style=flat-round&logo=discord&logoColor=white"></a>
<img alt="GitHub contributors" src="https://img.shields.io/github/contributors/justinlevinedotme/FMMLoader-26">
<img alt="GitHub Release" src="https://img.shields.io/github/v/release/justinlevinedotme/FMMLoader-26">

</div>

<p align="center">
  <a href="https://github.com/justinlevinedotme/FMMLoader-26/wiki">Wiki</a> •
  <a href="https://github.com/justinlevinedotme/FMMLoader-26/releases/latest">Download Latest Release</a> • <a href="#installation">Installation</a> •
  <a href="#features">Features</a> •
  <a href="#Usage Guide">Usage Guide</a> •
  <a href="#License">License Info</a> •
  <a href="modders.md">Modders Info</a>
</p>

**A cross-platform mod manager** for *Football Manager 2026*, built to make installing, organizing, and troubleshooting mods easy on both macOS and Windows.  

Developed by [Justin Levine](https://justinlevine.me) / notJalco and presented with the great help of [FM Match Lab](https://fmmatchlab.co.uk/)
___

## Installation

<details>
<summary>🍏 Mac Installation</summary>

### Installation on a Mac

1. Download **`FMMLoader26.zip`** from [Releases](https://github.com/justinlevinedotme/FMMLoader-26/releases).
2. Unzip the compressed file, then drag to your `Applications` folder

<p align="center">
  <img height="400" src="imgs/mac/appsmovemac.gif">
</p>
3. After moving the app to your apps folder (or folder of your choice), you can Control Click + Open to bypass Gatekeeper. You may still get a security pop up. _If you do not, skip the next section about bypassing gatekeeper_
<p align="center">
  <img height="400" src="imgs/mac/applesecurity.gif">
</p>
<h4>Bypassing Gatekeeper</h4>
Bypassing Gatekeeper is easy by visiting System Preferences →  Privacy & Security. You then can click "Open Anyway"
<p align="center">
  <img height="400" src="imgs/mac/openanyway.png">
</p>
After clicking "Open Anyway" you will likely recieve a popup. You can press "Open" again on that popup, and the app should open.<br><br>
</details>
<details>
<summary>🖥️ Windows Installation</summary>

### Installation on Windows

1. Download **`FMMLoader26.exe`** from [Releases](https://github.com/justinlevinedotme/FMMLoader-26/releases).
2. Run it directly as Administrator — no installation required. You may get a popup for Windows Defender, click "More Info" and "Run Anyway" **IMAGES NEEDED, WINDOWS USERS**

</details>

---

## Features

    - Cross-platform GUI (Tkinter + TkinterDnD2)
    - Automatic FM folder detection
    - Drag & drop import of .zip or folder-based mods
    - Enable / Disable mods with one click
    - Load order control (last-write-wins system)
    - Conflict Manager — detects overlapping files before applying mods
    - Restore points & one-click rollback
    - Type-aware installs
        Automatically installs mods to the right folders:
            Bundles → game data folder
            Tactics → Documents/Sports Interactive/Football Manager 26/tactics
            Graphics → /graphics
    - Detailed logs stored per run
    - No install required — standalone .exe or .app

___

## Usage Guide

### 1 - Detect or Set Your Target Folder

- Click **Detect Target** — the app finds your FM data folder automatically.  
- If that fails, click **Set Target…** and choose it manually (EPIC installs also auto-detect):

**macOS (Steam):**  

```bash
~/Library/Application Support/Steam/steamapps/common/Football Manager 26/fm.app/Contents/Resources/Data/StreamingAssets/aa/StandaloneOSX
````

**Windows (Steam):**

```bash
C:\Program Files (x86)\Steam\steamapps\common\Football Manager 26\fm_Data\StreamingAssets\aa\StandaloneWindows64
```

---

### 2 - Import a Mod

- Drag a `.zip` file or folder directly into the app window,
  **or** click **Import Mod…** and choose your file.
- The app automatically extracts `.zip` archives and places the mod in your `mods` folder.

---

### 3 - Enable / Disable

- Select a mod and click **Enable (mark)** or **Disable (unmark)**.
- Enabled mods show **`yes`** in the “Enabled” column.

---

### 4 - Adjust Load Order

- Select a mod and use **Up (Order)** or **Down (Order)**.
- The **last mod in the list wins** if multiple mods modify the same file.

---

### 5 - Apply Mods

- Click **Apply Order** to write changes to the FM folder.
- You’ll get a warning if:

  - Football Manager is running
  - There are file conflicts between mods

> [!WARNING]
> You may need to allow **FMMLoader** “App Management” permissions under
> `System Preferences → Privacy & Security → App Management`.

---

### 6 - Rollback

- Use **Rollback…** to restore a previous backup.
  (A backup is created automatically before each Apply.)

---

### 7 - Open Folders / Logs

- **Open Mods Folder** — view your installed mods
- **Open Logs Folder** — view detailed logs for each run
- **Copy Log Path** — copy the current log’s file path to your clipboard

---

## License

This project is licensed under the
Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License. Click the image below to learn more.
<p align="left"> <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/"> <img alt="CC BY-NC-SA 4.0 License" src="https://licensebuttons.net/l/by-nc-sa/4.0/88x31.png"> </a> </p>

___

## Modders Info

Modders can visit [Modders Docs](MODDERS.md) for more information on how to properly package their mods and skins for FMMLoader.

**Example Mods are also contained in the `example mods` directory of this repository. Thank you to knap and bassyboy for allowing me to include them with this release.**
___

## Contributing

Modders can visit [Contributing.md](CONTRIBUTING.md) for information on contributing to this project.
___

## Donating / Supporting

Supporting is as simple as downloading and sharing this project with your friends, favorite modder, or content creator. This is a passion project and is provided for free. If you decide you'd like to support me fiscally, you can do so via ko-fi.

<a href="https://ko-fi.com/jalco"><img src="https://shields.io/badge/ko--fi-donate-ff5f5f?logo=ko-fi&style=for-the-badgeKo-fi" alt="ko-fi shield"></img></a>

___

## Other Important Notes

>[!CAUTION]
> This application involves modifying your FM game files. Some mods work, some mods don't. It is not a substitute or plug-and-play solution. It simply makes the distrubtion and installation of mods simpler. You should know how to verify your game files, and undo any changes if need be
