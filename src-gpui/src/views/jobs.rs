//! Jobs screen — live download progress tracker.

use gpui::{
    App, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
    prelude::*, px, rgb,
};

use crate::state::{AppState, Screen};
use crate::views::theme::*;

pub struct JobsView {
    app_state: Entity<AppState>,
}

impl JobsView {
    pub fn new(app_state: Entity<AppState>) -> Self {
        Self { app_state }
    }
}

impl Render for JobsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);
        // newest first
        let mut jobs = state.jobs.clone();
        jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        let as_ref = self.app_state.clone();

        div()
            .size_full()
            .p_6()
            .bg(rgb(BG_APP))
            .flex()
            .flex_col()
            .gap_4()
            .overflow_y_scroll()
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
                            .child("Download Jobs"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(TEXT_DIM))
                    .child("Live progress for active and completed download jobs."),
            )
            // Job cards
            .when(jobs.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_3()
                        .py_16()
                        .text_color(rgb(TEXT_FAINT))
                        .child(div().text_4xl().child("📭"))
                        .child(
                            div()
                                .text_sm()
                                .child("No jobs yet. Go to Download to start one."),
                        ),
                )
            })
            .when(!jobs.is_empty(), |mut this| {
                for job in &jobs {
                    let pct = job.percent();
                    let elapsed = format_elapsed(job.elapsed_secs());
                    let status_label = if job.finished { "✅ Done" } else { "⏳ Running" };
                    let badge_bg = if job.finished { BG_BADGE_DONE } else { BG_BADGE_RUNNING };
                    let badge_text = if job.finished { TEXT_BADGE_DONE } else { TEXT_BADGE_RUNNING };
                    let card_border = if job.errors.is_empty() {
                        if job.finished { BORDER_DONE } else { BORDER }
                    } else {
                        BORDER_ERROR
                    };
                    let bar_colour = if job.finished { BG_PROGRESS_DONE } else { BG_PROGRESS_FILL };
                    let errors = job.errors.clone();
                    let job_id_txt = job.job_id.clone();
                    let class_name = job.class_name.clone();
                    let split_name = job.split.clone();
                    let downloaded = job.downloaded;
                    let total = job.total;

                    this = this.child(
                        div()
                            .p_4()
                            .bg(rgb(BG_CARD))
                            .border_1()
                            .border_color(rgb(card_border))
                            .rounded_xl()
                            .flex()
                            .flex_col()
                            .gap_2()
                            // Header row
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(TEXT_BODY))
                                            .text_sm()
                                            .child(class_name),
                                    )
                                    .child(badge(split_name, BG_BADGE_SPLIT, TEXT_PILL))
                                    .child(badge(status_label.to_string(), badge_bg, badge_text))
                                    .child(
                                        div()
                                            .ml_auto()
                                            .text_xs()
                                            .text_color(rgb(TEXT_FAINT))
                                            .child(elapsed),
                                    ),
                            )
                            // Progress bar
                            .child(
                                div()
                                    .h(px(6.0))
                                    .w_full()
                                    .bg(rgb(BG_PROGRESS))
                                    .rounded_full()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(gpui::relative(pct as f32 / 100.0))
                                            .bg(rgb(bar_colour))
                                            .rounded_full(),
                                    ),
                            )
                            // Stats row
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .justify_between()
                                    .text_xs()
                                    .text_color(rgb(TEXT_DIM))
                                    .child(format!(
                                        "{} / {} images",
                                        downloaded,
                                        if total == 0 { "?".to_string() } else { total.to_string() }
                                    ))
                                    .child(format!("{}%", pct)),
                            )
                            // Errors
                            .when(!errors.is_empty(), |this| {
                                this.child(
                                    div()
                                        .mt_1()
                                        .text_xs()
                                        .text_color(rgb(TEXT_RED))
                                        .child(format!(
                                            "{} error(s): {}",
                                            errors.len(),
                                            errors.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
                                        )),
                                )
                            })
                            // Job ID footer
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_FAINT))
                                    .child(job_id_txt),
                            ),
                    );
                }
                this
            })
            // Clear button
            .when(!jobs.is_empty(), |this| {
                this.child(
                    div()
                        .id("clear-jobs")
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(rgb(BG_BTN_SECONDARY))
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(TEXT_BODY))
                        .cursor_pointer()
                        .hover(|x| x.bg(rgb(0x4a5568)))
                        .child("Clear All")
                        .on_click({
                            let st = as_ref.clone();
                            move |_, _, cx| {
                                st.update(cx, |s, cx| {
                                    s.jobs.clear();
                                    cx.notify();
                                });
                            }
                        }),
                )
            })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn back_btn(app_state: Entity<AppState>) -> impl IntoElement {
    div()
        .id("jobs-back")
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

fn badge(text: impl Into<SharedString>, bg: u32, text_colour: u32) -> impl IntoElement {
    div()
        .px_2()
        .py_px()
        .rounded_full()
        .bg(rgb(bg))
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(text_colour))
        .child(text.into())
}

fn format_elapsed(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format!("{}m {}s", secs / 60, secs % 60)
    }
}
