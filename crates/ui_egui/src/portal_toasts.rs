use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_log_toast::PortalLogToast;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_command::app_work::ActiveAppWork;
use eframe::egui::Align2;
use eframe::egui::Area;
use eframe::egui::CursorIcon;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::Order;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Sense;
use eframe::egui::Shadow;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;
use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;
use teamy_tracing::capture::LogLevel;
use teamy_tracing::gui::escape_display;

const TOAST_MARGIN: f32 = 10.0;
const TOAST_WIDTH: f32 = 360.0;

impl MyApp {
    pub(crate) fn show_toasts(&mut self, ui: &mut Ui) {
        let now = Instant::now();
        self.collect_log_toasts(now);
        let work = self.work_tracker.active_work().unwrap_or_default();
        if let Some((open, record)) = show_toast_stack(
            ui,
            &mut self.log_toasts,
            &work,
            self.egui_config.demo_mode,
            now,
        ) {
            let tab = match open {
                PortalTabOpen::Current(target) => {
                    self.navigate_tab(target);
                    self.tabs.active_id()
                }
                PortalTabOpen::Background(target) => self.open_background_tab(target),
            };
            if let Some(record) = record {
                self.pending_log_jumps.insert(tab, record);
            }
            ui.ctx().request_repaint();
        }
        // Capture is independent of the window and does not wake egui.
        ui.ctx().request_repaint_after(Duration::from_millis(250));
    }

    fn collect_log_toasts(&mut self, now: Instant) {
        let records = self.log_capture.records_since(&mut self.log_toast_cursor);
        if self.egui_config.demo_mode {
            self.log_toasts.clear();
            return;
        }
        let start = self.log_toast_cursor - records.len();
        let mut messages = HashSet::new();
        let mut recent = Vec::with_capacity(4);
        for (index, record) in records.iter().enumerate().rev() {
            if let Some(toast) = PortalLogToast::from_record(start + index, record, now)
                && messages.insert(record.message.as_ref())
            {
                recent.push(toast);
                if recent.len() == 4 {
                    break;
                }
            }
        }
        self.log_toasts.extend(recent.into_iter().rev());
    }
}

fn show_toast_stack(
    ui: &mut Ui,
    logs: &mut Vec<PortalLogToast>,
    work: &[ActiveAppWork],
    demo_mode: bool,
    now: Instant,
) -> Option<(PortalTabOpen, Option<usize>)> {
    let viewport = ui.ctx().content_rect();
    let width = (viewport.width() - 2.0 * TOAST_MARGIN).min(TOAST_WIDTH);
    if width <= 2.0 * TOAST_MARGIN {
        return None;
    }
    let (work_open, reserved_height) =
        show_background_work_toasts(ui, work, demo_mode, now, width, viewport.height() * 0.5);
    let mut open = work_open.map(|open| (open, None));
    let mut bottom = TOAST_MARGIN + reserved_height;
    let count = logs.len();
    for toast in logs.iter_mut() {
        let area = Area::new(Id::new(("portal-log-toast", toast.record_index)))
            .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-TOAST_MARGIN, -bottom))
            .default_width(width)
            .order(Order::Foreground)
            .show(ui, |ui| {
                ui.set_width(width);
                let text = if toast.level == LogLevel::Error {
                    RichText::new(&toast.text).color(ui.visuals().error_fg_color)
                } else {
                    RichText::new(&toast.text)
                };
                let response = toast_frame(ui, PortalPage::Logs, text);
                let hovered = response.contains_pointer();
                toast.tick(now, hovered);
                if !hovered && toast.is_visible() {
                    ui.ctx().request_repaint_after(Duration::from_millis(33));
                }
                let bar = response.rect.shrink2(Vec2::new(5.0, 0.0));
                let start = bar.left_bottom() - Vec2::new(0.0, 2.0);
                ui.painter().line_segment(
                    [
                        start,
                        start + Vec2::new(bar.width() * toast.progress(), 0.0),
                    ],
                    Stroke::new(2.0, ui.visuals().weak_text_color()),
                );
                PortalTabOpen::from_response(&response, || PortalTabTarget::Page(PortalPage::Logs))
            });
        bottom += area.response.rect.height() + TOAST_MARGIN;
        if let Some(destination) = area.inner {
            open = Some((destination, Some(toast.record_index)));
            toast.dismiss();
        }
    }
    logs.retain(PortalLogToast::is_visible);
    if logs.len() != count {
        ui.ctx().request_repaint();
    }
    open
}

/// Draw one notice per live job, with the oldest job at the bottom of the stack.
fn show_background_work_toasts(
    ui: &mut Ui,
    work: &[ActiveAppWork],
    demo_mode: bool,
    now: Instant,
    width: f32,
    max_height: f32,
) -> (Option<PortalTabOpen>, f32) {
    if work.is_empty() {
        return (None, 0.0);
    }
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
                    let mut open = None;
                    for job in work.iter().rev() {
                        let response = ui
                            .push_id(job.id, |ui| {
                                show_background_work_toast(ui, job, now, demo_mode)
                            })
                            .inner;
                        if let Some(destination) = PortalTabOpen::from_response(&response, || {
                            PortalTabTarget::Page(PortalPage::BackgroundWork)
                        }) {
                            open = Some(destination);
                        }
                    }
                    open
                })
                .inner
        });
    (area.inner, area.response.rect.height() + TOAST_MARGIN)
}

fn show_background_work_toast(
    ui: &mut Ui,
    job: &ActiveAppWork,
    now: Instant,
    demo_mode: bool,
) -> Response {
    let elapsed = now.saturating_duration_since(job.started_at).as_secs_f64();
    let description = if demo_mode {
        "Background work".to_owned()
    } else {
        escape_display(&job.description)
    };
    let text = format!("{elapsed:.1} s · {description}");
    toast_frame(ui, PortalPage::BackgroundWork, RichText::new(text))
}

fn toast_frame(ui: &mut Ui, page: PortalPage, text: RichText) -> Response {
    let content_width = (ui.available_width() - 2.0 * TOAST_MARGIN).max(0.0);
    let accessible_text = text.text().to_owned();
    let response = Frame::window(ui.style())
        .inner_margin(TOAST_MARGIN)
        .shadow(Shadow::NONE)
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.set_width(content_width);
            ui.horizontal(|ui| {
                ui.add(Image::new(page_icon(page)).fit_to_exact_size(Vec2::splat(16.0)));
                ui.add(Label::new(text).truncate());
            });
        })
        .response
        .interact(Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| {
        WidgetInfo::labeled(WidgetType::Button, response.enabled(), &accessible_text)
    });
    response.on_hover_text(accessible_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::Modifiers;
    use eframe::egui::PointerButton;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use std::sync::Arc;
    use std::time::SystemTime;
    use teamy_tracing::capture::LogRecord;

    fn log_notice(record_index: usize, now: Instant) -> PortalLogToast {
        PortalLogToast::from_record(
            record_index,
            &LogRecord {
                timestamp: SystemTime::UNIX_EPOCH,
                level: LogLevel::Info,
                target: Arc::from("test"),
                message: Arc::from("Log notice"),
                fields: Vec::new(),
                spans: Vec::new(),
                file: None,
                line: None,
            },
            now,
        )
        .unwrap()
    }

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
        let mut logs = vec![log_notice(0, now)];
        let mut draw = |work: &[ActiveAppWork], demo_mode, width| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 550.0))),
                ..Default::default()
            };
            let mut output = context.run_ui(input, |ui| {
                assert!(show_toast_stack(ui, &mut logs, work, demo_mode, now).is_none());
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

    #[tokio::test]
    async fn clicking_log_and_work_toasts_requests_the_expected_tab_navigation() -> eyre::Result<()>
    {
        let task = tokio::spawn(async {});
        let now = Instant::now();
        let work = [ActiveAppWork {
            id: task.id(),
            description: "Inspect resource inventory".to_owned(),
            started_at: now,
        }];
        task.await?;
        let context = Context::default();
        let mut logs = vec![log_notice(42, now)];
        let draw = |logs: &mut Vec<PortalLogToast>, now, modifiers, mut events: Vec<Event>| {
            events.insert(0, Event::ModifiersChanged(modifiers));
            let mut open = None;
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 550.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    open = show_toast_stack(ui, logs, &work, false, now).or(open.take());
                },
            );
            output.textures_delta.clear();
            (open, output.shapes)
        };
        let mut frame_time = now;
        draw(&mut logs, frame_time, Modifiers::NONE, Vec::new());
        let (_, shapes) = draw(&mut logs, frame_time, Modifiers::NONE, Vec::new());

        for (label, modifiers, expected) in [
            (
                "Log notice",
                Modifiers::CTRL,
                (
                    PortalTabOpen::Background(PortalTabTarget::Page(PortalPage::Logs)),
                    Some(42),
                ),
            ),
            (
                "Inspect resource inventory",
                Modifiers::NONE,
                (
                    PortalTabOpen::Current(PortalTabTarget::Page(PortalPage::BackgroundWork)),
                    None,
                ),
            ),
        ] {
            let pos = shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    Shape::Text(text) if text.galley.text().ends_with(label) => {
                        Some(text.visual_bounding_rect().center())
                    }
                    _ => None,
                })
                .unwrap();
            draw(
                &mut logs,
                frame_time,
                modifiers,
                vec![Event::PointerMoved(pos)],
            );
            if expected.1.is_some() {
                let progress = logs[0].progress();
                frame_time += Duration::from_secs(6);
                draw(&mut logs, frame_time, modifiers, Vec::new());
                assert_eq!(logs[0].progress(), progress);
            }
            for pressed in [true, false] {
                let (open, _) = draw(
                    &mut logs,
                    frame_time,
                    modifiers,
                    vec![Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers,
                    }],
                );
                if pressed {
                    assert!(open.is_none());
                } else {
                    assert_eq!(open.as_ref(), Some(&expected));
                }
            }
        }
        assert!(logs.is_empty());
        Ok(())
    }
}
