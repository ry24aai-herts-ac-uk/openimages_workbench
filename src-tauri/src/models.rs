use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Dataset split
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Split {
    Train,
    Validation,
    Test,
    All,
}

impl Split {
    pub fn folders(&self) -> Vec<&'static str> {
        match self {
            Split::Train => vec!["train"],
            Split::Validation => vec!["validation"],
            Split::Test => vec!["test"],
            Split::All => vec!["train", "validation", "test"],
        }
    }

    pub fn annotation_files(&self) -> Vec<&'static str> {
        match self {
            Split::Train => vec!["train-annotations-bbox.csv"],
            Split::Validation => vec!["validation-annotations-bbox.csv"],
            Split::Test => vec!["test-annotations-bbox.csv"],
            Split::All => vec![
                "train-annotations-bbox.csv",
                "validation-annotations-bbox.csv",
                "test-annotations-bbox.csv",
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Download configuration (mirrors Python argparse)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    /// Classes to download (name strings, e.g. ["Cat", "Dog"])
    pub classes: Vec<String>,
    /// Dataset split
    pub split: Split,
    /// Download together as multi-class folder (true) or separately (false)
    pub multiclass: bool,
    /// Optional image count limit per class / folder
    pub limit: Option<usize>,
    /// Number of concurrent download threads
    pub threads: usize,
    /// Skip label file generation
    pub no_labels: bool,
    /// Image attribute filters
    pub filters: ImageFilters,
    /// Root output directory (defaults to OID/)
    pub dataset_dir: Option<String>,
    /// Whether to auto-confirm missing file downloads
    pub yes: bool,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            classes: vec![],
            split: Split::Train,
            multiclass: false,
            limit: None,
            threads: 20,
            no_labels: false,
            filters: ImageFilters::default(),
            dataset_dir: None,
            yes: true,
        }
    }
}

/// Mirrors the --image_Is* flags from the Python CLI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageFilters {
    pub is_occluded: Option<bool>,
    pub is_truncated: Option<bool>,
    pub is_group_of: Option<bool>,
    pub is_depiction: Option<bool>,
    pub is_inside: Option<bool>,
}

// ---------------------------------------------------------------------------
// Annotation / bounding-box row (Open Images bbox CSV columns)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBoxRow {
    #[serde(rename = "ImageID")]
    pub image_id: String,
    #[serde(rename = "LabelName")]
    pub label_name: String,
    #[serde(rename = "XMin")]
    pub x_min: f32,
    #[serde(rename = "XMax")]
    pub x_max: f32,
    #[serde(rename = "YMin")]
    pub y_min: f32,
    #[serde(rename = "YMax")]
    pub y_max: f32,
    #[serde(rename = "IsOccluded", default)]
    pub is_occluded: Option<i8>,
    #[serde(rename = "IsTruncated", default)]
    pub is_truncated: Option<i8>,
    #[serde(rename = "IsGroupOf", default)]
    pub is_group_of: Option<i8>,
    #[serde(rename = "IsDepiction", default)]
    pub is_depiction: Option<i8>,
    #[serde(rename = "IsInside", default)]
    pub is_inside: Option<i8>,
}

// ---------------------------------------------------------------------------
// Local dataset index entry (for the viewer)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    /// Absolute path to the .jpg file
    pub path: String,
    /// Image ID (filename without extension)
    pub image_id: String,
    /// Class / folder name
    pub class_name: String,
    /// Dataset split (train / validation / test)
    pub split: String,
    /// YOLO label file path (if it exists)
    pub label_path: Option<String>,
    /// Parsed bounding boxes from the label file
    pub boxes: Vec<YoloBox>,
}

/// One line from a YOLO .txt label file: class_idx cx cy w h
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloBox {
    pub class_idx: u32,
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
}

// ---------------------------------------------------------------------------
// Progress events (emitted to the frontend via Tauri events)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub job_id: String,
    pub class_name: String,
    pub split: String,
    pub downloaded: usize,
    pub total: usize,
    pub finished: bool,
    pub error: Option<String>,
}
