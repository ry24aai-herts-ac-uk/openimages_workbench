//! Local dataset index command
//!
//! Walks the local dataset directory and returns a list of `ImageEntry` structs
//! so the SvelteKit UI can render a browsable viewer without reading from disk
//! on every UI interaction.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::models::{ImageEntry, YoloBox};

fn app_data_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("OID"))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse a YOLO `.txt` label file into a Vec of `YoloBox`.
fn parse_yolo_labels(label_path: &std::path::Path) -> Vec<YoloBox> {
    let content = match std::fs::read_to_string(label_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                return None;
            }
            let class_idx: u32 = parts[0].parse().ok()?;
            let cx: f32 = parts[1].parse().ok()?;
            let cy: f32 = parts[2].parse().ok()?;
            let w: f32 = parts[3].parse().ok()?;
            let h: f32 = parts[4].parse().ok()?;
            Some(YoloBox { class_idx, cx, cy, w, h })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Scan the local dataset directory and return all `ImageEntry` records.
///
/// Expected layout:
///   <dataset_dir>/<split>/<class_name>/*.jpg
///   <dataset_dir>/<split>/<class_name>/Label/*.txt
///
/// `dataset_dir` defaults to `<app_data>/Dataset` if not provided.
#[tauri::command]
pub fn index_dataset(
    app: AppHandle,
    dataset_dir: Option<String>,
) -> Result<Vec<ImageEntry>, String> {
    let base = match dataset_dir {
        Some(d) => PathBuf::from(d),
        None => app_data_dir(&app).join("Dataset"),
    };

    if !base.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<ImageEntry> = Vec::new();

    // split-level
    for split_entry in std::fs::read_dir(&base).map_err(|e| e.to_string())? {
        let split_entry = split_entry.map_err(|e| e.to_string())?;
        let split_path = split_entry.path();
        if !split_path.is_dir() {
            continue;
        }
        let split_name = split_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        // class-level
        for class_entry in std::fs::read_dir(&split_path).map_err(|e| e.to_string())? {
            let class_entry = class_entry.map_err(|e| e.to_string())?;
            let class_path = class_entry.path();
            if !class_path.is_dir() {
                continue;
            }
            let class_name = class_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            // Skip the Label sub-directory itself
            if class_name == "Label" {
                continue;
            }

            let label_dir = class_path.join("Label");

            // image files
            for img_entry in
                std::fs::read_dir(&class_path).map_err(|e| e.to_string())?
            {
                let img_entry = img_entry.map_err(|e| e.to_string())?;
                let img_path = img_entry.path();
                if img_path.extension().map(|e| e != "jpg").unwrap_or(true) {
                    continue;
                }

                let image_id = img_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();

                let label_file = label_dir.join(format!("{}.txt", image_id));
                let (label_path_opt, boxes) = if label_file.exists() {
                    let boxes = parse_yolo_labels(&label_file);
                    (
                        Some(label_file.to_string_lossy().into_owned()),
                        boxes,
                    )
                } else {
                    (None, vec![])
                };

                entries.push(ImageEntry {
                    path: img_path.to_string_lossy().into_owned(),
                    image_id,
                    class_name: class_name.clone(),
                    split: split_name.clone(),
                    label_path: label_path_opt,
                    boxes,
                });
            }
        }
    }

    // Sort by split → class → image_id for a stable, predictable order
    entries.sort_by(|a, b| {
        a.split
            .cmp(&b.split)
            .then(a.class_name.cmp(&b.class_name))
            .then(a.image_id.cmp(&b.image_id))
    });

    Ok(entries)
}

/// Return all distinct class names present in the local dataset.
#[tauri::command]
pub fn list_local_classes(
    app: AppHandle,
    dataset_dir: Option<String>,
) -> Result<Vec<String>, String> {
    let entries = index_dataset(app, dataset_dir)?;
    let mut names: Vec<String> = entries
        .iter()
        .map(|e| e.class_name.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    names.sort();
    Ok(names)
}

/// Return the app data directory path (useful for the UI to display paths).
#[tauri::command]
pub fn get_data_dir(app: AppHandle) -> String {
    app_data_dir(&app).to_string_lossy().into_owned()
}
