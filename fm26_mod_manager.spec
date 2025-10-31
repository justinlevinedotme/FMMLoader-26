# fm26_mod_manager.spec
import sys
from PyInstaller.utils.hooks import collect_all

datas, binaries, hiddenimports = collect_all('tkinter')

app_name = "FM26 Mod Manager"
icon_path = "src/assets/icon.ico" if sys.platform.startswith("win") else "src/assets/icon.icns"

a = Analysis(
    ['src/fm26_mod_manager_gui.py'],
    pathex=['.'],
    binaries=binaries,
    datas=datas + [('src/assets', 'assets')],
    hiddenimports=hiddenimports,
    noarchive=False,
)

exe = EXE(
    a.pure, a.scripts, a.binaries, a.zipfiles, a.datas,
    name=app_name,
    icon=icon_path,
    console=False,  # no console window
)

app = BUNDLE(
    exe,
    name=f"{app_name}.app" if sys.platform == "darwin" else app_name,
    icon=icon_path,
)
