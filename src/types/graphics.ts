export type GraphicsPackType =
  | 'Faces'
  | 'Logos'
  | 'Kits'
  | { Mixed: GraphicsPackType[] }
  | 'Unknown';

export interface GraphicsPackMetadata {
  id: string;
  name: string;
  install_date: string;
  file_count: number;
  source_filename: string;
  pack_type: string;
  installed_to: string;
}

export interface GraphicsPackAnalysis {
  pack_type: GraphicsPackType;
  confidence: number;
  suggested_paths: string[];
  file_count: number;
  total_size_bytes: number;
  has_config_xml: boolean;
  subdirectory_breakdown: Record<string, number>;
  is_flat_pack: boolean;
}

export interface GraphicsPackIssue {
  pack_name: string;
  current_path: string;
  suggested_path: string;
  reason: string;
  pack_type: string;
}

export interface GraphicsConflictInfo {
  target_directory: string;
  existing_file_count: number;
  pack_name: string;
}
