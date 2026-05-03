//! CSV metadata management – class list and annotation CSV downloading.
//!
//! No Tauri dependency; just plain Rust + reqwest/tokio.

use std::path::{Path, PathBuf};

use crate::logic::models::ClassEntry;

const CLASSES_URL: &str =
    "https://storage.googleapis.com/openimages/v5/class-descriptions-boxable.csv";

const ANNOTATION_URLS: &[(&str, &str)] = &[
    (
        "train-annotations-bbox.csv",
        "https://storage.googleapis.com/openimages/v6/oidv6-class-descriptions.csv",
    ),
    (
        "train-annotations-bbox.csv",
        "https://storage.googleapis.com/openimages/v6/oidv6-class-descriptions-boxable.csv",
    ),
];

// Real annotation download URLs (same as Python toolkit)
const ANNOTATION_SPLIT_URLS: &[(&str, &str)] = &[
    (
        "train-annotations-bbox.csv",
        "https://storage.googleapis.com/openimages/v6/oidv6-class-descriptions-boxable.csv",
    ),
    (
        "validation-annotations-bbox.csv",
        "https://storage.googleapis.com/openimages/v5/validation-annotations-bbox.csv",
    ),
    (
        "test-annotations-bbox.csv",
        "https://storage.googleapis.com/openimages/v5/test-annotations-bbox.csv",
    ),
];

/// Returns the csv_folder path inside the app data directory.
pub fn csv_dir(app_data: &Path) -> PathBuf {
    app_data.join("csv_folder")
}

/// Parse the class-descriptions CSV and return a sorted Vec of ClassEntry.
pub fn load_classes(csv_dir: &Path) -> Result<Vec<ClassEntry>, String> {
    let path = csv_dir.join("class-descriptions-boxable.csv");
    if !path.exists() {
        return Ok(vec![]); // not downloaded yet
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read class-descriptions-boxable.csv: {}", e))?;

    let mut entries: Vec<ClassEntry> = content
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ',');
            let code = parts.next()?.trim().to_string();
            let name = parts
                .next()?
                .trim()
                .trim_matches('"')
                .to_string();
            Some(ClassEntry { code, name })
        })
        .collect();

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Download the class-descriptions CSV if not present, then return all classes.
pub async fn ensure_classes(csv_dir: &Path) -> Result<Vec<ClassEntry>, String> {
    std::fs::create_dir_all(csv_dir).map_err(|e| e.to_string())?;

    let dest = csv_dir.join("class-descriptions-boxable.csv");
    if !dest.exists() {
        let client = reqwest::Client::new();
        let bytes = client
            .get(CLASSES_URL)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch classes CSV: {}", e))?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    }

    load_classes(csv_dir)
}

/// Download annotation CSVs for the given split names.
pub async fn ensure_annotation_csvs(
    csv_dir: &Path,
    split: &crate::logic::models::Split,
) -> Result<(), String> {
    std::fs::create_dir_all(csv_dir).map_err(|e| e.to_string())?;

    let client = reqwest::Client::new();
    for ann_file in split.annotation_files() {
        let dest = csv_dir.join(ann_file);
        if dest.exists() {
            continue;
        }
        // Find the URL for this annotation file
        let url = match ann_file {
            "train-annotations-bbox.csv" => {
                "https://storage.googleapis.com/openimages/v6/oidv6-class-descriptions-boxable.csv"
            }
            "validation-annotations-bbox.csv" => {
                "https://storage.googleapis.com/openimages/v5/validation-annotations-bbox.csv"
            }
            "test-annotations-bbox.csv" => {
                "https://storage.googleapis.com/openimages/v5/test-annotations-bbox.csv"
            }
            _ => return Err(format!("Unknown annotation file: {}", ann_file)),
        };

        let bytes = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch {}: {}", ann_file, e))?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    }

    Ok(())
}
