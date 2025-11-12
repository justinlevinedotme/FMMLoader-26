import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

// Wait for Tauri to be ready
const waitForTauri = async (timeout = 5000): Promise<boolean> => {
  const startTime = Date.now();
  while (Date.now() - startTime < timeout) {
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      return true;
    }
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  return false;
};

// Check if we're running in a Tauri context
const isTauri = () => {
  return typeof window !== 'undefined' && '__TAURI__' in window;
};

// Wrapper to ensure we're in Tauri context
const safeInvoke = async <T>(
  cmd: string,
  args?: InvokeArgs
): Promise<T> => {
  if (!isTauri()) {
    // Try waiting for Tauri to load
    const ready = await waitForTauri();
    if (!ready) {
      throw new Error('Not running in Tauri context. Please run with "npm run tauri dev"');
    }
  }
  return invoke<T>(cmd, args);
};

export interface Config {
  target_path?: string;
  user_dir_path?: string;
  enabled_mods: string[];
  dark_mode?: boolean;
}

export interface ModManifest {
  name: string;
  version: string;
  mod_type: string;
  author: string;
  homepage: string;
  description: string;
  license: string;
  compatibility: {
    fm_version: string;
  };
  dependencies: string[];
  conflicts: string[];
  load_after: string[];
  files: FileEntry[];
}

export interface FileEntry {
  source: string;
  target_subpath: string;
  platform?: string;
}

export interface ConflictInfo {
  file_path: string;
  conflicting_mods: string[];
}

export interface RestorePoint {
  timestamp: string;
  path: string;
}

export interface ModMetadata {
  name?: string;
  version?: string;
  mod_type?: string;
  author?: string;
  description?: string;
}

export const tauriCommands = {
  initApp: () => safeInvoke<void>("init_app"),

  getAppVersion: () => safeInvoke<string>("get_app_version"),

  getConfig: () => safeInvoke<Config>("get_config"),

  updateConfig: (config: Config) => safeInvoke<void>("update_config", { config }),

  detectGamePath: () => safeInvoke<string[]>("detect_game_path"),

  setGameTarget: (path: string) => safeInvoke<void>("set_game_target", { path }),

  detectUserDir: () => safeInvoke<string>("detect_user_dir"),

  getModsList: () => safeInvoke<string[]>("get_mods_list"),

  getModDetails: (modName: string) =>
    safeInvoke<ModManifest>("get_mod_details", { modName }),

  enableMod: (modName: string) => safeInvoke<void>("enable_mod", { modName }),

  disableMod: (modName: string) => safeInvoke<void>("disable_mod", { modName }),

  applyMods: () => safeInvoke<string>("apply_mods"),

  removeMod: (modName: string) => safeInvoke<void>("remove_mod", { modName }),

  importMod: (
    sourcePath: string,
    metadata?: ModMetadata
  ) =>
    safeInvoke<string>("import_mod", {
      sourcePath,
      modName: metadata?.name,
      version: metadata?.version,
      modType: metadata?.mod_type,
      author: metadata?.author,
      description: metadata?.description,
    }),

  detectModType: (path: string) => safeInvoke<string>("detect_mod_type", { path }),

  checkConflicts: () => safeInvoke<ConflictInfo[]>("check_conflicts"),

  getRestorePoints: () => safeInvoke<RestorePoint[]>("get_restore_points"),

  restoreFromPoint: (pointPath: string) =>
    safeInvoke<string>("restore_from_point", { pointPath }),

  createBackupPoint: (name: string) =>
    safeInvoke<string>("create_backup_point", { name }),

  openLogsFolder: () => safeInvoke<void>("open_logs_folder"),

  getLogsPath: () => safeInvoke<string>("get_logs_path"),

  logUpdateEvent: (
    eventType: string,
    currentVersion: string,
    latestVersion: string | null,
    message: string,
    details?: string
  ) => safeInvoke<void>("log_update_event", {
    eventType,
    currentVersion,
    latestVersion,
    message,
    details,
  }),

  // FM Name Fix commands
  checkNameFixInstalled: () => safeInvoke<boolean>("check_name_fix_installed"),

  installNameFix: () => safeInvoke<string>("install_name_fix"),

  uninstallNameFix: () => safeInvoke<string>("uninstall_name_fix"),
};
