#!/usr/bin/env python3
# FM26 Mod Manager (FMMLoader26)
# Cross-platform (macOS/Windows) GUI with:
# - Enable/disable mods, load order, filter by type
# - Import from .zip or folder (+ drag & drop if tkinterdnd2 is available)
# - Conflict manager (detects overlapping target files; disable selected)
# - Restore points & rollback
# - Finder-safe file operations on macOS (no -R reveals)
# - Footer text credit line
# - Live FM process detection (prevents applying/importing while FM is running)
# - Type-aware installs (bundle/ui to Standalone; tactics/skins/graphics/etc. to user dir)

import os, sys, json, shutil, hashlib, webbrowser, subprocess, zipfile, tempfile
from pathlib import Path
from datetime import datetime
import tkinter as tk
from tkinter import ttk, messagebox, filedialog

# Optional deps
try:
    import psutil

    _PSUTIL_OK = True
except Exception:
    _PSUTIL_OK = False

try:
    from tkinterdnd2 import TkinterDnD, DND_FILES

    DND_AVAILABLE = True
except Exception:
    DND_AVAILABLE = False

APP_NAME = "FMMLoader26"
VERSION = "0.0.3"


# -----------------------
# Paths & storage helpers
# -----------------------
def _platform_tag():
    if sys.platform.startswith("win"):
        return "windows"
    if sys.platform == "darwin":
        return "mac"
    return "other"


def legacy_appdata_dir() -> Path:
    """Older location we may migrate *from*."""
    if sys.platform.startswith("win"):
        base = os.getenv("APPDATA") or str(Path.home() / "AppData/Roaming")
        return Path(base) / APP_NAME
    else:
        # Old mac path placed under App Support/APP_NAME (same as new, but keep for completeness)
        return Path.home() / "Library/Application Support" / APP_NAME


def appdata_dir() -> Path:
    """Current storage base. Windows = %APPDATA%/APP_NAME ; macOS = ~/Library/Application Support/APP_NAME"""
    if sys.platform.startswith("win"):
        base = os.getenv("APPDATA") or str(Path.home() / "AppData/Roaming")
        p = Path(base) / APP_NAME
    else:
        p = Path.home() / "Library/Application Support" / APP_NAME
    p.mkdir(parents=True, exist_ok=True)
    return p


BASE_DIR = appdata_dir()
CONFIG_PATH = BASE_DIR / "config.json"
BACKUP_DIR = BASE_DIR / "backups"
MODS_DIR = BASE_DIR / "mods"
LOGS_DIR = BASE_DIR / "logs"
RESTORE_POINTS_DIR = BASE_DIR / "restore_points"

RUN_LOG = LOGS_DIR / f"run_{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LAST_LINK = LOGS_DIR / "last_run.log"


def safe_open_path(path: Path):
    """Open folder/file (falls back to parent). Finder-safe (no -R)."""
    try:
        path = Path(path)
        target = path if path.exists() else path.parent
        if sys.platform.startswith("win"):
            os.startfile(str(target))
        elif sys.platform == "darwin":
            subprocess.run(["open", str(target)], check=False)
        else:
            subprocess.run(["xdg-open", str(target)], check=False)
    except Exception as e:
        messagebox.showerror("Open Error", f"Could not open:\n{path}\n\n{e}")


def _init_storage():
    for p in (BACKUP_DIR, MODS_DIR, LOGS_DIR, RESTORE_POINTS_DIR):
        p.mkdir(parents=True, exist_ok=True)
    # write "pointer" to last run log (symlink if allowed, else text)
    try:
        if LAST_LINK.exists() or LAST_LINK.is_symlink():
            try:
                LAST_LINK.unlink()
            except Exception:
                pass
        try:
            LAST_LINK.symlink_to(RUN_LOG.name)
        except Exception:
            LAST_LINK.write_text(str(RUN_LOG), encoding="utf-8")
    except Exception:
        pass


def migrate_legacy_storage_copy_only():
    old = legacy_appdata_dir()
    new = BASE_DIR
    try:
        if old.exists() and any(old.iterdir()) and not any(new.iterdir()):
            shutil.copytree(old, new, dirs_exist_ok=True)
    except Exception:
        pass


migrate_legacy_storage_copy_only()
_init_storage()


# -------------
# Config I/O
# -------------
def load_config():
    if CONFIG_PATH.exists():
        try:
            return json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
        except Exception:
            return {}
    return {}


def save_config(cfg):
    CONFIG_PATH.write_text(json.dumps(cfg, indent=2), encoding="utf-8")


def get_target() -> Path | None:
    p = load_config().get("target_path")
    return Path(p) if p else None


def set_target(path: Path):
    cfg = load_config()
    cfg["target_path"] = str(path)
    save_config(cfg)


def get_enabled_mods():
    return load_config().get("enabled_mods", [])


def set_enabled_mods(mods):
    cfg = load_config()
    cfg["enabled_mods"] = mods
    save_config(cfg)


def get_load_order():
    return load_config().get("load_order", [])


def set_load_order(order):
    cfg = load_config()
    cfg["load_order"] = order
    save_config(cfg)


# -----------------------
# Game detection (common)
# -----------------------
def default_candidates():
    """Try to discover the 'Standalone...' asset folder by platform."""
    home = Path.home()
    out = []
    if sys.platform.startswith("win"):
        steam = (
            Path(os.getenv("PROGRAMFILES(X86)", "C:/Program Files (x86)"))
            / "Steam/steamapps/common/Football Manager 26"
        )
        epic = (
            Path(os.getenv("PROGRAMFILES", "C:/ Program Files"))
            / "Epic Games/Football Manager 26"
        )
        for base in (steam, epic):
            for sub in (
                "fm_Data/StreamingAssets/aa/StandaloneWindows64",
                "data/StreamingAssets/aa/StandaloneWindows64",
            ):
                p = base / sub
                if p.exists():
                    out.append(p)
    else:
        # macOS
        for p in (
            home
            / "Library/Application Support/Steam/steamapps/common/Football Manager 26/fm.app/Contents/Resources/Data/StreamingAssets/aa/StandaloneOSX",
            home
            / "Library/Application Support/Steam/steamapps/common/Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneOSXUniversal",
            home
            / "Library/Application Support/Epic/Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneOSXUniversal",
        ):
            if p.exists():
                out.append(p)
    return out


def detect_and_set():
    c = default_candidates()
    if c:
        set_target(c[0])
        return c[0]
    return None


# -------------
# Manifest I/O
# -------------
def read_manifest(mod_dir: Path):
    mf = Path(mod_dir) / "manifest.json"
    if not mf.exists():
        raise FileNotFoundError(f"No manifest.json in {mod_dir}")
    data = json.loads(mf.read_text(encoding="utf-8"))
    # sensible defaults
    data.setdefault("name", Path(mod_dir).name)
    data.setdefault("version", "")
    data.setdefault("type", "misc")  # IMPORTANT for routing
    data.setdefault("author", "")
    data.setdefault("homepage", "")
    data.setdefault("description", "")
    data.setdefault("compatibility", {})
    data.setdefault("dependencies", [])
    data.setdefault("conflicts", [])
    data.setdefault("load_after", [])
    data.setdefault("license", "")
    if "files" not in data or not isinstance(data["files"], list):
        data["files"] = []
    return data


def resolve_target(base: Path, sub: str) -> Path:
    return Path(base) / sub


# ----------
# Backups
# ----------
def backup_original(target_file: Path):
    if not Path(target_file).exists():
        return None
    h = hashlib.sha256(str(target_file).encode("utf-8")).hexdigest()[:10]
    dest = BACKUP_DIR / f"{Path(target_file).name}.{h}.bak"
    i, final = 1, dest
    while final.exists():
        final = BACKUP_DIR / f"{dest.name}.{i}"
        i += 1
    shutil.copy2(target_file, final)
    return final


def find_latest_backup_for_filename(filename: str):
    cands = sorted(
        [p for p in BACKUP_DIR.glob(f"{filename}*") if p.is_file()],
        key=lambda p: p.stat().st_mtime,
        reverse=True,
    )
    return cands[0] if cands else None


# -----------------------
# NEW: Utility helpers
# -----------------------
def is_fm_running():
    """Detect if Football Manager is currently running."""
    if not _PSUTIL_OK:
        return False
    targets = [
        "Football Manager 26",
        "FootballManager26",
        "fm.exe",
        "fm.app",
        "fm26",
        "fm26.app",
        "fm",
        "Football Manager 26.app",
    ]
    for proc in psutil.process_iter(["name", "exe", "cmdline"]):
        try:
            name = proc.info["name"] or ""
            exe = proc.info["exe"] or ""
            cmd = " ".join(proc.info["cmdline"] or [])
            nl, el, cl = name.lower(), exe.lower(), cmd.lower()
            if any(
                t.lower() in nl or t.lower() in el or t.lower() in cl for t in targets
            ):
                return True
        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
            continue
    return False


def fm_user_dir():
    """Return FM user folder (for tactics, skins, graphics, etc.)."""
    if sys.platform.startswith("win"):
        return Path.home() / "Documents" / "Sports Interactive" / "Football Manager 26"
    else:
        # macOS
        return (
            Path.home()
            / "Library/Application Support/Sports Interactive/Football Manager 26"
        )


def get_target_for_type(mod_type: str):
    """Return the appropriate install target depending on mod type."""
    # Bundles/UI must go to Standalone target (game data aa/Standalone...).
    # User content types go to user dir subfolders.
    base_user = fm_user_dir()
    type_map = {
        # Game data (Standalone target)
        "bundle": get_target(),
        "ui": get_target(),
        # User content
        "tactics": base_user / "tactics",
        "graphics": base_user / "graphics",
        "misc": base_user,
    }
    return type_map.get(mod_type, base_user)


# -------------
# Mod actions
# -------------
def enable_mod(mod_name: str, log):
    mod_dir = MODS_DIR / mod_name
    if not mod_dir.exists():
        raise FileNotFoundError(f"Mod not found: {mod_name} in {MODS_DIR}")
    mf = read_manifest(mod_dir)
    mod_type = (mf.get("type") or "misc").strip().lower()
    base = get_target_for_type(mod_type)
    if not base:
        raise RuntimeError("No valid FM26 target set. Use Detect or Set Target.")
    if not base.exists():
        # Try to mkdir only for user-dir types; Standalone should already exist.
        if base.is_relative_to(fm_user_dir()):
            base.mkdir(parents=True, exist_ok=True)
        else:
            raise RuntimeError("No valid FM26 target set. Use Detect or Set Target.")

    files = mf.get("files", [])
    if not files:
        raise ValueError("Manifest has no 'files' entries.")
    plat = _platform_tag()
    log(f"[enable] {mf.get('name', mod_name)} ({mod_type}) → {base}")
    log(f"  [context] platform={plat} files={len(files)}")
    wrote = skipped = backed_up = errors = 0
    for e in files:
        ep = e.get("platform")
        src_rel = e.get("source")
        tgt_rel = e.get("target_subpath")
        if ep and ep != plat:
            log(f"  [skip/platform] {src_rel} (entry platform={ep})")
            skipped += 1
            continue
        if not src_rel or not tgt_rel:
            log(f"  [error/entry] Missing 'source' or 'target_subpath' in {e}")
            errors += 1
            continue
        src = mod_dir / src_rel
        tgt = resolve_target(base, tgt_rel)
        if not src.exists():
            log(f"  [error/missing] Source not found: {src}")
            errors += 1
            continue
        try:
            tgt.parent.mkdir(parents=True, exist_ok=True)
            if tgt.exists():
                b = backup_original(tgt)
                log(f"  [backup] {tgt_rel}  ←  {b.name if b else 'skipped'}")
                backed_up += 1
            shutil.copy2(src, tgt)
            log(f"  [write] {src_rel}  →  {tgt_rel}")
            wrote += 1
        except Exception as ex:
            log(f"  [error/copy] {src_rel} → {tgt_rel} :: {ex}")
            errors += 1
    log(
        f"[enable/done] wrote={wrote} backup={backed_up} skipped={skipped} errors={errors}"
    )


def disable_mod(mod_name: str, log):
    mod_dir = MODS_DIR / mod_name
    mf = read_manifest(mod_dir)
    mod_type = (mf.get("type") or "misc").strip().lower()
    base = get_target_for_type(mod_type)
    if not base or not base.exists():
        raise RuntimeError("No valid FM26 target set. Use Detect or Set Target.")
    files = mf.get("files", [])
    if not files:
        log("[disable] Manifest has no files to disable.")
        return
    log(f"[disable] {mf.get('name', mod_name)}  from  {base}")
    removed = restored = missing_backup = not_present = errors = 0
    for e in files:
        tgt_rel = e.get("target_subpath")
        if not tgt_rel:
            log(f"  [error/entry] Missing 'target_subpath' in {e}")
            errors += 1
            continue
        tgt = resolve_target(base, tgt_rel)
        if tgt.exists():
            try:
                tgt.unlink()
                log(f"  [remove] {tgt_rel}")
                removed += 1
                b = find_latest_backup_for_filename(tgt.name)
                if b and b.exists():
                    shutil.copy2(b, tgt)
                    log(f"  [restore] {b.name}  →  {tgt_rel}")
                    restored += 1
                else:
                    log(f"  [no-backup] {tgt.name} (left removed)")
                    missing_backup += 1
            except Exception as ex:
                log(f"  [error/remove] {tgt_rel} :: {ex}")
                errors += 1
        else:
            log(f"  [absent] {tgt_rel}")
            not_present += 1
    log(
        f"[disable/done] removed={removed} restored={restored} no_backup={missing_backup} absent={not_present} errors={errors}"
    )


def install_mod_from_folder(src_folder: Path, name_override: str | None, log=None):
    src_folder = Path(src_folder).resolve()
    if not (src_folder / "manifest.json").exists():
        raise FileNotFoundError("Selected folder does not contain a manifest.json")
    mf = json.loads((src_folder / "manifest.json").read_text(encoding="utf-8"))
    name = (name_override or mf.get("name") or src_folder.name).strip()
    if not name:
        raise ValueError("Mod name cannot be empty.")
    dest = MODS_DIR / name
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(src_folder, dest)
    if log:
        log(f"Installed mod '{name}' to {dest}")
    return name


# ----------------
# Conflict detect
# ----------------
def build_mod_index(names=None):
    if names is None:
        names = [p.name for p in MODS_DIR.iterdir() if p.is_dir()]
    manifests = {}
    idx = {}
    for m in names:
        mf = read_manifest(MODS_DIR / m)
        manifests[m] = mf
        for f in mf.get("files", []):
            tgt = f.get("target_subpath")
            if not tgt:
                continue
            idx.setdefault(tgt, []).append(m)
    return idx, manifests


def find_conflicts(names=None):
    """Return {target_subpath: [mods...]} and manifests dict."""
    idx, manifests = build_mod_index(names)
    conflicts = {t: ms for t, ms in idx.items() if len(ms) > 1}
    return conflicts, manifests


# --------------------
# Restore points
# --------------------
def create_restore_point(base: Path, log):
    ts = datetime.now().strftime("%Y%m%d-%H%M%S")
    rp = RESTORE_POINTS_DIR / ts
    rp.mkdir(parents=True, exist_ok=True)
    idx, _ = build_mod_index(get_enabled_mods())
    for rel in idx.keys():
        src = base / rel
        if src.exists() and src.is_file():
            dst = rp / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
    log(f"Restore point created: {rp.name}")
    return rp.name


def rollback_to_restore_point(name: str, base: Path, log):
    rp = RESTORE_POINTS_DIR / name
    if not rp.exists():
        raise FileNotFoundError("Restore point not found.")
    for p in rp.rglob("*"):
        if p.is_file():
            rel = p.relative_to(rp)
            dst = base / rel.as_posix()
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(p, dst)
    log(f"Rolled back to restore point: {name}")


# --------------
# Apply order
# --------------
def apply_enabled_mods_in_order(log):
    # IMPORTANT: For mixed types, we still create a restore point for the Standalone base.
    # For user-dir types, restore points won't snapshot (only intended for game-file overwrites).
    base = get_target()
    if not base or not base.exists():
        raise RuntimeError("No valid FM26 target set. Use Detect or Set Target.")
    enabled = get_enabled_mods()
    order = get_load_order()
    ordered = [m for m in order if m in enabled] + [
        m for m in enabled if m not in order
    ]
    if not ordered:
        log("No enabled mods to apply.")
        return
    rp = create_restore_point(base, log)
    for name in ordered:
        try:
            enable_mod(name, log)
        except Exception as ex:
            log(f"[WARN] Failed enabling {name}: {ex}")
    log(
        f"Applied {len(ordered)} mod(s) in order (last-write-wins). Restore point: {rp}"
    )


# ==========
#   GUI
# ==========
BaseTk = TkinterDnD.Tk if DND_AVAILABLE else tk.Tk


class App(BaseTk):
    def __init__(self):
        super().__init__()
        self.title(f"FMMLoader26 v{VERSION} — Presented by the FM Match Lab Team")
        self.geometry("1120x820")
        self.minsize(1000, 700)
        if DND_AVAILABLE:
            self.drop_target_register(DND_FILES)
            self.dnd_bind("<<Drop>>", self.on_drop)
        self.create_widgets()
        self.refresh_target_display()
        self.refresh_mod_list()
        self._log("Ready.")

    # ---- logging ----
    def _log(self, msg: str):
        try:
            self.log_text.insert(tk.END, msg + "\n")
            self.log_text.see(tk.END)
        except Exception:
            pass
        try:
            with open(RUN_LOG, "a", encoding="utf-8") as f:
                f.write(msg + "\n")
        except Exception:
            pass

    # ---- UI layout ----
    def create_widgets(self):
        # Menus
        menubar = tk.Menu(self)
        file_menu = tk.Menu(menubar, tearoff=0)
        file_menu.add_command(label="Detect Target\tCtrl+D", command=self.on_detect)
        file_menu.add_command(label="Set Target…\tCtrl+O", command=self.on_set_target)
        file_menu.add_separator()
        file_menu.add_command(label="Open Target", command=self.on_open_target)
        file_menu.add_command(label="Open Mods Folder", command=self.on_open_mods)
        file_menu.add_command(
            label="Open Logs Folder", command=self.on_open_logs_folder
        )
        file_menu.add_separator()
        file_menu.add_command(label="Quit", command=self.destroy)
        menubar.add_cascade(label="File", menu=file_menu)

        actions_menu = tk.Menu(menubar, tearoff=0)
        actions_menu.add_command(label="Apply Order\tF5", command=self.on_apply_order)
        actions_menu.add_command(label="Conflicts…", command=self.on_conflicts)
        actions_menu.add_command(label="Rollback…", command=self.on_rollback)
        menubar.add_cascade(label="Actions", menu=actions_menu)
        self.config(menu=menubar)

        # Shortcuts
        self.bind_all("<Control-d>", lambda e: self.on_detect())
        self.bind_all("<Control-o>", lambda e: self.on_set_target())
        self.bind_all("<F5>", lambda e: self.on_apply_order())
        if sys.platform == "darwin":
            self.bind_all("<Command-d>", lambda e: self.on_detect())
            self.bind_all("<Command-o>", lambda e: self.on_set_target())

        # Target row
        top = ttk.Frame(self)
        top.pack(side=tk.TOP, fill=tk.X, padx=8, pady=8)
        self.target_var = tk.StringVar()
        ttk.Label(top, text="Target:").pack(side=tk.LEFT)
        self.target_entry = ttk.Entry(top, textvariable=self.target_var, width=120)
        self.target_entry.pack(side=tk.LEFT, padx=(4, 6))

        # Controls row
        flt = ttk.Frame(self)
        flt.pack(side=tk.TOP, fill=tk.X, padx=8, pady=(0, 6))
        ttk.Button(flt, text="Detect", command=self.on_detect).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(flt, text="Set Target…", command=self.on_set_target).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(flt, text="Open Target", command=self.on_open_target).pack(
            side=tk.LEFT, padx=2
        )

        self.type_filter = tk.StringVar(value="(all)")
        self.type_combo = ttk.Combobox(
            flt,
            textvariable=self.type_filter,
            width=18,
            state="readonly",
            values=[
                "(all)",
                "ui",
                "skins",
                "database",
                "ruleset",
                "graphics",
                "audio",
                "tactics",
                "editor-data",
                "misc",
            ],
        )
        self.type_combo.pack(side=tk.RIGHT, padx=6)
        ttk.Label(flt, text="Filter mod type:").pack(side=tk.RIGHT)
        self.type_combo.bind("<<ComboboxSelected>>", lambda e: self.refresh_mod_list())

        # Main list + right panel
        mid = ttk.Frame(self)
        mid.pack(side=tk.TOP, fill=tk.BOTH, expand=True, padx=8, pady=(0, 8))
        cols = ("name", "version", "type", "author", "order", "enabled")
        self.tree = ttk.Treeview(mid, columns=cols, show="headings", height=12)
        for c in cols:
            self.tree.heading(c, text=c.capitalize())
        self.tree.column("name", width=300, anchor="w")
        self.tree.column("version", width=90, anchor="w")
        self.tree.column("type", width=110, anchor="w")
        self.tree.column("author", width=160, anchor="w")
        self.tree.column("order", width=60, anchor="center")
        self.tree.column("enabled", width=80, anchor="center")
        self.tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        sb = ttk.Scrollbar(mid, orient="vertical", command=self.tree.yview)
        sb.pack(side=tk.LEFT, fill=tk.Y)
        self.tree.configure(yscrollcommand=sb.set)

        right = ttk.Frame(mid)
        right.pack(side=tk.LEFT, fill=tk.Y, padx=8)
        ttk.Button(right, text="Refresh", command=self.refresh_mod_list).pack(
            fill=tk.X, pady=2
        )
        ttk.Button(right, text="Import Mod…", command=self.on_import_mod).pack(
            fill=tk.X, pady=2
        )
        ttk.Button(right, text="Enable (mark)", command=self.on_enable_selected).pack(
            fill=tk.X, pady=(12, 2)
        )
        ttk.Button(
            right, text="Disable (unmark)", command=self.on_disable_selected
        ).pack(fill=tk.X, pady=2)
        ttk.Button(right, text="Up (Order)", command=self.on_move_up).pack(
            fill=tk.X, pady=(12, 2)
        )
        ttk.Button(right, text="Down (Order)", command=self.on_move_down).pack(
            fill=tk.X, pady=2
        )
        ttk.Button(right, text="Apply Order", command=self.on_apply_order).pack(
            fill=tk.X, pady=(12, 2)
        )
        ttk.Button(right, text="Conflicts…", command=self.on_conflicts).pack(
            fill=tk.X, pady=2
        )
        ttk.Button(right, text="Rollback…", command=self.on_rollback).pack(
            fill=tk.X, pady=(12, 2)
        )
        ttk.Button(right, text="Open Mods Folder", command=self.on_open_mods).pack(
            fill=tk.X, pady=2
        )
        ttk.Button(
            right, text="Open Logs Folder", command=self.on_open_logs_folder
        ).pack(fill=tk.X, pady=2)
        ttk.Button(right, text="Copy Log Path", command=self.on_copy_log_path).pack(
            fill=tk.X, pady=(12, 2)
        )
        ttk.Button(
            right, text="Help (Manifest)", command=self.on_show_manifest_help
        ).pack(fill=tk.X, pady=(12, 2))

        # Details pane
        det = ttk.LabelFrame(self, text="Details")
        det.pack(side=tk.TOP, fill=tk.X, padx=8, pady=(0, 8))
        self.details_text = tk.Text(det, height=6)
        self.details_text.pack(fill=tk.BOTH, expand=True)
        self.tree.bind("<<TreeviewSelect>>", self.on_select_row)

        # Log pane
        log_frame = ttk.LabelFrame(self, text="Log")
        log_frame.pack(side=tk.TOP, fill=tk.BOTH, expand=False, padx=8, pady=(0, 8))
        self.log_text = tk.Text(log_frame, height=10)
        self.log_text.pack(fill=tk.BOTH, expand=True)

        # Footer
        footer = ttk.Frame(self)
        footer.pack(side=tk.BOTTOM, fill=tk.X, padx=8, pady=8)
        ttk.Label(
            footer, text="Presented by the FM Match Lab Team", anchor="center"
        ).pack()

    # ---- menu/button actions ----
    def on_copy_log_path(self):
        self.clipboard_clear()
        self.clipboard_append(str(RUN_LOG))
        self._log(f"Copied log path: {RUN_LOG}")

    def on_open_logs_folder(self):
        safe_open_path(LOGS_DIR)

    def refresh_target_display(self):
        t = get_target()
        self.target_var.set(str(t) if t else "")

    def refresh_mod_list(self):
        """Refresh table, accurately reflecting which mods are enabled."""
        for i in self.tree.get_children():
            self.tree.delete(i)

        wanted = self.type_filter.get()
        order = get_load_order()
        enabled = set(get_enabled_mods())
        rows = []

        for p in MODS_DIR.iterdir():
            if not p.is_dir():
                continue
            try:
                mf = read_manifest(p)
                mtype = mf.get("type", "misc")
                if wanted != "(all)" and mtype != wanted:
                    continue
                ord_idx = order.index(p.name) if p.name in order else -1
                ord_disp = (ord_idx + 1) if ord_idx >= 0 else ""
                ena = "yes" if p.name in enabled else ""  # dynamically check enabled set
                rows.append(((p.name, mf.get("version", ""), mtype,
                            mf.get("author", ""), ord_disp, ena), mf))
            except Exception:
                rows.append(((p.name, "?", "?", "?", "", ""), None))

        for row, _ in rows:
            self.tree.insert("", tk.END, values=row)

        # Update status in log
        self._log(f"Loaded {len(rows)} mod(s) (filter: {wanted}). Enabled mods: {len(enabled)}")

    def selected_mod_name(self):
        sel = self.tree.selection()
        if not sel:
            return None
        return self.tree.item(sel[0])["values"][0]

    def on_detect(self):
        t = detect_and_set()
        if t:
            self._log(f"Detected target: {t}")
        else:
            messagebox.showwarning(
                "Detect",
                "Could not auto-detect FM26 Standalone folder.\nSet it manually.",
            )
        self.refresh_target_display()

    def on_set_target(self):
        chosen = filedialog.askdirectory(
            title="Select FM26 Standalone folder (StandaloneWindows64/StandaloneOSX/OSXUniversal)"
        )
        if not chosen:
            return
        p = Path(chosen).expanduser()
        if not p.exists():
            messagebox.showerror("Set Target", "Selected path does not exist.")
            return
        if "Standalone" not in p.name:
            if not messagebox.askyesno(
                "Confirm",
                f"Selected folder does not contain 'Standalone' in its name.\nUse anyway?\n\n{p}",
            ):
                return
        set_target(p)
        self.refresh_mod_list()
        self._log(f"Set target to: {p}")
        self.refresh_target_display()

    def _choose_import_source(self) -> Path | None:
        """Choose import from ZIP or Folder, then show proper dialog."""
        if messagebox.askyesno(
            "Import", "Import from a .zip file?\n\nClick 'No' to pick a folder instead."
        ):
            path = filedialog.askopenfilename(
                title="Select Mod .zip", filetypes=[("Zip archives", "*.zip")]
            )
            return Path(path) if path else None
        else:
            folder = filedialog.askdirectory(
                title="Select Mod Folder (must contain manifest.json)"
            )
            return Path(folder) if folder else None

    def on_import_mod(self):
        if is_fm_running():
            messagebox.showwarning(
                "FM is Running", "Please close Football Manager before importing mods."
            )
            return
        choice = self._choose_import_source()
        if not choice:
            return
        temp_dir = None
        try:
            if choice.is_file() and choice.suffix.lower() == ".zip":
                temp_dir = Path(tempfile.mkdtemp(prefix="fm26_import_"))
                with zipfile.ZipFile(choice, "r") as z:
                    z.extractall(temp_dir)
                # try to find a child folder with manifest.json, else use root
                candidates = [
                    d
                    for d in temp_dir.iterdir()
                    if d.is_dir() and (d / "manifest.json").exists()
                ]
                src_folder = (
                    candidates[0]
                    if candidates
                    else (temp_dir if (temp_dir / "manifest.json").exists() else None)
                )
                if src_folder is None:
                    # fallback: if there's exactly one directory, use it; else use temp root
                    subs = [d for d in temp_dir.iterdir() if d.is_dir()]
                    src_folder = subs[0] if subs else temp_dir
            else:
                src_folder = choice
            newname = install_mod_from_folder(src_folder, None, log=self._log)
            order = get_load_order()
            if newname not in order:
                order.append(newname)
                set_load_order(order)
            self.refresh_mod_list()
            messagebox.showinfo("Import", f"Imported '{newname}'.")
        except Exception as e:
            messagebox.showerror("Import Error", str(e))
        finally:
            if temp_dir:
                shutil.rmtree(temp_dir, ignore_errors=True)

    # Drag & Drop import (optional)
    def on_drop(self, event):
        if is_fm_running():
            messagebox.showwarning(
                "FM is Running", "Please close Football Manager before importing mods."
            )
            return
        raw = event.data.strip()
        # On mac/win, tkinterdnd2 may wrap paths with {} when spaces exist
        if raw.startswith("{") and raw.endswith("}"):
            raw = raw[1:-1]
        path = Path(raw)
        if not path.exists():
            return
        temp = None
        try:
            if path.suffix.lower() == ".zip":
                temp = Path(tempfile.mkdtemp(prefix="fm26_drop_"))
                with zipfile.ZipFile(path, "r") as z:
                    z.extractall(temp)
                folder = next((d for d in temp.iterdir() if d.is_dir()), temp)
            else:
                folder = path
            newname = install_mod_from_folder(folder, None, log=self._log)
            order = get_load_order()
            if newname not in order:
                order.append(newname)
                set_load_order(order)
            self.refresh_mod_list()
            messagebox.showinfo("Import", f"Imported '{newname}' via drag-and-drop.")
        except Exception as e:
            messagebox.showerror("Import Error", str(e))
        finally:
            if temp:
                shutil.rmtree(temp, ignore_errors=True)

    def on_enable_selected(self):
        name = self.selected_mod_name()
        if not name:
            messagebox.showinfo("Mods", "Select a mod first.")
            return
        enabled = get_enabled_mods()
        if name not in enabled:
            enabled.append(name)
            set_enabled_mods(enabled)
            self._log(f"Enabled (marked) '{name}'. Use Apply Order to write files.")
            self.refresh_mod_list()
        else:
            messagebox.showinfo("Mods", f"'{name}' already enabled (marked).")

    def on_disable_selected(self):
        name = self.selected_mod_name()
        if not name:
            messagebox.showinfo("Mods", "Select a mod first.")
            return
        enabled = [m for m in get_enabled_mods() if m != name]
        set_enabled_mods(enabled)
        self._log(
            f"Disabled (unmarked) '{name}'. Apply Order to rewrite files without it."
        )
        self.refresh_mod_list()

    def on_move_up(self):
        name = self.selected_mod_name()
        if not name:
            return
        order = get_load_order()
        if name not in order:
            order.append(name)
        i = order.index(name)
        if i > 0:
            order[i - 1], order[i] = order[i], order[i - 1]
            set_load_order(order)
            self._log(f"Moved up: {name}")
            self.refresh_mod_list()

    def on_move_down(self):
        name = self.selected_mod_name()
        if not name:
            return
        order = get_load_order()
        if name not in order:
            order.append(name)
        i = order.index(name)
        if i < len(order) - 1:
            order[i + 1], order[i] = order[i], order[i + 1]
            set_load_order(order)
            self._log(f"Moved down: {name}")
            self.refresh_mod_list()

    def on_apply_order(self):
        # Only show conflicts window when the user tries to apply *and* conflicts exist
        enabled = get_enabled_mods()
        conflicts, _ = find_conflicts(enabled if enabled else None)
        if conflicts:
            self._log(f"Found {len(conflicts)} conflict(s) among enabled mods.")
            # Open the conflicts manager instead of applying
            self.on_conflicts()
            return
        try:
            if 'is_fm_running' in globals() and is_fm_running():
                messagebox.showwarning("FM is Running", "Please close Football Manager before applying mods.")
                return
        except Exception:
            pass
        try:
            apply_enabled_mods_in_order(self._log)
            messagebox.showinfo(
                "Apply Order",
                "All enabled mods applied in load order.\n(Last-write-wins).",
            )
        except Exception as e:
            messagebox.showerror("Apply Order Error", str(e))

    def on_conflicts(self):
        enabled = get_enabled_mods()
        conflicts, manifests = find_conflicts(enabled if enabled else None)
        if not conflicts:
            messagebox.showinfo("Conflicts", "No file overlaps among enabled mods.")
            return

        order = get_load_order()
        win = tk.Toplevel(self)
        win.title("Conflict Manager — FM26 Mod Manager")
        win.geometry("760x560")

        frame = ttk.Frame(win)
        frame.pack(fill=tk.BOTH, expand=True, padx=8, pady=8)

        text = tk.Text(frame, wrap="word", height=18)
        text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        sb = ttk.Scrollbar(frame, command=text.yview)
        sb.pack(side=tk.RIGHT, fill=tk.Y)
        text.configure(yscrollcommand=sb.set)

        text.insert(
            tk.END, "Detected conflicts where multiple mods write the same file(s):\n\n"
        )
        for rel, mods in conflicts.items():
            ranks = [(order.index(m) if m in order else -1, m) for m in mods]
            ranks.sort()
            winner = ranks[-1][1] if ranks else mods[-1]
            details = []
            for m in mods:
                mf = manifests[m]
                details.append(
                    f"{m} ({(mf.get('type','misc') or 'misc')}) by {mf.get('author','?')}"
                )
            text.insert(
                tk.END,
                f"{rel}\n  Mods: {', '.join(details)}\n  Winner by load order (last wins): {winner}\n\n",
            )
        text.config(state="disabled")

        ttk.Label(win, text="Select mods to disable:").pack(
            anchor="w", padx=8, pady=(8, 0)
        )

        # Checkbox area
        mods_to_disable = {}
        box_frame = ttk.Frame(win)
        box_frame.pack(fill=tk.BOTH, expand=True, padx=8, pady=4)
        unique_mods = sorted(set([mm for ms in conflicts.values() for mm in ms]))
        for m in unique_mods:
            var = tk.BooleanVar()
            mods_to_disable[m] = var
            ttk.Checkbutton(box_frame, text=m, variable=var).pack(anchor="w")

        def apply_disables():
            changed = []
            enabled_now = get_enabled_mods()
            for mod_name, var in mods_to_disable.items():
                if var.get() and mod_name in enabled_now:
                    enabled_now.remove(mod_name)
                    changed.append(mod_name)
            if changed:
                set_enabled_mods(enabled_now)
                self._log(f"Disabled mods due to conflicts: {', '.join(changed)}")
                messagebox.showinfo("Conflicts", f"Disabled: {', '.join(changed)}")
                self.refresh_mod_list()
            win.destroy()

        bframe = ttk.Frame(win)
        bframe.pack(pady=(8, 8))
        ttk.Button(bframe, text="Disable Selected Mods", command=apply_disables).pack(
            side=tk.LEFT, padx=6
        )
        ttk.Button(bframe, text="Close", command=win.destroy).pack(side=tk.LEFT, padx=6)

        self._log(
            f"Opened conflict manager with {len(conflicts)} overlapping file path(s)."
        )

    def on_rollback(self):
        rps = sorted(
            [p.name for p in RESTORE_POINTS_DIR.iterdir() if p.is_dir()], reverse=True
        )[:50]
        if not rps:
            messagebox.showinfo("Rollback", "No restore points found.")
            return

        win = tk.Toplevel(self)
        win.title("Choose Restore Point")
        win.geometry("420x420")
        lb = tk.Listbox(win, height=min(16, len(rps)))
        for rp in rps:
            lb.insert(tk.END, rp)
        lb.pack(fill=tk.BOTH, expand=True, padx=8, pady=8)

        def do_rb():
            sel = lb.curselection()
            if not sel:
                return
            rp = rps[sel[0]]
            try:
                base = get_target()
                if not base or not base.exists():
                    messagebox.showerror("Rollback Error", "No valid FM26 target set.")
                    return
                rollback_to_restore_point(rp, base, self._log)
                messagebox.showinfo("Rollback", f"Rolled back to {rp}.")
                win.destroy()
            except Exception as e:
                messagebox.showerror("Rollback Error", str(e))

        ttk.Button(win, text="Rollback to selected", command=do_rb).pack(pady=(0, 8))

    def on_open_target(self):
        t = get_target()
        if not t or not t.exists():
            messagebox.showinfo("Open Target", "No valid target set.")
            return
        safe_open_path(t)

    def on_open_mods(self):
        safe_open_path(MODS_DIR)

    def on_open_logs_folder(self):
        safe_open_path(LOGS_DIR)

    def on_show_manifest_help(self):
        txt = (
            "Each mod must include a manifest.json at its root:\n\n"
            "{\n"
            '  "name": "FM26 UI Pack",\n'
            '  "version": "1.0.0",\n'
            '  "type": "ui",\n'
            '  "author": "You",\n'
            '  "homepage": "https://example.com",\n'
            '  "description": "Replaces panel IDs bundle",\n'
            '  "files": [\n'
            '    { "source": "ui-panelids_assets_all Mac.bundle", "target_subpath": "ui-panelids_assets_all.bundle", "platform": "mac" },\n'
            '    { "source": "ui-panelids_assets_all Windows.bundle", "target_subpath": "ui-panelids_assets_all.bundle", "platform": "windows" }\n'
            "  ]\n"
            "}\n\n"
            "• target_subpath is relative to the Standalone… folder (for bundle/ui types).\n"
            "• Other types install under your FM user folder (tactics/skins/graphics/etc.).\n"
            "• Last-write-wins according to the load order.\n"
            f"• Mods live in: {MODS_DIR}\n"
            f"• Logs live in: {LOGS_DIR}\n"
        )
        messagebox.showinfo("Manifest format", txt)

    def on_select_row(self, _event):
        sel = self.tree.selection()
        if not sel:
            self.details_text.delete("1.0", tk.END)
            return
        name = self.tree.item(sel[0])["values"][0]
        try:
            mf = read_manifest(MODS_DIR / name)
            desc = mf.get("description", "")
            hp = mf.get("homepage", "")
            typ = (mf.get("type", "misc") or "misc").strip().lower()
            auth = mf.get("author", "")
            lic = mf.get("license", "")
            deps = ", ".join(mf.get("dependencies", [])) or "—"
            conf = ", ".join(mf.get("conflicts", [])) or "—"
            comp = mf.get("compatibility", {})
            comp_str = ", ".join([f"{k}: {v}" for k, v in comp.items()]) or "—"
            files = mf.get("files", [])
            file_list = (
                "\n".join(
                    [
                        f"- {f.get('source','?')}  →  {f.get('target_subpath','?')}"
                        for f in files
                    ]
                )
                or "—"
            )
            text = (
                f"Name: {mf.get('name',name)}\nVersion: {mf.get('version','')}\n"
                f"Type: {typ} | Author: {auth} | License: {lic}\nHomepage: {hp}\n"
                f"Compatibility: {comp_str}\nDependencies: {deps}\nConflicts: {conf}\n\n"
                f"Description:\n{desc}\n\nFiles:\n{file_list}\n"
            )
            self.details_text.delete("1.0", tk.END)
            self.details_text.insert(tk.END, text)
        except Exception as e:
            self.details_text.delete("1.0", tk.END)
            self.details_text.insert(tk.END, f"(error reading manifest) {e}")


# ---- main ----
if __name__ == "__main__":
    # macOS may print "Secure coding is not enabled..." warning for Tk — harmless.
    app = App()
    app.mainloop()
