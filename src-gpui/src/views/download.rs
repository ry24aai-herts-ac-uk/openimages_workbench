//! Download configuration screen.

use gpui::{
    App, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
    prelude::*, px, rgb,
};

use crate::logic::models::{DownloadConfig, ImageFilters, Split};
use crate::logic::{csv_meta, download};
use crate::state::{AppState, Screen};
use crate::views::theme::*;

pub struct DownloadView {
    app_state: Entity<AppState>,
}

impl DownloadView {
    pub fn new(app_state: Entity<AppState>) -> Self {
        Self { app_state }
    }

    // -----------------------------------------------------------------------
    // Actions
    // -----------------------------------------------------------------------

    fn fetch_meta(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let split = self.app_state.read(cx).download_form.split.clone();
        let csv_dir = self.app_state.read(cx).csv_dir();
        let app_state = self.app_state.clone();

        self.app_state.update(cx, |s, cx| {
            s.download_form.status_msg = "Fetching annotation CSVs…".to_string();
            s.download_form.is_loading = true;
            cx.notify();
        });

        let app_state = self.app_state.clone();
        cx.spawn(async move |_this, mut cx| {
            let result = csv_meta::ensure_annotation_csvs(&csv_dir, &split).await;
            cx.update(|cx| {
                app_state.update(cx, |s, cx| {
                    s.download_form.is_loading = false;
                    s.download_form.status_msg = match result {
                        Ok(()) => "✅ Annotation CSVs ready.".to_string(),
                        Err(e) => format!("❌ {}", e),
                    };
                    cx.notify();
                });
            }).ok();
        }).detach();
    }

    fn start_download(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let state = self.app_state.read(cx);
        let form = &state.download_form;

        if form.selected_classes.is_empty() {
            self.app_state.update(cx, |s, cx| {
                s.download_form.status_msg =
                    "❌ Please select at least one class.".to_string();
                cx.notify();
            });
            return;
        }

        let threads: usize = form.threads_str.parse().unwrap_or(20);
        let limit: Option<usize> = form.limit_str.parse().ok();
        let config = DownloadConfig {
            classes: form.selected_classes.clone(),
            split: form.split.clone(),
            multiclass: form.multiclass,
            limit,
            threads,
            no_labels: form.no_labels,
            filters: ImageFilters {
                is_occluded: form.is_occluded,
                is_truncated: form.is_truncated,
                is_group_of: form.is_group_of,
                is_depiction: form.is_depiction,
                is_inside: form.is_inside,
            },
            dataset_dir: Some(state.dataset_dir().to_string_lossy().into_owned()),
        };
        let csv_dir = state.csv_dir();
        let dataset_dir = state.dataset_dir();

        let job_id = download::generate_job_id();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Register one job row per class
        for class_name in &config.classes {
            self.app_state.update(cx, |s, cx| {
                s.add_job(&job_id, class_name, config.split.as_str());
                cx.notify();
            });
        }

        // Navigate to jobs screen
        self.app_state.update(cx, |s, cx| {
            s.screen = Screen::Jobs;
            cx.notify();
        });

        let app_state = self.app_state.clone();
        let job_id_clone = job_id.clone();

        // Spawn background download task
        cx.background_executor()
            .spawn(download::run_download(
                config,
                csv_dir,
                dataset_dir,
                tx,
                job_id,
            ))
            .detach();

        // Spawn a task to drain the progress channel and update UI state
        let app_state2 = app_state.clone();
        cx.spawn(async move |_this, mut cx| {
            while let Some(event) = rx.recv().await {
                cx.update(|cx| {
                    app_state2.update(cx, |s, cx| {
                        s.apply_progress(event);
                        cx.notify();
                    });
                }).ok();
            }
        })
        .detach();
    }

    fn toggle_class(&mut self, name: String, cx: &mut Context<Self>) {
        self.app_state.update(cx, |s, cx| {
            let pos = s.download_form.selected_classes.iter().position(|c| c == &name);
            if let Some(i) = pos {
                s.download_form.selected_classes.remove(i);
            } else {
                s.download_form.selected_classes.push(name);
            }
            s.download_form.class_search.clear();
            cx.notify();
        });
    }

    fn remove_class(&mut self, name: String, cx: &mut Context<Self>) {
        self.app_state.update(cx, |s, cx| {
            s.download_form.selected_classes.retain(|c| c != &name);
            cx.notify();
        });
    }
}

impl Render for DownloadView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);
        let form = &state.download_form;

        // Build filtered class suggestions
        let search = form.class_search.to_lowercase();
        let suggestions: Vec<String> = if search.len() >= 2 {
            state
                .all_classes
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&search))
                .take(50)
                .map(|c| c.name.clone())
                .collect()
        } else {
            vec![]
        };

        let selected = form.selected_classes.clone();
        let status = form.status_msg.clone();
        let is_loading = form.is_loading;
        let split_str = form.split.to_string();
        let threads_str = form.threads_str.clone();
        let limit_str = form.limit_str.clone();
        let multiclass = form.multiclass;
        let no_labels = form.no_labels;

        let as_ref = self.app_state.clone();

        div()
            .id("download-scroll")
            .size_full()
            .p_6()
            .bg(rgb(BG_APP))
            .flex()
            .flex_col()
            .gap_4()
            .overflow_y_scroll()
            // ── Title ──
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(back_btn(as_ref.clone()))
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::EXTRA_BOLD)
                            .text_color(rgb(TEXT_TITLE))
                            .child("Download Dataset"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(TEXT_DIM))
                    .child("Configure a download job. Classes fetched from Open Images."),
            )
            // ── Status banner ──
            .when(!status.is_empty(), |this| {
                let colour = if status.starts_with('❌') { BG_ERROR } else { BG_SUCCESS };
                let text_colour = if status.starts_with('❌') { TEXT_ERROR } else { TEXT_SUCCESS };
                this.child(
                    div()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(rgb(colour))
                        .border_1()
                        .border_color(rgb(if status.starts_with('❌') { BORDER_ERROR } else { BORDER_DONE }))
                        .text_sm()
                        .text_color(rgb(text_colour))
                        .child(status.clone()),
                )
            })
            // ── Two-column form ──
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_start()
                    // Left: class picker
                    .child(
                        div()
                            .flex_1()
                            .p_5()
                            .bg(rgb(BG_CARD))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .rounded_xl()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(TEXT_TITLE))
                                    .child("1 · Select Classes"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_FAINT))
                                    .child("(Type to search — min 2 characters)"),
                            )
                            // Suggestions list
                            .when(!suggestions.is_empty(), |this| {
                                let mut list = div()
                                    .id("class-suggestions")
                                    .bg(rgb(BG_INPUT))
                                    .border_1()
                                    .border_color(rgb(BORDER))
                                    .rounded_lg()
                                    .max_h(px(200.0))
                                    .overflow_y_scroll()
                                    .flex()
                                    .flex_col();
                                for name in &suggestions {
                                    let n = name.clone();
                                    let is_sel = selected.contains(name);
                                    let st = as_ref.clone();
                                    list = list.child(
                                        div()
                                            .id(SharedString::from(format!("sug-{}", n)))
                                            .px_3()
                                            .py_1()
                                            .text_xs()
                                            .cursor_pointer()
                                            .text_color(if is_sel { rgb(TEXT_GREEN) } else { rgb(TEXT_MUTED) })
                                            .hover(|x| x.bg(rgb(0x1a2535)).text_color(rgb(TEXT_BODY)))
                                            .child(n.clone())
                                            .on_click(move |_, _, cx| {
                                                st.update(cx, |s, cx| {
                                                    let pos = s.download_form.selected_classes
                                                        .iter().position(|c| c == &n);
                                                    if let Some(i) = pos {
                                                        s.download_form.selected_classes.remove(i);
                                                    } else {
                                                        s.download_form.selected_classes.push(n.clone());
                                                    }
                                                    s.download_form.class_search.clear();
                                                    cx.notify();
                                                });
                                            }),
                                    );
                                }
                                this.child(list)
                            })
                            // Selected pills
                            .when(!selected.is_empty(), |this| {
                                let mut pills = div().flex().flex_row().flex_wrap().gap_2();
                                for cls in &selected {
                                    let c = cls.clone();
                                    let st = as_ref.clone();
                                    pills = pills.child(
                                        div()
                                            .id(SharedString::from(format!("pill-{}", c)))
                                            .px_3()
                                            .py_1()
                                            .rounded_full()
                                            .bg(rgb(BG_PILL))
                                            .text_xs()
                                            .text_color(rgb(TEXT_PILL))
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .child(c.clone())
                                            .child(
                                                div()
                                                    .id(SharedString::from(format!("rm-{}", c)))
                                                    .cursor_pointer()
                                                    .text_color(rgb(TEXT_BLUE))
                                                    .child("×")
                                                    .on_click(move |_, _, cx| {
                                                        st.update(cx, |s, cx| {
                                                            s.download_form.selected_classes.retain(|x| x != &c);
                                                            cx.notify();
                                                        });
                                                    }),
                                            ),
                                    );
                                }
                                this.child(pills)
                            })
                            .when(state.classes_loading, |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(TEXT_FAINT))
                                        .child("Loading classes…"),
                                )
                            }),
                    )
                    // Right: options
                    .child(
                        div()
                            .flex_1()
                            .p_5()
                            .bg(rgb(BG_CARD))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .rounded_xl()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(TEXT_TITLE))
                                    .child("2 · Dataset Options"),
                            )
                            .child(labeled_row(
                                "Split",
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(split_str.clone()),
                            ))
                            .child(labeled_row(
                                "Threads",
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(format!("{} (edit in source)", threads_str)),
                            ))
                            .child(labeled_row(
                                "Image limit",
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(if limit_str.is_empty() {
                                        "no limit".to_string()
                                    } else {
                                        limit_str.clone()
                                    }),
                            ))
                            .child(checkbox_row(
                                "Merge into single folder (multiclass)",
                                multiclass,
                                {
                                    let st = as_ref.clone();
                                    move |_, _, cx: &mut App| {
                                        st.update(cx, |s, cx| {
                                            s.download_form.multiclass = !s.download_form.multiclass;
                                            cx.notify();
                                        });
                                    }
                                },
                            ))
                            .child(checkbox_row(
                                "Skip YOLO label file generation",
                                no_labels,
                                {
                                    let st = as_ref.clone();
                                    move |_, _, cx: &mut App| {
                                        st.update(cx, |s, cx| {
                                            s.download_form.no_labels = !s.download_form.no_labels;
                                            cx.notify();
                                        });
                                    }
                                },
                            )),
                    ),
            )
            // ── Action buttons ──
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .child(action_btn(
                        "btn-fetch",
                        if is_loading { "Fetching…" } else { "📥  Fetch Metadata CSVs" },
                        BG_BTN_SECONDARY,
                        is_loading,
                        {
                            let st = as_ref.clone();
                            move |_, _, cx: &mut App| {
                                st.update(cx, |_, cx| cx.notify());
                            }
                        },
                    ))
                    .child(action_btn(
                        "btn-start",
                        "🚀  Start Download",
                        BG_BTN_PRIMARY,
                        selected.is_empty() || is_loading,
                        {
                            let st = as_ref.clone();
                            move |_, _, cx: &mut App| {
                                st.update(cx, |_, cx| cx.notify());
                            }
                        },
                    )),
            )
    }
}

// ---------------------------------------------------------------------------
// Small reusable layout helpers
// ---------------------------------------------------------------------------

fn back_btn(app_state: Entity<AppState>) -> impl IntoElement {
    div()
        .id("back-btn")
        .px_3()
        .py_1()
        .bg(rgb(BG_BTN_SECONDARY))
        .rounded_lg()
        .text_xs()
        .text_color(rgb(TEXT_BODY))
        .cursor_pointer()
        .hover(|x| x.bg(rgb(0x4a5568)))
        .child("← Back")
        .on_click(move |_, _, cx| {
            app_state.update(cx, |s, cx| {
                s.screen = Screen::Home;
                cx.notify();
            });
        })
}

fn labeled_row(label: &'static str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .child(label),
        )
        .child(content)
}

fn checkbox_row(
    label: &'static str,
    checked: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(label)
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .on_click(on_click)
        .child(
            div()
                .size_4()
                .rounded_sm()
                .border_1()
                .border_color(rgb(BORDER))
                .bg(if checked { rgb(BG_BTN_PRIMARY) } else { rgb(BG_INPUT) })
                .flex()
                .items_center()
                .justify_center()
                .when(checked, |x| {
                    x.child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_BODY))
                            .child("✓"),
                    )
                }),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .child(label),
        )
}

fn action_btn(
    id: &'static str,
    label: &'static str,
    bg: u32,
    disabled: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_5()
        .py_2()
        .rounded_lg()
        .bg(rgb(bg))
        .text_sm()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(TEXT_BODY))
        .cursor_pointer()
        .when(disabled, |x| x.opacity(0.5))
        .when(!disabled, |x| x.hover(|y| y.opacity(0.85)))
        .child(label)
        .on_click(move |ev, win, cx| {
            if !disabled {
                on_click(ev, win, cx);
            }
        })
}
