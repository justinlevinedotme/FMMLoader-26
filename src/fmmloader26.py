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
import customtkinter as ctk
import urllib.request
import urllib.error

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
VERSION = "0.0.8"
GITHUB_REPO = "justinlevinedotme/FMMLoader-26"


# -----------------------
# Update checker
# -----------------------
def check_for_updates():
    """
    Check GitHub releases for a newer version.
    Returns (has_update, latest_version, download_url) or (False, None, None) on error.
    """
    try:
        url = f"https://api.github.com/repos/{GITHUB_REPO}/releases/latest"
        req = urllib.request.Request(url)
        req.add_header("Accept", "application/vnd.github.v3+json")

        with urllib.request.urlopen(req, timeout=5) as response:
            data = json.loads(response.read().decode("utf-8"))
            latest_version = data.get("tag_name", "").lstrip("v")
            download_url = data.get("html_url", "")

            # Simple version comparison (assumes semantic versioning)
            current = VERSION.split(".")
            latest = latest_version.split(".")

            # Pad to same length
            while len(current) < len(latest):
                current.append("0")
            while len(latest) < len(current):
                latest.append("0")

            # Compare each part
            for c, l in zip(current, latest):
                try:
                    if int(l) > int(c):
                        return True, latest_version, download_url
                    elif int(l) < int(c):
                        return False, None, None
                except ValueError:
                    # If version parts aren't numbers, do string comparison
                    if l > c:
                        return True, latest_version, download_url
                    elif l < c:
                        return False, None, None

            # Versions are equal
            return False, None, None
    except Exception:
        # Silently fail if we can't check for updates
        return False, None, None


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


def cleanup_old_backups(keep=10):
    """Keep only the most recent N backups, delete older ones."""
    try:
        backups = sorted(
            [p for p in BACKUP_DIR.glob("*") if p.is_file()],
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )
        for old_backup in backups[keep:]:
            try:
                old_backup.unlink()
            except Exception:
                pass
    except Exception:
        pass


def cleanup_old_restore_points(keep=10):
    """Keep only the most recent N restore points, delete older ones."""
    try:
        restore_points = sorted(
            [p for p in RESTORE_POINTS_DIR.iterdir() if p.is_dir()],
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )
        for old_rp in restore_points[keep:]:
            try:
                shutil.rmtree(old_rp)
            except Exception:
                pass
    except Exception:
        pass


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

    # Clean up old backups and restore points on startup
    cleanup_old_backups(keep=10)
    cleanup_old_restore_points(keep=10)


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
    """Get list of enabled mods, filtering out any that don't exist on disk."""
    enabled = load_config().get("enabled_mods", [])
    # Filter out mods that don't exist
    valid_mods = [m for m in enabled if (MODS_DIR / m).exists()]
    # Auto-clean config if we found invalid mods
    if len(valid_mods) != len(enabled):
        set_enabled_mods(valid_mods)
    return valid_mods


def set_enabled_mods(mods):
    cfg = load_config()
    cfg["enabled_mods"] = mods
    save_config(cfg)


def get_load_order():
    """Get load order, filtering out any mods that don't exist on disk."""
    order = load_config().get("load_order", [])
    # Filter out mods that don't exist
    valid_order = [m for m in order if (MODS_DIR / m).exists()]
    # Auto-clean config if we found invalid mods
    if len(valid_order) != len(order):
        set_load_order(valid_order)
    return valid_order


def set_load_order(order):
    cfg = load_config()
    cfg["load_order"] = order
    save_config(cfg)


def get_user_dir() -> Path | None:
    """Get custom FM user directory path if set."""
    p = load_config().get("user_dir_path")
    return Path(p) if p else None


def set_user_dir(path: Path):
    """Set custom FM user directory path."""
    cfg = load_config()
    cfg["user_dir_path"] = str(path)
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


def _copy_any(src: Path, dst: Path) -> int:
    """
    Merge-copy src -> dst.
    - If src is a file: copy2(src, dst)
    - If src is a directory: recursively copy its contents into dst (dirs_exist_ok)
    Returns the number of files copied.
    """
    count = 0
    if src.is_dir():
        dst.mkdir(parents=True, exist_ok=True)
        for child in src.rglob("*"):
            rel = child.relative_to(src)
            out = dst / rel
            if child.is_dir():
                out.mkdir(parents=True, exist_ok=True)
            else:
                out.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(child, out)
                count += 1
    else:
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)
        count = 1
    return count


def fm_user_dir():
    """Return FM user folder (for tactics, skins, graphics, etc.)."""
    # Check if user has set a custom path
    custom_path = get_user_dir()
    if custom_path and custom_path.exists():
        return custom_path

    # Default paths
    if sys.platform.startswith("win"):
        return Path.home() / "Documents" / "Sports Interactive" / "Football Manager 26"
    else:
        # macOS
        return (
            Path.home()
            / "Library/Application Support/Sports Interactive/Football Manager 26"
        )


# --- add this helper near your other utils ---
def _has_manifest(p: Path) -> bool:
    """Check if a directory has a manifest.json file (case-insensitive)."""
    for name in ("manifest.json", "Manifest.json", "MANIFEST.JSON"):
        if (p / name).exists():
            return True
    return False


def _find_mod_root(path: Path) -> tuple[Path, Path | None]:
    """
    Return (mod_root, temp_dir).
    - If `path` is a directory: prefer that dir when it has manifest.json,
      otherwise search one level deep for a dir that has it.
    - If `path` is a .zip: extract to a temp dir, then search as above.
    - If `path` is a single .bundle or .fmf file: return its parent directory.
    temp_dir (if created) must be cleaned up by the caller.
    """

    # Handle single .bundle or .fmf files
    if path.is_file() and path.suffix.lower() in ('.bundle', '.fmf'):
        # For single files, we'll treat them as standalone mods
        # Return the file itself (caller will handle this specially)
        return path, None

    if path.is_file() and path.suffix.lower() == ".zip":
        tmp = Path(tempfile.mkdtemp(prefix="fm26_import_"))
        with zipfile.ZipFile(path, "r") as z:
            z.extractall(tmp)
        # 1) root
        if _has_manifest(tmp):
            return tmp, tmp
        # 2) any first-level dir with manifest
        for d in sorted([d for d in tmp.iterdir() if d.is_dir()]):
            if _has_manifest(d):
                return d, tmp
        # 3) if exactly one subdir, try it anyway
        subs = [d for d in tmp.iterdir() if d.is_dir()]
        if len(subs) == 1:
            return subs[0], tmp
        # No manifest found
        return tmp, tmp

    # Directory path
    if path.is_dir():
        if _has_manifest(path):
            return path, None
        for d in sorted([d for d in path.iterdir() if d.is_dir()]):
            if _has_manifest(d):
                return d, None
        # if exactly one subdir, try that
        subs = [d for d in path.iterdir() if d.is_dir()]
        if len(subs) == 1:
            return subs[0], None
    return path, None


def get_target_for_type(mod_type: str, mod_name: str = "") -> Path:
    """
    Return the appropriate install directory depending on mod type and mod name.
    Auto-creates /graphics and its subfolders (kits, faces, logos) if missing.
    """
    base = fm_user_dir()
    graphics_base = base / "graphics"
    mod_type = (mod_type or "").lower()
    mod_name = (mod_name or "").lower()

    # UI/bundle mods go to the main FM install location
    if mod_type in ("ui", "bundle"):
        return get_target()

    # Tactics mods go to the user's tactics folder
    if mod_type == "tactics":
        path = base / "tactics"
        path.mkdir(parents=True, exist_ok=True)
        return path

    # Graphics and its subtypes
    if mod_type == "graphics":
        graphics_base.mkdir(parents=True, exist_ok=True)
        if any(x in mod_name for x in ("kit", "kits")):
            path = graphics_base / "kits"
        elif any(x in mod_name for x in ("face", "faces", "portraits")):
            path = graphics_base / "faces"
        elif any(x in mod_name for x in ("logo", "logos", "badges")):
            path = graphics_base / "logos"
        else:
            path = graphics_base
        path.mkdir(parents=True, exist_ok=True)
        return path

    # Default fallback (misc mods)
    base.mkdir(parents=True, exist_ok=True)
    return base


# -------------
# Mod actions
# -------------
# Helper: works like Path.is_relative_to but compatible everywhere
def _is_under(child: Path, root: Path) -> bool:
    try:
        child.resolve().relative_to(root.resolve())
        return True
    except Exception:
        return False


def enable_mod(mod_name: str, log):
    mod_dir = MODS_DIR / mod_name
    if not mod_dir.exists():
        raise FileNotFoundError(f"Mod not found: {mod_name} in {MODS_DIR}")

    mf = read_manifest(mod_dir)
    mod_type = (mf.get("type") or "misc").strip().lower()

    # Check if manifest specifies a custom install_path
    if mf.get("install_path"):
        base = Path(mf.get("install_path")).expanduser()
        log(f"  [custom path] Using custom install path: {base}")
    else:
        # Pass mod name so graphics/* routing (kits/faces/logos) can work
        base = get_target_for_type(mod_type, mf.get("name", mod_name))

    if not base:
        raise RuntimeError("No valid FM26 target set. Use Detect or Set Target.")

    if not base.exists():
        # Only auto-create for user-dir installs (tactics/graphics/etc.),
        # not for the game's Standalone... directory.
        if _is_under(base, fm_user_dir()):
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
        ep = (e.get("platform") or "").strip().lower()
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
            # Back up only when the target is an existing FILE
            if tgt.exists() and tgt.is_file():
                b = backup_original(tgt)
                log(f"  [backup] {tgt_rel}  ←  {b.name if b else 'skipped'}")
                backed_up += 1

            # Directory-aware copy (merges folders, supports packs like logos/kits/faces)
            copied = _copy_any(src, tgt)
            wrote += copied
            log(f"  [write] {src_rel}  →  {tgt_rel}{' (dir)' if src.is_dir() else ''} ({copied} file(s))")

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

    # Check if manifest specifies a custom install_path
    if mf.get("install_path"):
        base = Path(mf.get("install_path")).expanduser()
        log(f"  [custom path] Using custom install path: {base}")
    else:
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


def remove_mod(mod_name: str, log):
    """
    Permanently delete a mod from the mod manager.
    Removes it from enabled_mods, load_order, and deletes the mod directory.
    """
    mod_dir = MODS_DIR / mod_name
    if not mod_dir.exists():
        raise FileNotFoundError(f"Mod not found: {mod_name}")

    # Remove from enabled mods
    enabled = get_enabled_mods()
    if mod_name in enabled:
        enabled.remove(mod_name)
        set_enabled_mods(enabled)

    # Remove from load order
    order = get_load_order()
    if mod_name in order:
        order.remove(mod_name)
        set_load_order(order)

    # Delete the mod directory
    try:
        shutil.rmtree(mod_dir)
        log(f"[remove] Permanently deleted mod '{mod_name}' from {mod_dir}")
    except Exception as ex:
        raise RuntimeError(f"Failed to delete mod directory: {ex}")


def _auto_detect_mod_type(path: Path) -> str:
    """Auto-detect mod type based on file extensions and names."""
    if path.is_file():
        ext = path.suffix.lower()
        name = path.name.lower()
        if ext == '.fmf':
            return 'tactics'
        if ext == '.bundle':
            if 'ui-' in name or 'panelids' in name:
                return 'ui'
            return 'bundle'

    # For directories, check contents
    if path.is_dir():
        has_fmf = any(f.suffix.lower() == '.fmf' for f in path.rglob('*.fmf'))
        has_bundle = any(f.suffix.lower() == '.bundle' for f in path.rglob('*.bundle'))
        has_graphics = any(d.name.lower() in ('kits', 'faces', 'logos', 'graphics')
                          for d in path.rglob('*') if d.is_dir())

        if has_fmf:
            return 'tactics'
        if has_bundle:
            return 'ui'
        if has_graphics:
            return 'graphics'

    return 'misc'


def _generate_manifest(mod_root: Path, mod_metadata: dict) -> dict:
    """
    Generate a manifest.json for a mod without one.
    mod_metadata should contain: name, type, version (optional), author (optional),
    description (optional), install_path (optional)
    """
    manifest = {
        "name": mod_metadata.get("name", mod_root.name),
        "version": mod_metadata.get("version", "1.0.0"),
        "type": mod_metadata.get("type", "misc"),
        "author": mod_metadata.get("author", ""),
        "homepage": "",
        "description": mod_metadata.get("description", ""),
        "files": [],
        "compatibility": {},
        "dependencies": [],
        "conflicts": [],
        "load_after": [],
        "license": ""
    }

    # Add custom install_path if provided
    if mod_metadata.get("install_path"):
        manifest["install_path"] = mod_metadata.get("install_path")

    mod_type = manifest["type"]

    # Handle single file mods (.bundle or .fmf)
    if mod_root.is_file():
        filename = mod_root.name
        manifest["files"] = [{
            "source": filename,
            "target_subpath": filename
        }]
        return manifest

    # Handle directory mods - auto-detect files
    files = []

    if mod_type == "tactics":
        # For tactics, include all .fmf files
        for fmf_file in sorted(mod_root.rglob("*.fmf")):
            rel_path = fmf_file.relative_to(mod_root)
            files.append({
                "source": str(rel_path),
                "target_subpath": fmf_file.name  # Tactics go flat to tactics folder
            })

    elif mod_type in ("ui", "bundle"):
        # For UI/bundle, include all .bundle files
        for bundle_file in sorted(mod_root.rglob("*.bundle")):
            rel_path = bundle_file.relative_to(mod_root)
            files.append({
                "source": str(rel_path),
                "target_subpath": bundle_file.name  # Usually goes to Standalone root
            })

    elif mod_type == "graphics":
        # For graphics, preserve directory structure
        for file in sorted(mod_root.rglob("*")):
            if file.is_file() and not file.name.startswith('.'):
                rel_path = file.relative_to(mod_root)
                files.append({
                    "source": str(rel_path),
                    "target_subpath": str(rel_path)
                })

    else:
        # For other types, include all files preserving structure
        for file in sorted(mod_root.rglob("*")):
            if file.is_file() and not file.name.startswith('.'):
                rel_path = file.relative_to(mod_root)
                files.append({
                    "source": str(rel_path),
                    "target_subpath": str(rel_path)
                })

    manifest["files"] = files
    return manifest


def install_mod_from_folder(src_folder: Path, name_override: str | None, log=None, generated_manifest: dict = None):
    """
    Install a mod from a folder. If generated_manifest is provided, it will be written to the mod directory.
    """
    src_folder = Path(src_folder).resolve()

    # Handle single file mods
    is_single_file = src_folder.is_file()

    if is_single_file:
        # For single files, we need the generated_manifest
        if not generated_manifest:
            raise ValueError("Single file mods require a generated manifest")

        filename = src_folder.name
        name = (name_override or generated_manifest.get("name") or filename).strip()
        if not name:
            raise ValueError("Mod name cannot be empty.")

        dest = MODS_DIR / name
        if dest.exists():
            shutil.rmtree(dest)
        dest.mkdir(parents=True, exist_ok=True)

        # Copy the single file to the mod directory
        shutil.copy2(src_folder, dest / filename)

        # Write the generated manifest
        manifest_path = dest / "manifest.json"
        manifest_path.write_text(json.dumps(generated_manifest, indent=2), encoding="utf-8")

        if log:
            log(f"Installed single-file mod '{name}' to {dest}")
        return name

    # Handle directory mods
    if generated_manifest:
        # Mod without original manifest - use generated one
        name = (name_override or generated_manifest.get("name") or src_folder.name).strip()
        if not name:
            raise ValueError("Mod name cannot be empty.")

        dest = MODS_DIR / name
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(src_folder, dest)

        # Write the generated manifest
        manifest_path = dest / "manifest.json"
        manifest_path.write_text(json.dumps(generated_manifest, indent=2), encoding="utf-8")

        if log:
            log(f"Installed mod '{name}' with generated manifest to {dest}")
        return name
    else:
        # Mod with existing manifest
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
    """Index target_subpath -> [unique mods touching it], for THIS platform only."""
    if names is None:
        names = [p.name for p in MODS_DIR.iterdir() if p.is_dir()]
    manifests = {}
    idx = {}  # target_subpath -> set of mod names
    plat = _platform_tag()

    for m in names:
        mod_dir = MODS_DIR / m
        # Skip mods that don't exist or don't have manifests
        if not mod_dir.exists():
            continue
        try:
            mf = read_manifest(mod_dir)
            manifests[m] = mf
            for f in mf.get("files", []):
                # 1) Skip other platforms
                ep = f.get("platform")
                if ep and ep != plat:
                    continue
                # 2) Guard + dedupe
                tgt = f.get("target_subpath")
                if not tgt:
                    continue
                idx.setdefault(tgt, set()).add(m)
        except FileNotFoundError:
            # Skip mods without manifests
            continue

    # Convert sets to lists for stable downstream use
    idx = {t: sorted(list(ms)) for t, ms in idx.items()}
    return idx, manifests


def find_conflicts(names=None):
    """Return {target_subpath: [mods...]} and manifests dict, deduped and platform-filtered."""
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
    backed_up = 0
    for rel in idx.keys():
        src = base / rel
        if src.exists():
            dst = rp / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            if src.is_file():
                shutil.copy2(src, dst)
                backed_up += 1
            elif src.is_dir():
                # Backup entire directory tree
                shutil.copytree(src, dst, dirs_exist_ok=True)
                # Count files in the directory
                backed_up += sum(1 for _ in dst.rglob("*") if _.is_file())
    log(f"Restore point created: {rp.name} (backed up {backed_up} file(s))")

    # Clean up old restore points, keeping only the 10 most recent
    cleanup_old_restore_points(keep=10)

    return rp.name


def rollback_to_restore_point(name: str, base: Path, log):
    rp = RESTORE_POINTS_DIR / name
    if not rp.exists():
        raise FileNotFoundError("Restore point not found.")

    # Build a set of all files in the restore point (these are the "good" files)
    restore_files = set()
    for p in rp.rglob("*"):
        if p.is_file():
            rel = p.relative_to(rp)
            restore_files.add(rel)

    # Build a set of all paths that mods touch (these are the areas we manage)
    idx, _ = build_mod_index(get_enabled_mods())
    managed_paths = set(Path(rel) for rel in idx.keys())

    # Delete orphaned files: files that exist in the target but not in the restore point
    # Only delete files in managed paths
    deleted = 0
    for managed_path in managed_paths:
        target_path = base / managed_path
        if target_path.exists():
            if target_path.is_file():
                # If it's a file and not in the restore point, delete it
                if managed_path not in restore_files:
                    try:
                        target_path.unlink()
                        log(f"  [deleted orphan] {managed_path}")
                        deleted += 1
                    except Exception as ex:
                        log(f"  [error deleting] {managed_path}: {ex}")
            elif target_path.is_dir():
                # If it's a directory, check all files within it
                for file_in_dir in target_path.rglob("*"):
                    if file_in_dir.is_file():
                        rel = file_in_dir.relative_to(base)
                        if rel not in restore_files:
                            try:
                                file_in_dir.unlink()
                                log(f"  [deleted orphan] {rel}")
                                deleted += 1
                            except Exception as ex:
                                log(f"  [error deleting] {rel}: {ex}")
                # Clean up empty directories
                for dir_in_path in sorted(target_path.rglob("*"), key=lambda x: len(str(x)), reverse=True):
                    if dir_in_path.is_dir() and not any(dir_in_path.iterdir()):
                        try:
                            dir_in_path.rmdir()
                        except Exception:
                            pass

    # Restore all files from the restore point
    restored = 0
    for p in rp.rglob("*"):
        if p.is_file():
            rel = p.relative_to(rp)
            dst = base / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(p, dst)
            restored += 1

    log(f"Rolled back to restore point: {name} (restored {restored} file(s), deleted {deleted} orphan(s))")


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
class ModMetadataDialog(ctk.CTkToplevel):
    """Dialog for collecting mod metadata when manifest.json is missing."""

    def __init__(self, parent, mod_path: Path, auto_detected_type: str):
        super().__init__(parent)
        self.title("Mod Metadata - No manifest.json found")
        self.geometry("520x540")
        self.resizable(False, False)

        self.mod_path = mod_path
        self.result = None

        # Make dialog modal
        self.transient(parent)
        self.grab_set()

        self._create_widgets(auto_detected_type)

        # Center on parent
        self.update_idletasks()
        x = parent.winfo_x() + (parent.winfo_width() // 2) - (self.winfo_width() // 2)
        y = parent.winfo_y() + (parent.winfo_height() // 2) - (self.winfo_height() // 2)
        self.geometry(f"+{x}+{y}")

    def _create_widgets(self, auto_detected_type: str):
        content = ctk.CTkFrame(self, corner_radius=16)
        content.pack(fill=tk.BOTH, expand=True, padx=16, pady=16)
        content.grid_columnconfigure(0, weight=1)

        ctk.CTkLabel(
            content,
            text="No manifest.json found. Please provide mod information.",
            wraplength=440,
            justify="left",
            font=("Segoe UI", 12)
        ).grid(row=0, column=0, columnspan=2, sticky="w", pady=(4, 18))

        form = ctk.CTkFrame(content, fg_color="transparent")
        form.grid(row=1, column=0, columnspan=2, sticky="nsew")
        form.grid_columnconfigure(1, weight=1)

        label_font = ("Segoe UI", 11)

        ctk.CTkLabel(form, text="Mod Name", font=label_font).grid(row=0, column=0, sticky="w", pady=6, padx=(0, 12))
        self.name_var = tk.StringVar(value=self.mod_path.stem if self.mod_path.is_file() else self.mod_path.name)
        ctk.CTkEntry(form, textvariable=self.name_var).grid(row=0, column=1, sticky="ew", pady=6)

        ctk.CTkLabel(form, text="Mod Type", font=label_font).grid(row=1, column=0, sticky="w", pady=6, padx=(0, 12))
        self.type_var = tk.StringVar(value=auto_detected_type)
        type_combo = ctk.CTkComboBox(
            form,
            values=["ui", "bundle", "tactics", "graphics", "misc"],
            state="readonly",
            command=lambda choice: self.type_var.set(choice)
        )
        type_combo.set(self.type_var.get())
        type_combo.grid(row=1, column=1, sticky="ew", pady=6)

        ctk.CTkLabel(form, text="Version (optional)", font=label_font).grid(row=2, column=0, sticky="w", pady=6, padx=(0, 12))
        self.version_var = tk.StringVar(value="1.0.0")
        ctk.CTkEntry(form, textvariable=self.version_var).grid(row=2, column=1, sticky="ew", pady=6)

        ctk.CTkLabel(form, text="Author (optional)", font=label_font).grid(row=3, column=0, sticky="w", pady=6, padx=(0, 12))
        self.author_var = tk.StringVar()
        ctk.CTkEntry(form, textvariable=self.author_var).grid(row=3, column=1, sticky="ew", pady=6)

        ctk.CTkLabel(form, text="Install Path (optional)", font=label_font).grid(row=4, column=0, sticky="w", pady=6, padx=(0, 12))
        self.install_path_var = tk.StringVar()
        ctk.CTkEntry(form, textvariable=self.install_path_var).grid(row=4, column=1, sticky="ew", pady=6)
        ctk.CTkLabel(
            form,
            text="Leave empty to use the default location based on type.",
            text_color=("#4a4a4a", "#b5b5b5"),
            font=("Segoe UI", 10)
        ).grid(row=5, column=1, sticky="w", pady=(0, 10))

        ctk.CTkLabel(form, text="Description (optional)", font=label_font).grid(row=6, column=0, sticky="nw", pady=6, padx=(0, 12))
        self.description_text = ctk.CTkTextbox(form, height=100)
        self.description_text.grid(row=6, column=1, sticky="ew", pady=6)

        button_frame = ctk.CTkFrame(content, fg_color="transparent")
        button_frame.grid(row=2, column=0, columnspan=2, sticky="e", pady=(18, 0))
        ctk.CTkButton(button_frame, text="Cancel", width=120, command=self._on_cancel).pack(side=tk.RIGHT, padx=(8, 0))
        ctk.CTkButton(button_frame, text="Import", width=140, command=self._on_import).pack(side=tk.RIGHT)

    def _on_import(self):
        name = self.name_var.get().strip()
        if not name:
            messagebox.showerror("Error", "Mod name is required.", parent=self)
            return

        mod_type = self.type_var.get().strip()
        if not mod_type:
            messagebox.showerror("Error", "Mod type is required.", parent=self)
            return

        self.result = {
            "name": name,
            "type": mod_type,
            "version": self.version_var.get().strip() or "1.0.0",
            "author": self.author_var.get().strip(),
            "install_path": self.install_path_var.get().strip(),
            "description": self.description_text.get("1.0", tk.END).strip()
        }
        self.destroy()

    def _on_cancel(self):
        self.result = None
        self.destroy()

    def get_result(self):
        """Wait for dialog to close and return the result."""
        self.wait_window()
        return self.result


ctk.set_appearance_mode("system")
ctk.set_default_color_theme("blue")


if DND_AVAILABLE:

    class BaseTk(ctk.CTk, TkinterDnD.DnDWrapper):
        def __init__(self, *args, **kwargs):
            ctk.CTk.__init__(self, *args, **kwargs)
            TkinterDnD.DnDWrapper.__init__(self)


else:

    class BaseTk(ctk.CTk):
        pass


class App(BaseTk):
    def __init__(self):
        super().__init__()
        self.title(f"FMMLoader26 v{VERSION} — Presented by the JALCO / Justin Levine")
        self.geometry("1200x900")
        self.minsize(1100, 750)
        if DND_AVAILABLE:
            self.drop_target_register(DND_FILES)
            self.dnd_bind("<<Drop>>", self.on_drop)
        self.create_widgets()
        self.refresh_target_display()
        self.refresh_user_dir_display()
        self.refresh_mod_list()
        self._log("Ready.")
        # Check for updates after a short delay (non-blocking)
        self.after(1000, self._check_for_updates_async)

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

    # ---- update checker ----
    def _check_for_updates_async(self):
        """Check for updates in the background and notify user if available."""
        has_update, latest_version, download_url = check_for_updates()
        if has_update:
            self._log(f"Update available: v{latest_version}")
            response = messagebox.askyesno(
                "Update Available",
                f"A new version of {APP_NAME} is available!\n\n"
                f"Current version: v{VERSION}\n"
                f"Latest version: v{latest_version}\n\n"
                f"Would you like to visit the download page?",
                icon='info'
            )
            if response and download_url:
                webbrowser.open(download_url)

    # ---- UI layout ----
    def create_widgets(self):
        # Menus
        menubar = tk.Menu(self)
        file_menu = tk.Menu(menubar, tearoff=0)
        file_menu.add_command(label="Detect Target\tCtrl+D", command=self.on_detect)
        file_menu.add_command(label="Set Target…\tCtrl+O", command=self.on_set_target)
        file_menu.add_separator()
        file_menu.add_command(label="Set User Directory…", command=self.on_set_user_dir)
        file_menu.add_command(label="Reset User Directory to Default", command=self.on_reset_user_dir)
        file_menu.add_separator()
        file_menu.add_command(label="Open Target", command=self.on_open_target)
        file_menu.add_command(label="Open User Directory", command=self.on_open_user_dir)
        file_menu.add_command(label="Open Mods Folder", command=self.on_open_mods)
        file_menu.add_command(
            label="Open Logs Folder", command=self.on_open_logs_folder
        )
        file_menu.add_separator()
        file_menu.add_command(label="Quit", command=self.destroy)
        menubar.add_cascade(label="File", menu=file_menu)

        actions_menu = tk.Menu(menubar, tearoff=0)
        actions_menu.add_command(label="Apply\tF5", command=self.on_apply_order)
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

        wrapper = ctk.CTkFrame(self, fg_color="transparent")
        wrapper.pack(fill=tk.BOTH, expand=True, padx=16, pady=(12, 8))

        # Target row
        self.target_var = tk.StringVar()
        target_row = ctk.CTkFrame(wrapper, fg_color="transparent")
        target_row.pack(fill=tk.X, pady=(0, 10))
        target_row.grid_columnconfigure(1, weight=1)
        ctk.CTkLabel(
            target_row,
            text="Game Target",
            font=("Segoe UI Semibold", 12)
        ).grid(row=0, column=0, sticky="w")
        self.target_entry = ctk.CTkEntry(
            target_row,
            textvariable=self.target_var,
            placeholder_text="Detect or set your Football Manager 26 folder",
            height=36
        )
        self.target_entry.grid(row=0, column=1, sticky="ew", padx=(12, 0))
        target_actions = ctk.CTkFrame(target_row, fg_color="transparent")
        target_actions.grid(row=0, column=2, sticky="e", padx=(12, 0))
        ctk.CTkButton(
            target_actions,
            text="Detect",
            width=110,
            command=self.on_detect
        ).pack(side=tk.LEFT, padx=(0, 8))
        ctk.CTkButton(
            target_actions,
            text="Browse…",
            width=110,
            command=self.on_set_target
        ).pack(side=tk.LEFT)

        # User directory row
        self.user_dir_var = tk.StringVar()
        user_row = ctk.CTkFrame(wrapper, fg_color="transparent")
        user_row.pack(fill=tk.X, pady=(0, 12))
        user_row.grid_columnconfigure(1, weight=1)
        ctk.CTkLabel(
            user_row,
            text="User Directory",
            font=("Segoe UI Semibold", 12)
        ).grid(row=0, column=0, sticky="w")
        self.user_dir_entry = ctk.CTkEntry(
            user_row,
            textvariable=self.user_dir_var,
            placeholder_text="Defaults to your Football Manager documents folder",
            height=36
        )
        self.user_dir_entry.grid(row=0, column=1, sticky="ew", padx=(12, 0))
        user_actions = ctk.CTkFrame(user_row, fg_color="transparent")
        user_actions.grid(row=0, column=2, sticky="e", padx=(12, 0))
        ctk.CTkButton(
            user_actions,
            text="Choose…",
            width=110,
            command=self.on_set_user_dir
        ).pack(side=tk.LEFT, padx=(0, 8))
        ctk.CTkButton(
            user_actions,
            text="Reset",
            width=110,
            command=self.on_reset_user_dir
        ).pack(side=tk.LEFT)
        ctk.CTkLabel(
            user_row,
            text="Used for tactics, graphics, skins and other user files",
            anchor="w",
            text_color=("#4a4a4a", "#b5b5b5"),
            font=("Segoe UI", 10)
        ).grid(row=1, column=1, sticky="w", pady=(6, 0))

        # Filter row
        toolbar = ctk.CTkFrame(wrapper, fg_color="transparent")
        toolbar.pack(fill=tk.X, pady=(0, 4))
        toolbar.pack_propagate(False)
        filter_frame = ctk.CTkFrame(toolbar, fg_color="transparent")
        filter_frame.pack(side=tk.RIGHT)
        ctk.CTkLabel(
            filter_frame,
            text="Filter by type",
            font=("Segoe UI", 11)
        ).pack(side=tk.LEFT, padx=(0, 8))
        filter_values = [
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
        ]
        self.type_filter = tk.StringVar(value="(all)")

        def on_filter_change(choice):
            self.type_filter.set(choice)
            self.refresh_mod_list()

        self.type_combo = ctk.CTkComboBox(
            filter_frame,
            values=filter_values,
            command=on_filter_change,
            width=150,
            state="readonly"
        )
        self.type_combo.set(self.type_filter.get())
        self.type_combo.pack(side=tk.LEFT)

        # Tabbed interface
        notebook = ctk.CTkTabview(wrapper)
        notebook.pack(fill=tk.BOTH, expand=True, pady=(12, 0))

        mods_tab = notebook.add("Mods")
        logs_tab = notebook.add("Logs")
        mods_tab.grid_rowconfigure(0, weight=1)
        mods_tab.grid_columnconfigure(0, weight=1)
        mods_tab.grid_rowconfigure(1, weight=0)

        # Configure treeview styling to better align with the theme
        self._configure_treeview_style()

        mid = ctk.CTkFrame(mods_tab, fg_color="transparent")
        mid.grid(row=0, column=0, sticky="nsew")
        mid.grid_rowconfigure(0, weight=1)
        mid.grid_columnconfigure(0, weight=1)

        tree_container = ctk.CTkFrame(mid, fg_color="transparent")
        tree_container.grid(row=0, column=0, sticky="nsew")
        tree_container.grid_rowconfigure(0, weight=1)
        tree_container.grid_columnconfigure(0, weight=1)

        cols = ("name", "version", "type", "author", "order", "enabled")
        self.tree = ttk.Treeview(
            tree_container,
            columns=cols,
            show="headings",
            style="CTk.Treeview"
        )
        for c in cols:
            self.tree.heading(c, text=c.capitalize())
        self.tree.column("name", width=320, anchor="w")
        self.tree.column("version", width=90, anchor="w")
        self.tree.column("type", width=120, anchor="w")
        self.tree.column("author", width=190, anchor="w")
        self.tree.column("order", width=70, anchor="center")
        self.tree.column("enabled", width=90, anchor="center")
        self.tree.grid(row=0, column=0, sticky="nsew")

        sb = ctk.CTkScrollbar(tree_container, command=self.tree.yview)
        sb.grid(row=0, column=1, sticky="ns")
        self.tree.configure(yscrollcommand=sb.set)

        right = ctk.CTkFrame(
            mid,
            fg_color=("#f0f4fa", "#111820"),
            corner_radius=14,
            border_width=1,
            border_color=("#d3d9e3", "#28303a")
        )
        right.grid(row=0, column=1, sticky="ns", padx=(16, 0))
        for child in (
            ("Refresh", self.refresh_mod_list),
            ("Import Mod…", self.on_import_mod),
            ("Enable (mark)", self.on_enable_selected),
            ("Disable (unmark)", self.on_disable_selected),
            ("Remove Mod", self.on_remove_selected),
            ("Up (Order)", self.on_move_up),
            ("Down (Order)", self.on_move_down),
            ("Apply", self.on_apply_order),
            ("Conflicts…", self.on_conflicts),
            ("Rollback…", self.on_rollback),
            ("Open Mods Folder", self.on_open_mods),
            ("Open Logs Folder", self.on_open_logs_folder),
        ):
            pad = (18, 6) if child[0] in {"Enable (mark)", "Up (Order)", "Apply", "Rollback…"} else (6, 6)
            ctk.CTkButton(
                right,
                text=child[0],
                command=child[1],
                width=180
            ).pack(fill=tk.X, padx=18, pady=pad)

        details_frame = ctk.CTkFrame(
            mods_tab,
            fg_color=("#f7f9fc", "#111820"),
            corner_radius=14,
            border_width=1,
            border_color=("#d3d9e3", "#28303a")
        )
        details_frame.grid(row=1, column=0, columnspan=2, sticky="nsew", pady=(16, 0))
        details_frame.grid_rowconfigure(1, weight=1)
        details_frame.grid_columnconfigure(0, weight=1)
        ctk.CTkLabel(
            details_frame,
            text="Mod details",
            font=("Segoe UI Semibold", 13)
        ).grid(row=0, column=0, sticky="w", padx=16, pady=(12, 0))
        self.details_text = ctk.CTkTextbox(details_frame, height=160)
        self.details_text.grid(row=1, column=0, sticky="nsew", padx=16, pady=(8, 16))
        self.tree.bind("<<TreeviewSelect>>", self.on_select_row)

        logs_tab.grid_rowconfigure(0, weight=1)
        logs_tab.grid_columnconfigure(0, weight=1)
        log_frame = ctk.CTkFrame(logs_tab, fg_color="transparent")
        log_frame.grid(row=0, column=0, sticky="nsew")
        log_frame.grid_rowconfigure(0, weight=1)
        log_frame.grid_columnconfigure(0, weight=1)
        self.log_text = ctk.CTkTextbox(log_frame, wrap="word")
        self.log_text.grid(row=0, column=0, sticky="nsew")
        log_scrollbar = ctk.CTkScrollbar(log_frame, command=self.log_text.yview)
        log_scrollbar.grid(row=0, column=1, sticky="ns")
        self.log_text.configure(yscrollcommand=log_scrollbar.set)

        # Footer
        footer = ctk.CTkFrame(self, corner_radius=0, fg_color=("#f4f6fb", "#0e141b"))
        footer.pack(fill=tk.X, padx=0, pady=(0, 0))
        footer.grid_columnconfigure(0, weight=1)
        ctk.CTkLabel(
            footer,
            text="Presented by JALCO / Justin Levine",
            anchor="w",
            font=("Segoe UI", 11)
        ).grid(row=0, column=0, sticky="w", padx=20, pady=12)

        social_frame = ctk.CTkFrame(footer, fg_color="transparent")
        social_frame.grid(row=0, column=1, sticky="e", padx=20)
        ctk.CTkButton(
            social_frame,
            text="Join Discord",
            width=140,
            command=lambda: webbrowser.open("https://discord.gg/AspRvTTAch")
        ).pack(side=tk.RIGHT, padx=(12, 0))
        ctk.CTkButton(
            social_frame,
            text="Support on Ko-fi",
            width=160,
            command=lambda: webbrowser.open("https://ko-fi.com/jalco")
        ).pack(side=tk.RIGHT)

    # ---- menu/button actions ----
    def _configure_treeview_style(self):
        style = ttk.Style(self)
        try:
            style.theme_use("clam")
        except tk.TclError:
            pass

        mode = (ctk.get_appearance_mode() or "Light").lower()
        if mode == "dark":
            bg = "#0f1115"
            fg = "#f4f6fb"
            border = "#1f2833"
            header_bg = "#1b2430"
            header_fg = "#f4f6fb"
            select_bg = "#1f6aa5"
            select_fg = "#ffffff"
        else:
            bg = "#ffffff"
            fg = "#1c1c1f"
            border = "#d5d9e2"
            header_bg = "#eef1f6"
            header_fg = "#1c1c1f"
            select_bg = "#1f6aa5"
            select_fg = "#ffffff"

        style.layout("CTk.Treeview", style.layout("Treeview"))
        style.configure(
            "CTk.Treeview",
            background=bg,
            fieldbackground=bg,
            foreground=fg,
            bordercolor=border,
            rowheight=36,
            font=("Segoe UI", 11),
        )
        style.configure(
            "CTk.Treeview.Heading",
            background=header_bg,
            foreground=header_fg,
            borderwidth=0,
            relief="flat",
            font=("Segoe UI Semibold", 11),
        )
        style.map(
            "CTk.Treeview",
            background=[("selected", select_bg)],
            foreground=[("selected", select_fg)],
        )
        style.map(
            "CTk.Treeview.Heading",
            background=[("active", header_bg)],
            relief=[("pressed", "flat")],
        )

    def on_open_logs_folder(self):
        safe_open_path(LOGS_DIR)

    def refresh_target_display(self):
        t = get_target()
        self.target_var.set(str(t) if t else "")

    def refresh_user_dir_display(self):
        """Update the user directory display field."""
        custom = get_user_dir()
        if custom:
            self.user_dir_var.set(f"{custom} (custom)")
        else:
            default = fm_user_dir()
            self.user_dir_var.set(f"{default} (default)")

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

    def on_set_user_dir(self):
        """Allow user to set custom FM user directory for tactics/graphics/skins."""
        chosen = filedialog.askdirectory(
            title="Select FM26 User Directory (contains tactics/graphics/skins folders)"
        )
        if not chosen:
            return
        p = Path(chosen).expanduser()
        if not p.exists():
            messagebox.showerror("Set User Directory", "Selected path does not exist.")
            return

        # Verify it looks like an FM user directory
        if "Football Manager" not in str(p):
            if not messagebox.askyesno(
                "Confirm",
                f"Selected folder does not contain 'Football Manager' in its path.\nUse anyway?\n\n{p}",
            ):
                return

        set_user_dir(p)
        self._log(f"Set user directory to: {p}")
        self.refresh_user_dir_display()
        messagebox.showinfo(
            "User Directory Set",
            f"User directory set to:\n{p}\n\nTactics, graphics, and other user mods will be installed here."
        )

    def on_reset_user_dir(self):
        """Reset user directory to default platform-specific path."""
        cfg = load_config()
        if "user_dir_path" in cfg:
            del cfg["user_dir_path"]
            save_config(cfg)
        self._log("Reset user directory to default")
        self.refresh_user_dir_display()
        messagebox.showinfo(
            "User Directory Reset",
            f"User directory reset to default:\n{fm_user_dir()}"
        )

    def on_open_user_dir(self):
        """Open the FM user directory."""
        user_dir = fm_user_dir()
        if not user_dir.exists():
            messagebox.showwarning(
                "Directory Not Found",
                f"User directory does not exist:\n{user_dir}\n\nPlease set a custom path if your Documents folder has been moved."
            )
            return
        safe_open_path(user_dir)

    def _choose_import_source(self) -> Path | None:
        """Choose import from ZIP, Folder, or single file, then show proper dialog."""
        # Create a custom dialog with 3 options
        dialog = tk.Toplevel(self)
        dialog.title("Import Mod")
        dialog.geometry("400x200")
        dialog.resizable(False, False)
        dialog.transient(self)
        dialog.grab_set()

        result = {"choice": None}

        ttk.Label(
            dialog,
            text="Select import source type:",
            font=("TkDefaultFont", 10, "bold")
        ).pack(pady=20)

        def choose_zip():
            result["choice"] = "zip"
            dialog.destroy()

        def choose_folder():
            result["choice"] = "folder"
            dialog.destroy()

        def choose_file():
            result["choice"] = "file"
            dialog.destroy()

        def cancel():
            result["choice"] = None
            dialog.destroy()

        button_frame = ttk.Frame(dialog)
        button_frame.pack(pady=10)

        ttk.Button(button_frame, text="ZIP File", command=choose_zip, width=20).pack(pady=5)
        ttk.Button(button_frame, text="Folder", command=choose_folder, width=20).pack(pady=5)
        ttk.Button(button_frame, text="Single File (.bundle/.fmf)", command=choose_file, width=20).pack(pady=5)
        ttk.Button(button_frame, text="Cancel", command=cancel, width=20).pack(pady=5)

        # Center on parent
        dialog.update_idletasks()
        x = self.winfo_x() + (self.winfo_width() // 2) - (dialog.winfo_width() // 2)
        y = self.winfo_y() + (self.winfo_height() // 2) - (dialog.winfo_height() // 2)
        dialog.geometry(f"+{x}+{y}")

        dialog.wait_window()

        if result["choice"] == "zip":
            path = filedialog.askopenfilename(
                title="Select Mod .zip", filetypes=[("Zip archives", "*.zip")]
            )
            return Path(path) if path else None
        elif result["choice"] == "folder":
            folder = filedialog.askdirectory(
                title="Select Mod Folder"
            )
            return Path(folder) if folder else None
        elif result["choice"] == "file":
            path = filedialog.askopenfilename(
                title="Select Mod File",
                filetypes=[
                    ("FM Mod Files", "*.bundle *.fmf"),
                    ("Bundle Files", "*.bundle"),
                    ("Tactic Files", "*.fmf"),
                    ("All Files", "*.*")
                ]
            )
            return Path(path) if path else None
        else:
            return None
    def on_import_mod(self):
        if is_fm_running():
            messagebox.showwarning("FM is Running", "Please close Football Manager before importing mods.")
            return
        choice = self._choose_import_source()
        if not choice:
            return
        mod_root, temp_dir = _find_mod_root(choice)
        try:
            # Check if manifest exists
            has_manifest = _has_manifest(mod_root) if mod_root.is_dir() else False

            generated_manifest = None

            if not has_manifest:
                # Auto-detect mod type
                auto_type = _auto_detect_mod_type(mod_root)

                # Show metadata dialog
                self._log(f"No manifest.json found. Opening metadata dialog...")
                dialog = ModMetadataDialog(self, mod_root, auto_type)
                metadata = dialog.get_result()

                if not metadata:
                    # User cancelled
                    self._log("Import cancelled by user.")
                    return

                # Generate manifest
                generated_manifest = _generate_manifest(mod_root, metadata)
                self._log(f"Generated manifest for mod '{metadata['name']}' (type: {metadata['type']})")

            newname = install_mod_from_folder(mod_root, None, log=self._log, generated_manifest=generated_manifest)
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

    def on_drop(self, event):
        """Handle drag-and-drop events. Defers processing to avoid UI freezing."""
        raw = event.data.strip()
        if raw.startswith("{") and raw.endswith("}"):
            raw = raw[1:-1]
        path = Path(raw)
        if not path.exists():
            return

        # Defer processing to let the drop event complete and avoid beach ball
        self.after(100, lambda: self._process_dropped_file(path))

    def _process_dropped_file(self, path: Path):
        """Process a dropped file/folder after the drop event has completed."""
        if is_fm_running():
            messagebox.showwarning(
                "FM is Running", "Please close Football Manager before importing mods."
            )
            return

        mod_root, temp_dir = _find_mod_root(path)
        try:
            # Check if manifest exists
            has_manifest = _has_manifest(mod_root) if mod_root.is_dir() else False

            generated_manifest = None

            if not has_manifest:
                # Auto-detect mod type
                auto_type = _auto_detect_mod_type(mod_root)

                # Show metadata dialog
                self._log(f"No manifest.json found in dropped file/folder. Opening metadata dialog...")
                dialog = ModMetadataDialog(self, mod_root, auto_type)
                metadata = dialog.get_result()

                if not metadata:
                    # User cancelled
                    self._log("Import cancelled by user.")
                    return

                # Generate manifest
                generated_manifest = _generate_manifest(mod_root, metadata)
                self._log(f"Generated manifest for mod '{metadata['name']}' (type: {metadata['type']})")

            newname = install_mod_from_folder(mod_root, None, log=self._log, generated_manifest=generated_manifest)
            order = get_load_order()
            if newname not in order:
                order.append(newname)
                set_load_order(order)
            self.refresh_mod_list()
            messagebox.showinfo("Import", f"Imported '{newname}' via drag-and-drop.")
        except Exception as e:
            messagebox.showerror("Import Error", str(e))
        finally:
            if temp_dir:
                shutil.rmtree(temp_dir, ignore_errors=True)


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

    def on_remove_selected(self):
        name = self.selected_mod_name()
        if not name:
            messagebox.showinfo("Remove Mod", "Select a mod first.")
            return

        # Confirm deletion
        result = messagebox.askyesno(
            "Remove Mod",
            f"Are you sure you want to permanently delete '{name}'?\n\n"
            "This will remove the mod from your mods folder.\n"
            "This action cannot be undone.",
            icon='warning'
        )

        if not result:
            return

        try:
            remove_mod(name, self._log)
            self.refresh_mod_list()
            messagebox.showinfo("Remove Mod", f"Successfully removed '{name}'.")
        except Exception as e:
            messagebox.showerror("Remove Mod Error", f"Failed to remove mod:\n{e}")

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
        win = ctk.CTkToplevel(self)
        win.title("Conflict Manager — FM26 Mod Manager")
        win.geometry("780x580")
        win.transient(self)

        frame = ctk.CTkFrame(win, corner_radius=16)
        frame.pack(fill=tk.BOTH, expand=True, padx=16, pady=16)
        frame.grid_columnconfigure(0, weight=1)
        frame.grid_rowconfigure(1, weight=1)

        header = ctk.CTkLabel(
            frame,
            text="Detected conflicts where multiple mods write the same files.",
            font=("Segoe UI", 12)
        )
        header.grid(row=0, column=0, sticky="w", pady=(0, 12))

        text = ctk.CTkTextbox(frame, wrap="word", height=220)
        text.grid(row=1, column=0, sticky="nsew")
        sb = ctk.CTkScrollbar(frame, command=text.yview)
        sb.grid(row=1, column=1, sticky="ns", padx=(8, 0))
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

        ctk.CTkLabel(
            frame,
            text="Select mods to disable:",
            font=("Segoe UI Semibold", 12)
        ).grid(row=2, column=0, sticky="w", pady=(16, 6))

        # Checkbox area
        mods_to_disable = {}
        box_frame = ctk.CTkScrollableFrame(frame, height=140, corner_radius=12)
        box_frame.grid(row=3, column=0, columnspan=2, sticky="nsew")
        unique_mods = sorted(set([mm for ms in conflicts.values() for mm in ms]))
        for m in unique_mods:
            var = tk.BooleanVar()
            mods_to_disable[m] = var
            ctk.CTkCheckBox(box_frame, text=m, variable=var).pack(anchor="w", pady=2)

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

        bframe = ctk.CTkFrame(frame, fg_color="transparent")
        bframe.grid(row=4, column=0, sticky="e", pady=(16, 0))
        ctk.CTkButton(
            bframe,
            text="Close",
            width=140,
            command=win.destroy
        ).pack(side=tk.RIGHT, padx=(12, 0))
        ctk.CTkButton(
            bframe,
            text="Disable Selected Mods",
            width=200,
            command=apply_disables
        ).pack(side=tk.RIGHT)

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

        win = ctk.CTkToplevel(self)
        win.title("Choose Restore Point")
        win.geometry("440x440")
        win.transient(self)

        container = ctk.CTkFrame(win, corner_radius=16)
        container.pack(fill=tk.BOTH, expand=True, padx=16, pady=16)
        container.grid_columnconfigure(0, weight=1)

        ctk.CTkLabel(
            container,
            text="Select a restore point to roll back to.",
            font=("Segoe UI", 12)
        ).grid(row=0, column=0, sticky="w", pady=(0, 12))

        list_frame = ctk.CTkFrame(container, fg_color="transparent")
        list_frame.grid(row=1, column=0, sticky="nsew")
        list_frame.grid_columnconfigure(0, weight=1)
        list_frame.grid_rowconfigure(0, weight=1)
        lb = tk.Listbox(list_frame, height=min(16, len(rps)))
        for rp in rps:
            lb.insert(tk.END, rp)
        lb.grid(row=0, column=0, sticky="nsew")
        lb.config(borderwidth=0, highlightthickness=0)
        lb_scroll = ctk.CTkScrollbar(list_frame, command=lb.yview)
        lb_scroll.grid(row=0, column=1, sticky="ns", padx=(8, 0))
        lb.configure(yscrollcommand=lb_scroll.set)

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

        btn_frame = ctk.CTkFrame(container, fg_color="transparent")
        btn_frame.grid(row=2, column=0, sticky="e", pady=(16, 0))
        ctk.CTkButton(btn_frame, text="Cancel", width=120, command=win.destroy).pack(side=tk.RIGHT, padx=(12, 0))
        ctk.CTkButton(btn_frame, text="Rollback to selected", width=200, command=do_rb).pack(side=tk.RIGHT)

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
