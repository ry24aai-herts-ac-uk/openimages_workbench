//! CSV metadata commands
//!
//! Mirrors the Python `csv_downloader.py` module.
//!
//! Open Images stores annotation CSV files and a class-description CSV at:
//!   https://storage.googleapis.com/openimages/2018_04/<folder>/<file>
//! or directly at the root for `class-descriptions-boxable.csv`.
//!
//! This module provides Tauri commands to:
//!   - Download and cache those files locally.
//!   - Parse class descriptions and return a searchable list to the UI.
//!   - Resolve a human-readable class name → Open Images label code (/m/…).

use std::path::{Path, PathBuf};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::io::AsyncWriteExt;

const OID_BASE_URL: &str = "https://storage.googleapis.com/openimages/2018_04/";

/// A single class entry returned to the UI for the class-search box.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassEntry {
    /// Open Images label code, e.g. "/m/01g317"
    pub code: String,
    /// Human-readable display name, e.g. "Person"
    pub name: String,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn csv_dir(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("OID"));
    base.join("csv_folder")
}

async fn ensure_csv(
    client: &Client,
    csv_dir: &Path,
    file_name: &str,
) -> Result<PathBuf, String> {
    tokio::fs::create_dir_all(csv_dir)
        .await
        .map_err(|e| e.to_string())?;

    let dest = csv_dir.join(file_name);
    if dest.exists() {
        return Ok(dest);
    }

    // Derive the sub-folder portion of the URL (same logic as the Python code)
    let folder = file_name.split('-').next().unwrap_or("");
    let url = if folder == "class" {
        format!("{}{}", OID_BASE_URL, file_name)
    } else {
        format!("{}{}/{}", OID_BASE_URL, folder, file_name)
    };

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP error downloading {}: {}", file_name, e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to download {}: HTTP {}",
            file_name,
            resp.status()
        ));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Error reading {}: {}", file_name, e))?;

    let mut f = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| e.to_string())?;
    f.write_all(&bytes).await.map_err(|e| e.to_string())?;

    Ok(dest)
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Download (if missing) and return all class entries from
/// `class-descriptions-boxable.csv`.
///
/// Used by the UI to populate the class-search dropdown.
#[tauri::command]
pub async fn list_classes(app: AppHandle) -> Result<Vec<ClassEntry>, String> {
    let client = Client::new();
    let dir = csv_dir(&app);
    let path = ensure_csv(&client, &dir, "class-descriptions-boxable.csv").await?;

    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| e.to_string())?;

    let mut entries: Vec<ClassEntry> = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() == 2 {
            entries.push(ClassEntry {
                code: parts[0].trim().to_string(),
                name: parts[1].trim().trim_matches('"').to_string(),
            });
        }
    }

    // Sort alphabetically by name for usability
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Ensure the annotation bbox CSV files are cached locally for `split`.
///
/// Returns the list of local file paths that were downloaded / already present.
#[tauri::command]
pub async fn ensure_annotation_csvs(
    app: AppHandle,
    split: String,
) -> Result<Vec<String>, String> {
    let client = Client::new();
    let dir = csv_dir(&app);

    let files: Vec<&str> = match split.as_str() {
        "train" => vec!["train-annotations-bbox.csv"],
        "validation" => vec!["validation-annotations-bbox.csv"],
        "test" => vec!["test-annotations-bbox.csv"],
        _ => vec![
            "train-annotations-bbox.csv",
            "validation-annotations-bbox.csv",
            "test-annotations-bbox.csv",
        ],
    };

    let mut paths: Vec<String> = Vec::new();
    for file in files {
        let path = ensure_csv(&client, &dir, file).await?;
        paths.push(path.to_string_lossy().into_owned());
    }
    Ok(paths)
}

/// Return the local cache directory used for CSV files (for display in UI).
#[tauri::command]
pub fn csv_cache_dir(app: AppHandle) -> String {
    csv_dir(&app).to_string_lossy().into_owned()
}
