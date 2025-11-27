export interface Config {
  target_path?: string;
  user_dir_path?: string;
  enabled_mods: string[];
  dark_mode?: boolean;
  language?: string;
}

export interface ExtractionProgress {
  current: number;
  total: number;
  current_file: string;
  bytes_processed: number;
  phase: string;
}
