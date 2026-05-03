//! Local dataset index — no Tauri dependency.

use std::path::{Path, PathBuf};

use crate::logic::models::{ImageEntry, YoloBox};

pub(crate) fn parse_yolo_labels(label_path: &Path) -> Vec<YoloBox> {
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

/// Scan the local dataset directory and return all `ImageEntry` records.
///
/// Expected layout:
///   <dataset_dir>/<split>/<class_name>/*.jpg
///   <dataset_dir>/<split>/<class_name>/Label/*.txt
pub fn index_dataset(dataset_dir: &Path) -> Result<Vec<ImageEntry>, String> {
    if !dataset_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<ImageEntry> = Vec::new();

    for split_entry in std::fs::read_dir(dataset_dir).map_err(|e| e.to_string())? {
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

            if class_name == "Label" {
                continue;
            }

            let label_dir = class_path.join("Label");

            for img_entry in std::fs::read_dir(&class_path).map_err(|e| e.to_string())? {
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
                    (Some(label_file.to_string_lossy().into_owned()), boxes)
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

    entries.sort_by(|a, b| {
        a.split
            .cmp(&b.split)
            .then(a.class_name.cmp(&b.class_name))
            .then(a.image_id.cmp(&b.image_id))
    });

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_yolo_labels_valid() {
        let dir = tempfile::tempdir().unwrap();
        let label_path = dir.path().join("test.txt");
        let mut f = std::fs::File::create(&label_path).unwrap();
        writeln!(f, "0 0.5 0.5 0.4 0.3").unwrap();
        writeln!(f, "1 0.2 0.8 0.1 0.2").unwrap();

        let boxes = parse_yolo_labels(&label_path);
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].class_idx, 0);
        assert!((boxes[0].cx - 0.5).abs() < 1e-6);
        assert_eq!(boxes[1].class_idx, 1);
    }

    #[test]
    fn test_parse_yolo_labels_missing_file() {
        let boxes = parse_yolo_labels(std::path::Path::new("/nonexistent/path.txt"));
        assert!(boxes.is_empty());
    }

    #[test]
    fn test_parse_yolo_labels_malformed() {
        let dir = tempfile::tempdir().unwrap();
        let label_path = dir.path().join("bad.txt");
        let mut f = std::fs::File::create(&label_path).unwrap();
        writeln!(f, "0 0.5 0.5").unwrap();
        writeln!(f, "not_a_number 0.5 0.5 0.4 0.3").unwrap();

        let boxes = parse_yolo_labels(&label_path);
        assert!(boxes.is_empty());
    }
}
