use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use cloud_terrastodon_command::app_work::ActiveAppWork;
use eframe::egui::Align2;
use eframe::egui::Area;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::Order;
use eframe::egui::ScrollArea;
use eframe::egui::Shadow;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use egui_toast::Toasts;
use std::mem;
use std::time::Instant;
use teamy_tracing::gui::escape_display;

const TOAST_MARGIN: f32 = 10.0;
const WORK_TOAST_WIDTH: f32 = 360.0;

impl MyApp {
    pub(crate) fn show_toasts(&mut self, ui: &mut Ui) {
        self.show_log_toasts(ui);
        let work = self.work_tracker.active_work().unwrap_or_default();
        show_toast_stack(ui, &mut self.toasts, &work, self.egui_config.demo_mode);
    }
}

fn show_toast_stack(ui: &mut Ui, logs: &mut Toasts, work: &[ActiveAppWork], demo_mode: bool) {
    let reserved_height = show_background_work_toasts(ui, work, demo_mode)
        .map_or(0.0, |height| height + TOAST_MARGIN);
    // Leave the timed log stack above the work toasts when they are visible.
    *logs = mem::take(logs).anchor(
        Align2::RIGHT_BOTTOM,
        (-TOAST_MARGIN, -TOAST_MARGIN - reserved_height),
    );
    logs.show(ui);
}

/// Draw one notice per live job, with the oldest job at the bottom of the stack.
fn show_background_work_toasts(
    ui: &mut Ui,
    work: &[ActiveAppWork],
    demo_mode: bool,
) -> Option<f32> {
    let viewport = ui.ctx().content_rect();
    // An area's available width includes its remembered size from the last frame.
    // Size from the viewport so truncation cannot make the next frame narrower.
    let width = (viewport.width() - 2.0 * TOAST_MARGIN).min(WORK_TOAST_WIDTH);
    if work.is_empty() || width <= 2.0 * TOAST_MARGIN {
        return None;
    }
    let max_height = viewport.height() * 0.5;
    let now = Instant::now();
    let area = Area::new(Id::new("portal-background-work-toasts"))
        .anchor(Align2::RIGHT_BOTTOM, Vec2::splat(-TOAST_MARGIN))
        .default_width(width)
        .order(Order::Foreground)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);
            ui.spacing_mut().item_spacing.y = TOAST_MARGIN;
            ScrollArea::vertical()
                .max_height(max_height)
                .min_scrolled_height(0.0)
                .auto_shrink([false, true])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for job in work.iter().rev() {
                        ui.push_id(job.id, |ui| {
                            show_background_work_toast(ui, job, now, demo_mode)
                        });
                    }
                });
        });
    Some(area.response.rect.height())
}

fn show_background_work_toast(ui: &mut Ui, job: &ActiveAppWork, now: Instant, demo_mode: bool) {
    let elapsed = now.duration_since(job.started_at).as_secs_f64();
    let description = if demo_mode {
        "Background work".to_owned()
    } else {
        escape_display(&job.description)
    };
    let text = format!("{elapsed:.1} s · {description}");
    let content_width = (ui.available_width() - 2.0 * TOAST_MARGIN).max(0.0);
    Frame::window(ui.style())
        .inner_margin(TOAST_MARGIN)
        .shadow(Shadow::NONE)
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.set_width(content_width);
            ui.horizontal(|ui| {
                ui.add(
                    Image::new(page_icon(PortalPage::BackgroundWork))
                        .fit_to_exact_size(Vec2::splat(16.0)),
                );
                ui.add(Label::new(&text).truncate()).on_hover_text(text);
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use eframe::egui::Direction;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use egui_toast::Toast;
    use egui_toast::ToastOptions;
    use std::time::Duration;

    #[tokio::test]
    async fn running_work_toasts_recover_their_width_and_follow_each_job() -> eyre::Result<()> {
        let first = tokio::spawn(async {});
        let second = tokio::spawn(async {});
        let now = Instant::now();
        let work = [
            ActiveAppWork {
                id: first.id(),
                description: "Inspect resource inventory".to_owned(),
                started_at: now - Duration::from_secs(12),
            },
            ActiveAppWork {
                id: second.id(),
                description: "Load management group hierarchy".to_owned(),
                started_at: now - Duration::from_secs(3),
            },
        ];
        first.await?;
        second.await?;

        let context = Context::default();
        let mut logs = Toasts::new().direction(Direction::BottomUp);
        logs.add(
            Toast::new()
                .text("Log notice")
                .options(ToastOptions::default().duration(Duration::from_secs(5))),
        );
        let mut draw = |work: &[ActiveAppWork], demo_mode, width| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 550.0))),
                ..Default::default()
            };
            let mut output = context.run_ui(input, |ui| {
                show_toast_stack(ui, &mut logs, work, demo_mode);
            });
            output.textures_delta.clear();
            output
                .shapes
                .into_iter()
                .filter_map(|shape| match shape.shape {
                    Shape::Text(text) => Some((
                        text.galley.text().to_owned(),
                        text.visual_bounding_rect(),
                        text.galley.elided,
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        draw(&work, false, 168.0);
        assert!(
            draw(&work, false, 168.0)
                .iter()
                .any(|(_, _, elided)| *elided)
        );
        draw(&work, false, 700.0);
        let mut stacked_log = Rect::NOTHING;
        for _ in 0..8 {
            let painted = draw(&work, false, 700.0);
            let (first_label, first_rect, first_elided) = painted
                .iter()
                .find(|(text, _, _)| text.ends_with("Inspect resource inventory"))
                .unwrap();
            let (second_label, second_rect, second_elided) = painted
                .iter()
                .find(|(text, _, _)| text.ends_with("Load management group hierarchy"))
                .unwrap();
            assert!(!first_elided && !second_elided);
            let first_elapsed = first_label.split(" s · ").next().unwrap().parse::<f64>()?;
            let second_elapsed = second_label.split(" s · ").next().unwrap().parse::<f64>()?;
            assert!(first_elapsed >= 12.0 && second_elapsed >= 3.0);
            assert!((first_elapsed - second_elapsed - 9.0).abs() <= 0.2);
            assert!(second_rect.bottom() < first_rect.top());
            let (_, log_label, _) = painted
                .iter()
                .find(|(text, _, _)| text == "Log notice")
                .unwrap();
            assert!(log_label.bottom() < second_rect.top());
            stacked_log = *log_label;
        }

        let demo = draw(&work, true, 700.0);
        assert_eq!(
            demo.iter()
                .filter(|(text, _, _)| text.ends_with("Background work"))
                .count(),
            2
        );
        assert!(
            !demo
                .iter()
                .any(|(text, _, _)| work.iter().any(|job| text.ends_with(&job.description)))
        );

        let remaining = draw(&work[1..], false, 700.0);
        assert!(
            remaining
                .iter()
                .any(|(text, _, _)| text.ends_with(&work[1].description))
        );
        assert!(
            !remaining
                .iter()
                .any(|(text, _, _)| text.ends_with(&work[0].description))
        );

        let finished = draw(&[], false, 700.0);
        assert!(
            !finished
                .iter()
                .any(|(text, _, _)| text.ends_with(&work[1].description))
        );
        let (_, log_at_bottom, _) = finished
            .iter()
            .find(|(text, _, _)| text == "Log notice")
            .unwrap();
        assert!(log_at_bottom.top() > stacked_log.top());
        Ok(())
    }
}
