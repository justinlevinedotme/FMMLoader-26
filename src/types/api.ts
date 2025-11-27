export interface ConflictInfo {
  file_path: string;
  conflicting_mods: string[];
}

export interface RestorePoint {
  name: string;
  timestamp: string;
  path: string;
}
