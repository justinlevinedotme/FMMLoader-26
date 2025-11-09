import { invoke } from "@tauri-apps/api/core";

export interface Config {
  target_path?: string;
  user_dir_path?: string;
  enabled_mods: string[];
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

export const tauriCommands = {
  initApp: () => invoke<void>("init_app"),

  getConfig: () => invoke<Config>("get_config"),

  updateConfig: (config: Config) => invoke<void>("update_config", { config }),

  detectGamePath: () => invoke<string[]>("detect_game_path"),

  setGameTarget: (path: string) => invoke<void>("set_game_target", { path }),

  getModsList: () => invoke<string[]>("get_mods_list"),

  getModDetails: (modName: string) =>
    invoke<ModManifest>("get_mod_details", { modName }),

  enableMod: (modName: string) => invoke<void>("enable_mod", { modName }),

  disableMod: (modName: string) => invoke<void>("disable_mod", { modName }),

  applyMods: () => invoke<string>("apply_mods"),

  removeMod: (modName: string) => invoke<void>("remove_mod", { modName }),
};
