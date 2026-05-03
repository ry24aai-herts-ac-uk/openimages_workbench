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

    pub fn as_str(&self) -> &'static str {
        match self {
            Split::Train => "train",
            Split::Validation => "validation",
            Split::Test => "test",
            Split::All => "all",
        }
    }
}

impl std::fmt::Display for Split {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Download configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub classes: Vec<String>,
    pub split: Split,
    pub multiclass: bool,
    pub limit: Option<usize>,
    pub threads: usize,
    pub no_labels: bool,
    pub filters: ImageFilters,
    pub dataset_dir: Option<String>,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageFilters {
    pub is_occluded: Option<bool>,
    pub is_truncated: Option<bool>,
    pub is_group_of: Option<bool>,
    pub is_depiction: Option<bool>,
    pub is_inside: Option<bool>,
}

// ---------------------------------------------------------------------------
// Class entry (name + Open Images code)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassEntry {
    pub code: String,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Annotation bounding-box row
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBoxRow {
    pub image_id: String,
    pub label_name: String,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub is_occluded: Option<i8>,
    pub is_truncated: Option<i8>,
    pub is_group_of: Option<i8>,
    pub is_depiction: Option<i8>,
    pub is_inside: Option<i8>,
}

// ---------------------------------------------------------------------------
// Local dataset index entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    pub path: String,
    pub image_id: String,
    pub class_name: String,
    pub split: String,
    pub label_path: Option<String>,
    pub boxes: Vec<YoloBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloBox {
    pub class_idx: u32,
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
}

// ---------------------------------------------------------------------------
// Progress events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub job_id: String,
    pub class_name: String,
    pub split: String,
    pub downloaded: usize,
    pub total: usize,
    pub finished: bool,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Download job (UI state)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub job_id: String,
    pub class_name: String,
    pub split: String,
    pub downloaded: usize,
    pub total: usize,
    pub finished: bool,
    pub errors: Vec<String>,
    pub started_at: std::time::Instant,
}

impl DownloadJob {
    pub fn new(job_id: String, class_name: String, split: String) -> Self {
        Self {
            job_id,
            class_name,
            split,
            downloaded: 0,
            total: 0,
            finished: false,
            errors: vec![],
            started_at: std::time::Instant::now(),
        }
    }

    pub fn apply(&mut self, event: &ProgressEvent) {
        self.downloaded = event.downloaded;
        if event.total > 0 {
            self.total = event.total;
        }
        self.finished = event.finished;
        if let Some(err) = &event.error {
            self.errors.push(err.clone());
        }
    }

    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        ((self.downloaded as f64 / self.total as f64) * 100.0) as u8
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}
