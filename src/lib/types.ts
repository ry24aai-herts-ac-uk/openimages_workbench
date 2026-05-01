// Shared TypeScript types matching the Rust models in src-tauri/src/models.rs

export type Split = 'train' | 'validation' | 'test' | 'all';

export interface ImageFilters {
  is_occluded?: boolean;
  is_truncated?: boolean;
  is_group_of?: boolean;
  is_depiction?: boolean;
  is_inside?: boolean;
}

export interface DownloadConfig {
  classes: string[];
  split: Split;
  multiclass: boolean;
  limit?: number;
  threads: number;
  no_labels: boolean;
  filters: ImageFilters;
  dataset_dir?: string;
  yes: boolean;
}

export interface ClassEntry {
  code: string;
  name: string;
}

export interface YoloBox {
  class_idx: number;
  cx: number;
  cy: number;
  w: number;
  h: number;
}

export interface ImageEntry {
  path: string;
  image_id: string;
  class_name: string;
  split: string;
  label_path?: string;
  boxes: YoloBox[];
}

export interface ProgressEvent {
  job_id: string;
  class_name: string;
  split: string;
  downloaded: number;
  total: number;
  finished: boolean;
  error?: string;
}

export interface DownloadJob {
  job_id: string;
  class_name: string;
  split: string;
  downloaded: number;
  total: number;
  finished: boolean;
  errors: string[];
  started_at: number;
}
