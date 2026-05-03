//! Open Images Workbench — GPUI-rs native desktop application.
//!
//! This binary replaces the SvelteKit WebView frontend with a pure-Rust
//! GPU-accelerated UI built on GPUI (Zed's UI framework).
//!
//! Architecture
//! ────────────
//! • `AppState`   — shared entity; all screens read / mutate this.
//! • `RootView`   — top-level GPUI view; routes to the correct screen.
//! • `HomeView`   — landing page with navigation cards.
//! • `DownloadView` — dataset download configuration form.
//! • `JobsView`   — live progress monitor for running downloads.
//! • `ViewerView` — image browser with YOLO bounding-box overlays.
//!
//! Business logic (csv_meta, download, index) lives in `logic/` and has
//! no dependency on Tauri or any web runtime.

#![allow(unused_imports)] // iterative development

mod logic;
mod state;
mod views;

use std::path::PathBuf;

use gpui::{
    App, Application, Bounds, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};

use logic::csv_meta;
use state::{AppState, Screen};
use views::{
    download::DownloadView,
    home::HomeView,
    jobs::JobsView,
    viewer::ViewerView,
    theme::*,
};

// ---------------------------------------------------------------------------
// FileAssetSource — lets img(PathBuf) load images from the local file system
// ---------------------------------------------------------------------------

struct FileAssetSource;

impl gpui::AssetSource for FileAssetSource {
    fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let bytes = std::fs::read(path)?;
        Ok(Some(bytes.into()))
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|e| {
                Some(SharedString::from(
                    e.ok()?.path().to_string_lossy().into_owned(),
                ))
            })
            .collect())
    }
}

// ---------------------------------------------------------------------------
// RootView — delegates rendering to the active screen's view entity
// ---------------------------------------------------------------------------

struct RootView {
    app_state: Entity<AppState>,
    home_view: Entity<HomeView>,
    download_view: Entity<DownloadView>,
    jobs_view: Entity<JobsView>,
    viewer_view: Entity<ViewerView>,
}

impl RootView {
    fn new(app_state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let home_view = cx.new(|_| HomeView::new(app_state.clone()));
        let download_view = cx.new(|_| DownloadView::new(app_state.clone()));
        let jobs_view = cx.new(|_| JobsView::new(app_state.clone()));
        let viewer_view = cx.new(|_| ViewerView::new(app_state.clone()));

        Self {
            app_state,
            home_view,
            download_view,
            jobs_view,
            viewer_view,
        }
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let screen = self.app_state.read(cx).screen.clone();

        let content: gpui::AnyElement = match screen {
            Screen::Home => self.home_view.clone().into_any_element(),
            Screen::Download => self.download_view.clone().into_any_element(),
            Screen::Jobs => self.jobs_view.clone().into_any_element(),
            Screen::Viewer => self.viewer_view.clone().into_any_element(),
        };

        div()
            .size_full()
            .bg(rgb(BG_APP))
            .child(content)
    }
}

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

fn main() {
    // Determine the application data directory
    let app_data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("oidw");

    Application::new()
        .with_assets(FileAssetSource)
        .run(move |cx: &mut App| {
            let app_state = cx.new(|_| AppState::new(app_data_dir.clone()));

            // Kick off background class loading
            {
                let csv_dir = app_data_dir.join("csv_folder");
                let app_state_clone = app_state.clone();
                app_state.update(cx, |s, cx| {
                    s.classes_loading = true;
                    cx.notify();
                });
                cx.spawn({
                    let app_state_clone = app_state_clone.clone();
                    async move |mut cx| {
                        let result = csv_meta::ensure_classes(&csv_dir).await;
                        cx.update(|cx| {
                            app_state_clone.update(cx, |s, cx| {
                                s.classes_loading = false;
                                match result {
                                    Ok(classes) => s.all_classes = classes,
                                    Err(e) => s.classes_error = e,
                                }
                                cx.notify();
                            });
                        }).ok();
                    }
                }).detach();
            }

            let bounds = Bounds::centered(
                None,
                size(px(1200.0), px(800.0)),
                cx,
            );
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("Open Images Workbench".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                move |window, cx| {
                    let as2 = app_state.clone();
                    cx.new(|cx| RootView::new(as2, window, cx))
                },
            )
            .expect("Failed to open window");

            cx.activate(true);
        });
}
