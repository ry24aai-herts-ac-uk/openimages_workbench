//! Image download + YOLO label generation — no Tauri dependency.
//!
//! Progress is reported through a `tokio::sync::mpsc::Sender<ProgressEvent>`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use csv::ReaderBuilder;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use tokio::io::AsyncWriteExt;

use futures::channel::mpsc::UnboundedSender;

use crate::logic::models::{BBoxRow, DownloadConfig, ImageFilters, ProgressEvent};

const S3_BASE: &str = "https://open-images-dataset.s3.amazonaws.com";
const CLASSES_FILE: &str = "class-descriptions-boxable.csv";

// ---------------------------------------------------------------------------
// Class map helpers
// ---------------------------------------------------------------------------

fn load_class_map(csv_dir: &Path) -> Result<HashMap<String, String>, String> {
    let path = csv_dir.join(CLASSES_FILE);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read {}: {}", CLASSES_FILE, e))?;

    let mut map = HashMap::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() == 2 {
            let code = parts[0].trim().to_string();
            let name = parts[1].trim().trim_matches('"').to_string();
            map.insert(name, code);
        }
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// BBox CSV parsing
// ---------------------------------------------------------------------------

fn load_bbox_rows(
    csv_path: &Path,
    class_code: &str,
    filters: &ImageFilters,
) -> Result<Vec<BBoxRow>, String> {
    let file = std::fs::File::open(csv_path)
        .map_err(|e| format!("Cannot open {:?}: {}", csv_path, e))?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(file);

    let headers = reader.headers().map_err(|e| e.to_string())?.clone();
    let col = |name: &str| -> Option<usize> { headers.iter().position(|h| h == name) };

    let idx_image = col("ImageID").ok_or("Missing ImageID column")?;
    let idx_label = col("LabelName").ok_or("Missing LabelName column")?;
    let idx_xmin = col("XMin").ok_or("Missing XMin column")?;
    let idx_xmax = col("XMax").ok_or("Missing XMax column")?;
    let idx_ymin = col("YMin").ok_or("Missing YMin column")?;
    let idx_ymax = col("YMax").ok_or("Missing YMax column")?;

    let idx_occluded = col("IsOccluded");
    let idx_truncated = col("IsTruncated");
    let idx_group_of = col("IsGroupOf");
    let idx_depiction = col("IsDepiction");
    let idx_inside = col("IsInside");

    let parse_flag = |record: &csv::StringRecord, idx: Option<usize>| -> Option<i8> {
        idx.and_then(|i| record.get(i))
            .and_then(|v| v.parse::<i8>().ok())
    };

    let mut rows: Vec<BBoxRow> = Vec::new();

    for result in reader.records() {
        let r = result.map_err(|e| e.to_string())?;

        let label = r.get(idx_label).unwrap_or("").trim();
        if label != class_code {
            continue;
        }

        let is_occluded = parse_flag(&r, idx_occluded);
        let is_truncated = parse_flag(&r, idx_truncated);
        let is_group_of = parse_flag(&r, idx_group_of);
        let is_depiction = parse_flag(&r, idx_depiction);
        let is_inside = parse_flag(&r, idx_inside);

        if let Some(want) = filters.is_occluded {
            if is_occluded != Some(want as i8) { continue; }
        }
        if let Some(want) = filters.is_truncated {
            if is_truncated != Some(want as i8) { continue; }
        }
        if let Some(want) = filters.is_group_of {
            if is_group_of != Some(want as i8) { continue; }
        }
        if let Some(want) = filters.is_depiction {
            if is_depiction != Some(want as i8) { continue; }
        }
        if let Some(want) = filters.is_inside {
            if is_inside != Some(want as i8) { continue; }
        }

        let image_id = r.get(idx_image).unwrap_or("").trim().to_string();

        let parse_coord = |idx: usize, col_name: &str| -> Option<f32> {
            let raw = r.get(idx).unwrap_or("").trim();
            raw.parse::<f32>().map_err(|_| {
                eprintln!(
                    "[WARN] Invalid coordinate '{}' in '{}' for image '{}'. Skipped.",
                    raw, col_name, image_id
                );
            }).ok()
        };

        let (x_min, x_max, y_min, y_max) = match (
            parse_coord(idx_xmin, "XMin"),
            parse_coord(idx_xmax, "XMax"),
            parse_coord(idx_ymin, "YMin"),
            parse_coord(idx_ymax, "YMax"),
        ) {
            (Some(x1), Some(x2), Some(y1), Some(y2)) => (x1, x2, y1, y2),
            _ => continue,
        };

        rows.push(BBoxRow {
            image_id,
            label_name: class_code.to_string(),
            x_min,
            x_max,
            y_min,
            y_max,
            is_occluded,
            is_truncated,
            is_group_of,
            is_depiction,
            is_inside,
        });
    }

    Ok(rows)
}

fn xyxy_to_xywh(x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32, f32, f32) {
    let cx = (x1 + x2) / 2.0;
    let cy = (y1 + y2) / 2.0;
    let w = x2 - x1;
    let h = y2 - y1;
    (cx, cy, w, h)
}

fn write_yolo_label(
    label_dir: &Path,
    image_id: &str,
    class_idx: usize,
    boxes: &[BBoxRow],
) -> Result<(), String> {
    std::fs::create_dir_all(label_dir).map_err(|e| e.to_string())?;
    let label_path = label_dir.join(format!("{}.txt", image_id));

    let mut content = if label_path.exists() {
        std::fs::read_to_string(&label_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    for row in boxes {
        let (cx, cy, w, h) = xyxy_to_xywh(row.x_min, row.y_min, row.x_max, row.y_max);
        content.push_str(&format!("{} {} {} {} {}\n", class_idx, cx, cy, w, h));
    }

    std::fs::write(&label_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

async fn download_image(
    client: &Client,
    split_folder: &str,
    image_id: &str,
    dest_dir: &Path,
) -> Result<bool, String> {
    let dest = dest_dir.join(format!("{}.jpg", image_id));
    if dest.exists() {
        return Ok(false);
    }

    let url = format!("{}/{}/{}.jpg", S3_BASE, split_folder, image_id);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GET {} failed: {}", url, e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {}", resp.status(), url));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Read error for {}: {}", image_id, e))?;

    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;

    Ok(true)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a unique job ID from timestamp nanoseconds.
pub fn generate_job_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("job-{}", ts)
}

/// Run the download job.  Progress events are sent via `tx`.
///
/// Returns the job_id on success, or an error string.
pub async fn run_download(
    config: DownloadConfig,
    csv_dir: PathBuf,
    dataset_dir: PathBuf,
    tx: tokio::sync::mpsc::UnboundedSender<ProgressEvent>,
    job_id: String,
) -> Result<(), String> {
    let client = Client::new();

    let class_map = load_class_map(&csv_dir)?;

    let mut class_entries: Vec<(String, String, usize)> = Vec::new();
    for (idx, class_name) in config.classes.iter().enumerate() {
        let code = class_map
            .get(class_name)
            .ok_or_else(|| format!("Unknown class: '{}'", class_name))?
            .clone();
        class_entries.push((class_name.clone(), code, idx));
    }

    let folders = config.split.folders();
    let ann_files = config.split.annotation_files();

    let multi_folder_name = if config.multiclass {
        Some(config.classes.join("_"))
    } else {
        None
    };

    for (split_folder, ann_file) in folders.iter().zip(ann_files.iter()) {
        let ann_path = csv_dir.join(ann_file);

        if !ann_path.exists() {
            return Err(format!(
                "Annotation file not found: {:?}. Use 'Fetch Metadata' first.",
                ann_path
            ));
        }

        for (class_name, class_code, class_idx) in &class_entries {
            let folder_name = multi_folder_name
                .as_deref()
                .unwrap_or(class_name.as_str());

            let img_dir = dataset_dir.join(split_folder).join(folder_name);
            let label_dir = img_dir.join("Label");
            tokio::fs::create_dir_all(&img_dir)
                .await
                .map_err(|e| e.to_string())?;

            let all_rows = load_bbox_rows(&ann_path, class_code, &config.filters)?;

            let mut image_ids: HashSet<String> = all_rows
                .iter()
                .map(|r| r.image_id.clone())
                .collect();

            if let Some(limit) = config.limit {
                image_ids = image_ids.into_iter().take(limit).collect();
            }

            let total = image_ids.len();
            let downloaded_count = Arc::new(AtomicUsize::new(0));

            let _ = tx.send(ProgressEvent {
                job_id: job_id.clone(),
                class_name: class_name.clone(),
                split: split_folder.to_string(),
                downloaded: 0,
                total,
                finished: false,
                error: None,
            });

            let image_ids_vec: Vec<String> = image_ids.into_iter().collect();
            let threads = config.threads;

            stream::iter(image_ids_vec.iter())
                .for_each_concurrent(threads, |image_id| {
                    let client = client.clone();
                    let img_dir = img_dir.clone();
                    let tx = tx.clone();
                    let job_id = job_id.clone();
                    let class_name = class_name.clone();
                    let split_str = split_folder.to_string();
                    let count = Arc::clone(&downloaded_count);

                    async move {
                        let result =
                            download_image(&client, split_folder, image_id, &img_dir).await;
                        let done = count.fetch_add(1, Ordering::Relaxed) + 1;
                        let error = result.err();
                        let _ = tx.send(ProgressEvent {
                            job_id,
                            class_name,
                            split: split_str,
                            downloaded: done,
                            total,
                            finished: false,
                            error: error.map(|e| e.to_string()),
                        });
                    }
                })
                .await;

            if !config.no_labels {
                let mut rows_by_image: HashMap<String, Vec<BBoxRow>> = HashMap::new();
                for row in all_rows {
                    rows_by_image
                        .entry(row.image_id.clone())
                        .or_default()
                        .push(row);
                }

                let downloaded_ids: Vec<String> = std::fs::read_dir(&img_dir)
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .filter(|e| {
                                e.path().extension().map(|x| x == "jpg").unwrap_or(false)
                            })
                            .filter_map(|e| {
                                e.path()
                                    .file_stem()
                                    .map(|s| s.to_string_lossy().into_owned())
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                for image_id in &downloaded_ids {
                    if let Some(boxes) = rows_by_image.get(image_id) {
                        write_yolo_label(&label_dir, image_id, *class_idx, boxes)?;
                    }
                }
            }

            let _ = tx.send(ProgressEvent {
                job_id: job_id.clone(),
                class_name: class_name.clone(),
                split: split_folder.to_string(),
                downloaded: total,
                total,
                finished: true,
                error: None,
            });
        }
    }

    Ok(())
}
