export interface NameFixSource {
  id: string;
  name: string;
  source_type: 'GitHub' | 'Imported';
  install_type: 'Files' | 'Folders';
  description: string;
  imported_date: string;
}
