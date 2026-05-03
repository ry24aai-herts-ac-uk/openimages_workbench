//! Home screen — navigation hub with three cards.

use gpui::{
    App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, prelude::*,
    px, rgb, rgba,
};

use crate::state::{AppState, Screen};
use crate::views::theme::*;

pub struct HomeView {
    app_state: Entity<AppState>,
}

impl HomeView {
    pub fn new(app_state: Entity<AppState>) -> Self {
        Self { app_state }
    }

    fn nav_to(screen: Screen, app_state: Entity<AppState>) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) {
        move |_, _, cx| {
            app_state.update(cx, |s, cx| {
                s.screen = screen.clone();
                cx.notify();
            });
        }
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);
        let data_dir = state.app_data_dir.to_string_lossy().into_owned();
        let as_ref = self.app_state.clone();

        div()
            .size_full()
            .p_8()
            .bg(rgb(BG_APP))
            .flex()
            .flex_col()
            .gap_6()
            .child(
                // Title
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::EXTRA_BOLD)
                            .text_color(rgb(TEXT_TITLE))
                            .child("Open Images Workbench"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(TEXT_DIM))
                            .child(
                                "A desktop workbench for downloading, exploring, and exporting \
                                 Open Images datasets — powered by Rust + GPUI.",
                            ),
                    ),
            )
            .child(
                // Cards row
                div()
                    .flex()
                    .flex_row()
                    .gap_5()
                    .child(nav_card(
                        "⬇  Download Dataset",
                        "Select classes, choose a dataset split and annotation filters, \
                         then fetch images and YOLO-format labels from Open Images.",
                        {
                            let st = as_ref.clone();
                            move |_, _, cx: &mut App| {
                                st.update(cx, |s, cx| {
                                    s.screen = Screen::Download;
                                    cx.notify();
                                });
                            }
                        },
                    ))
                    .child(nav_card(
                        "📋  Download Jobs",
                        "Monitor active and completed download jobs with live progress \
                         bars and per-image status updates.",
                        {
                            let st = as_ref.clone();
                            move |_, _, cx: &mut App| {
                                st.update(cx, |s, cx| {
                                    s.screen = Screen::Jobs;
                                    cx.notify();
                                });
                            }
                        },
                    ))
                    .child(nav_card(
                        "🖼  Dataset Viewer",
                        "Browse locally downloaded images with bounding-box overlays \
                         rendered from the YOLO label files.",
                        {
                            let st = as_ref.clone();
                            move |_, _, cx: &mut App| {
                                st.update(cx, |s, cx| {
                                    s.screen = Screen::Viewer;
                                    cx.notify();
                                });
                            }
                        },
                    )),
            )
            .child(
                // Data dir footer
                div()
                    .text_xs()
                    .text_color(rgb(TEXT_FAINT))
                    .child(format!("📁  Data directory: {}", data_dir)),
            )
            .child(
                div()
                    .mt_4()
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .bg(rgb(0x2b4b7e))
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_TITLE))
                    .child("GPUI-rs branch — native Rust UI"),
            )
    }
}

fn nav_card(
    title: &'static str,
    description: &'static str,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(title)
        .w(px(280.0))
        .p_6()
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER))
        .rounded_xl()
        .cursor_pointer()
        .hover(|this| this.border_color(rgb(BORDER_FOCUS)))
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_base()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(TEXT_BODY))
                .child(title),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_DIM))
                .child(description),
        )
        .on_click(on_click)
}
