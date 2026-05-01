pub mod commands;
pub mod models;

use commands::{csv_meta, download, index};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // CSV metadata
            csv_meta::list_classes,
            csv_meta::ensure_annotation_csvs,
            csv_meta::csv_cache_dir,
            // Download + label generation
            download::start_download,
            // Local dataset index
            index::index_dataset,
            index::list_local_classes,
            index::get_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
