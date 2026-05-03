//! Shared application state for the GPUI frontend.
//!
//! A single `AppState` entity is held by the main view.  All screens read
//! from / write to this state.  Any mutation calls `cx.notify()` on the
//! entity so GPUI re-renders affected views.

use std::path::PathBuf;

use crate::logic::models::{ClassEntry, DownloadConfig, DownloadJob, ImageEntry, ProgressEvent, Split};

// ---------------------------------------------------------------------------
// Screen enum – controls which view is rendered
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Home,
    Download,
    Jobs,
    Viewer,
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Screen::Home => write!(f, "Home"),
            Screen::Download => write!(f, "Download"),
            Screen::Jobs => write!(f, "Jobs"),
            Screen::Viewer => write!(f, "Viewer"),
        }
    }
}

// ---------------------------------------------------------------------------
// Download form state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DownloadFormState {
    pub class_search: String,
    pub selected_classes: Vec<String>,
    pub split: Split,
    pub multiclass: bool,
    pub limit_str: String,   // raw string from the text field
    pub threads_str: String, // raw string from the text field
    pub no_labels: bool,
    pub is_occluded: Option<bool>,
    pub is_truncated: Option<bool>,
    pub is_group_of: Option<bool>,
    pub is_depiction: Option<bool>,
    pub is_inside: Option<bool>,
    pub status_msg: String,
    pub is_loading: bool,
}

impl Default for DownloadFormState {
    fn default() -> Self {
        Self {
            class_search: String::new(),
            selected_classes: vec![],
            split: Split::Train,
            multiclass: false,
            limit_str: String::new(),
            threads_str: "20".to_string(),
            no_labels: false,
            is_occluded: None,
            is_truncated: None,
            is_group_of: None,
            is_depiction: None,
            is_inside: None,
            status_msg: String::new(),
            is_loading: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Viewer state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ViewerState {
    pub filter_class: String,
    pub filter_split: String,
    pub current_index: usize,
    pub is_loading: bool,
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

pub struct AppState {
    pub screen: Screen,

    /// Absolute path to the app data directory (e.g. ~/.local/share/oidw)
    pub app_data_dir: PathBuf,

    /// All known Open Images classes (loaded once on startup)
    pub all_classes: Vec<ClassEntry>,
    pub classes_loading: bool,
    pub classes_error: String,

    /// Download form
    pub download_form: DownloadFormState,

    /// Active / completed download jobs
    pub jobs: Vec<DownloadJob>,

    /// Dataset index (for the viewer)
    pub images: Vec<ImageEntry>,
    pub viewer: ViewerState,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            screen: Screen::Home,
            app_data_dir,
            all_classes: vec![],
            classes_loading: false,
            classes_error: String::new(),
            download_form: DownloadFormState::default(),
            jobs: vec![],
            images: vec![],
            viewer: ViewerState::default(),
        }
    }

    // -----------------------------------------------------------------------
    // Convenience accessors
    // -----------------------------------------------------------------------

    pub fn csv_dir(&self) -> PathBuf {
        self.app_data_dir.join("csv_folder")
    }

    pub fn dataset_dir(&self) -> PathBuf {
        self.app_data_dir.join("Dataset")
    }

    // -----------------------------------------------------------------------
    // Jobs management
    // -----------------------------------------------------------------------

    pub fn add_job(&mut self, job_id: &str, class_name: &str, split: &str) {
        self.jobs.push(DownloadJob::new(
            job_id.to_string(),
            class_name.to_string(),
            split.to_string(),
        ));
    }

    pub fn apply_progress(&mut self, event: ProgressEvent) {
        let key = (&event.job_id, &event.class_name, &event.split);
        if let Some(job) = self
            .jobs
            .iter_mut()
            .find(|j| (&j.job_id, &j.class_name, &j.split) == key)
        {
            job.apply(&event);
        } else {
            let mut job = DownloadJob::new(
                event.job_id.clone(),
                event.class_name.clone(),
                event.split.clone(),
            );
            job.apply(&event);
            self.jobs.push(job);
        }
    }

    // -----------------------------------------------------------------------
    // Viewer derived helpers
    // -----------------------------------------------------------------------

    pub fn filtered_images(&self) -> Vec<&ImageEntry> {
        self.images
            .iter()
            .filter(|e| {
                (self.viewer.filter_class.is_empty()
                    || e.class_name == self.viewer.filter_class)
                    && (self.viewer.filter_split.is_empty()
                        || e.split == self.viewer.filter_split)
            })
            .collect()
    }

    pub fn current_image(&self) -> Option<&ImageEntry> {
        let filtered = self.filtered_images();
        filtered.get(self.viewer.current_index).copied()
    }

    pub fn viewer_classes(&self) -> Vec<String> {
        let mut classes: Vec<String> = self
            .images
            .iter()
            .map(|e| e.class_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        classes.sort();
        classes
    }

    pub fn viewer_splits(&self) -> Vec<String> {
        let mut splits: Vec<String> = self
            .images
            .iter()
            .map(|e| e.split.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        splits.sort();
        splits
    }
}
