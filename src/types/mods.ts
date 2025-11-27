export interface FileEntry {
  source: string;
  target_subpath: string;
  platform?: string;
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

export interface ModMetadata {
  name?: string;
  version?: string;
  mod_type?: string;
  author?: string;
  description?: string;
}

export interface ResolvedFilePreview {
  target_subpath: string;
  resolved_path: string;
}

export interface ModInstallPreview {
  base_target: string;
  resolved_files: ResolvedFilePreview[];
}
