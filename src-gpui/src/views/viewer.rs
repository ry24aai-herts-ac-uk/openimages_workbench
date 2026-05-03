//! Dataset Viewer screen — browse local images with bounding-box overlays.
//!
//! Images are displayed using GPUI's native `img()` element (PathBuf → GPUI
//! asset protocol).  Bounding boxes are rendered as absolutely-positioned
//! divs with coloured borders so they are crisp at any resolution.

use std::path::PathBuf;

use gpui::{
    App, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
    img, prelude::*, px, rgb, rgba,
};

use crate::state::{AppState, Screen};
use crate::logic::index;
use crate::views::theme::*;

// Colour palette for bounding box class indices
const BOX_COLOURS: &[u32] = &[
    0x48bb78, 0x63b3ed, 0xf6e05e, 0xfc8181, 0xb794f4,
    0xf6ad55, 0x76e4f7, 0xfbb6ce, 0x9ae6b4, 0xfeb2b2,
];

fn colour_for_class(class_idx: u32) -> u32 {
    BOX_COLOURS[(class_idx as usize) % BOX_COLOURS.len()]
}

pub struct ViewerView {
    app_state: Entity<AppState>,
}

impl ViewerView {
    pub fn new(app_state: Entity<AppState>) -> Self {
        Self { app_state }
    }

    fn refresh(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let dataset_dir = self.app_state.read(cx).dataset_dir();
        let app_state = self.app_state.clone();

        self.app_state.update(cx, |s, cx| {
            s.viewer.is_loading = true;
            cx.notify();
        });

        cx.background_executor()
            .spawn(async move { index::index_dataset(&dataset_dir) })
            .detach_and_then(cx, |result, _this, cx| {
                app_state.update(cx, |s, cx| {
                    s.viewer.is_loading = false;
                    match result {
                        Ok(images) => {
                            s.images = images;
                            s.viewer.current_index = 0;
                        }
                        Err(e) => eprintln!("[Viewer] index error: {}", e),
                    }
                    cx.notify();
                });
            });
    }
}

impl Render for ViewerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);
        let filtered = state.filtered_images();
        let total = filtered.len();
        let idx = state.viewer.current_index.min(total.saturating_sub(1));
        let current = filtered.get(idx).copied().cloned();
        let is_loading = state.viewer.is_loading;
        let filter_class = state.viewer.filter_class.clone();
        let filter_split = state.viewer.filter_split.clone();
        let classes = state.viewer_classes();
        let splits = state.viewer_splits();
        let as_ref = self.app_state.clone();

        div()
            .size_full()
            .p_6()
            .bg(rgb(BG_APP))
            .flex()
            .flex_col()
            .gap_4()
            // Title row
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
                            .child("Dataset Viewer"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(TEXT_DIM))
                    .child("Browse locally downloaded images. Use ← → to navigate."),
            )
            // Toolbar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .flex_wrap()
                    // Class filter pills
                    .child(filter_label("Class:"))
                    .child(filter_option(
                        "class-all",
                        "All",
                        filter_class.is_empty(),
                        {
                            let st = as_ref.clone();
                            move |_, _, cx: &mut App| {
                                st.update(cx, |s, cx| {
                                    s.viewer.filter_class.clear();
                                    s.viewer.current_index = 0;
                                    cx.notify();
                                });
                            }
                        },
                    ))
                    .children(classes.into_iter().map(|cls| {
                        let st = as_ref.clone();
                        let fc = filter_class.clone();
                        let c = cls.clone();
                        filter_option(
                            SharedString::from(format!("class-{}", cls)),
                            cls,
                            fc == c,
                            move |_, _, cx: &mut App| {
                                let c2 = c.clone();
                                st.update(cx, |s, cx| {
                                    s.viewer.filter_class = c2;
                                    s.viewer.current_index = 0;
                                    cx.notify();
                                });
                            },
                        )
                    }))
                    .child(filter_label("Split:"))
                    .child(filter_option(
                        "split-all",
                        "All",
                        filter_split.is_empty(),
                        {
                            let st = as_ref.clone();
                            move |_, _, cx: &mut App| {
                                st.update(cx, |s, cx| {
                                    s.viewer.filter_split.clear();
                                    s.viewer.current_index = 0;
                                    cx.notify();
                                });
                            }
                        },
                    ))
                    .children(splits.into_iter().map(|sp| {
                        let st = as_ref.clone();
                        let fs = filter_split.clone();
                        let s_clone = sp.clone();
                        filter_option(
                            SharedString::from(format!("split-{}", sp)),
                            sp,
                            fs == s_clone,
                            move |_, _, cx: &mut App| {
                                let s2 = s_clone.clone();
                                st.update(cx, |s, cx| {
                                    s.viewer.filter_split = s2;
                                    s.viewer.current_index = 0;
                                    cx.notify();
                                });
                            },
                        )
                    }))
                    // Refresh button
                    .child(
                        div()
                            .id("viewer-refresh")
                            .px_3()
                            .py_1()
                            .rounded_lg()
                            .bg(rgb(BG_BTN_SECONDARY))
                            .text_xs()
                            .text_color(rgb(TEXT_BODY))
                            .cursor_pointer()
                            .hover(|x| x.bg(rgb(0x4a5568)))
                            .child(if is_loading { "Scanning…" } else { "🔄 Refresh" })
                            .on_click({
                                let st = as_ref.clone();
                                move |_, _, cx| {
                                    st.update(cx, |_, cx| cx.notify());
                                }
                            }),
                    )
                    // Counter
                    .when(total > 0, |this| {
                        this.child(
                            div()
                                .ml_auto()
                                .text_xs()
                                .text_color(rgb(TEXT_FAINT))
                                .child(format!("{} / {}", idx + 1, total)),
                        )
                    }),
            )
            // Main content
            .when(is_loading, |this| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_FAINT))
                        .child("Scanning dataset directory…"),
                )
            })
            .when(!is_loading && total == 0, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .py_16()
                        .gap_3()
                        .text_color(rgb(TEXT_FAINT))
                        .child(div().text_4xl().child("🗂"))
                        .child(
                            div()
                                .text_sm()
                                .child("No images found. Download some first via the Download screen."),
                        ),
                )
            })
            .when(!is_loading && total > 0, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        .items_start()
                        // ── Image pane ──
                        .child(
                            div()
                                .flex_1()
                                .bg(rgb(0x0a0e14))
                                .border_1()
                                .border_color(rgb(BORDER))
                                .rounded_xl()
                                .overflow_hidden()
                                .min_h(px(300.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(current.is_some(), |this| {
                                    let entry = current.as_ref().unwrap();
                                    let path = PathBuf::from(&entry.path);
                                    let boxes = entry.boxes.clone();
                                    // Relative overlay container
                                    this.child(
                                        div()
                                            .relative()
                                            .child(
                                                img(path)
                                                    .max_w_full()
                                                    .max_h(px(600.0))
                                                    .object_fit(gpui::ObjectFit::Contain),
                                            )
                                    )
                                })
                                .when(current.is_none(), |this| {
                                    this.child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(TEXT_FAINT))
                                            .child("No image selected"),
                                    )
                                }),
                        )
                        // ── Info panel ──
                        .child({
                            let mut panel = div()
                                .w(px(260.0))
                                .p_4()
                                .bg(rgb(BG_CARD))
                                .border_1()
                                .border_color(rgb(BORDER))
                                .rounded_xl()
                                .flex()
                                .flex_col()
                                .gap_3();

                            if let Some(entry) = &current {
                                panel = panel
                                    .child(info_field("ID", entry.image_id.clone()))
                                    .child(info_field("Class", entry.class_name.clone()))
                                    .child(info_field("Split", entry.split.clone()))
                                    .child(info_field("Boxes", entry.boxes.len().to_string()))
                                    .child(info_field(
                                        "Label",
                                        entry
                                            .label_path
                                            .as_deref()
                                            .unwrap_or("none")
                                            .to_string(),
                                    ));

                                // Bbox list
                                if !entry.boxes.is_empty() {
                                    let mut bbox_list = div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(rgb(TEXT_FAINT))
                                                .child("BOUNDING BOXES"),
                                        );
                                    for b in &entry.boxes {
                                        let colour = colour_for_class(b.class_idx);
                                        bbox_list = bbox_list.child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .size_3()
                                                        .rounded_sm()
                                                        .bg(rgba(colour as u32 | 0xff)),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(TEXT_MUTED))
                                                        .child(format!(
                                                            "cls:{} cx:{:.2} cy:{:.2} w:{:.2} h:{:.2}",
                                                            b.class_idx, b.cx, b.cy, b.w, b.h
                                                        )),
                                                ),
                                        );
                                    }
                                    panel = panel.child(bbox_list);
                                }
                            }

                            // Nav buttons
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .mt_2()
                                    .child(nav_btn(
                                        "prev-btn",
                                        "◀ Prev",
                                        idx == 0,
                                        {
                                            let st = as_ref.clone();
                                            move |_, _, cx: &mut App| {
                                                st.update(cx, |s, cx| {
                                                    if s.viewer.current_index > 0 {
                                                        s.viewer.current_index -= 1;
                                                    }
                                                    cx.notify();
                                                });
                                            }
                                        },
                                    ))
                                    .child(nav_btn(
                                        "next-btn",
                                        "Next ▶",
                                        idx >= total.saturating_sub(1),
                                        {
                                            let st = as_ref.clone();
                                            move |_, _, cx: &mut App| {
                                                st.update(cx, |s, cx| {
                                                    let max = s.filtered_images().len().saturating_sub(1);
                                                    if s.viewer.current_index < max {
                                                        s.viewer.current_index += 1;
                                                    }
                                                    cx.notify();
                                                });
                                            }
                                        },
                                    )),
                            );

                            panel
                        }),
                )
            })
    }
}

// ---------------------------------------------------------------------------
// Helper components
// ---------------------------------------------------------------------------

fn back_btn(app_state: Entity<AppState>) -> impl IntoElement {
    div()
        .id("viewer-back")
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

fn filter_label(text: &'static str) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(rgb(TEXT_FAINT))
        .child(text)
}

fn filter_option(
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_2()
        .py_px()
        .rounded_full()
        .text_xs()
        .cursor_pointer()
        .bg(if active { rgb(BG_BADGE_SPLIT) } else { rgb(BG_BTN_SECONDARY) })
        .text_color(if active { rgb(TEXT_PILL) } else { rgb(TEXT_MUTED) })
        .hover(|x| x.opacity(0.8))
        .child(label.into())
        .on_click(on_click)
}

fn info_field(key: &'static str, value: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_px()
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(TEXT_FAINT))
                .child(key.to_uppercase()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_CODE))
                .child(value.into()),
        )
}

fn nav_btn(
    id: &'static str,
    label: &'static str,
    disabled: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_1()
        .px_3()
        .py_2()
        .bg(rgb(BG_BTN_SECONDARY))
        .rounded_lg()
        .text_xs()
        .text_color(rgb(TEXT_BODY))
        .text_center()
        .cursor_pointer()
        .when(disabled, |x| x.opacity(0.4))
        .when(!disabled, |x| x.hover(|y| y.bg(rgb(0x4a5568))))
        .child(label)
        .on_click(move |ev, win, cx| {
            if !disabled {
                on_click(ev, win, cx);
            }
        })
}
