use crate::app::MyApp;
use crate::inventory_table::show_inventory_table;
use crate::portal_log_exclusion::DEFAULT_LOG_EXCLUSIONS;
use crate::portal_log_exclusion::PortalLogExclusion;
use crate::portal_search::portal_search;
use crate::table_row_background::TableRowBackground;
use eframe::egui::Button;
use eframe::egui::Checkbox;
use eframe::egui::CollapsingHeader;
use eframe::egui::Color32;
use eframe::egui::ComboBox;
use eframe::egui::Frame;
use eframe::egui::Label;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use egui_extras::Column;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use std::collections::HashSet;
use teamy_tracing::capture::LogLevel;
use teamy_tracing::capture::LogRecord;
use teamy_tracing::capture::LogValue;
use teamy_tracing::capture::format_record_plain;
use teamy_tracing::gui::DiagnosticReportView;
use teamy_tracing::gui::LogView;
use teamy_tracing::gui::escape_display;

impl MyApp {
    pub(crate) fn show_portal_logs(&mut self, ui: &mut Ui) {
        let mut end = 0;
        let records = self.log_capture.records_since(&mut end);
        let start = end - records.len();
        let captured = start..end;
        let selected_id = ui.make_persistent_id("selected-log-record");
        let mut selected = ui.data(|data| data.get_temp::<usize>(selected_id));
        let revealed_id = ui.make_persistent_id("revealed-log-record");
        let follow_id = ui.make_persistent_id("follow-latest-log");
        let mut follow_latest = ui.data(|data| {
            data.get_temp::<bool>(follow_id)
                .unwrap_or(self.log_view.follow_tail)
        });
        let jump = self.pending_log_jumps.remove(&self.tabs.active_id());
        let revealed = jump.or_else(|| ui.data(|data| data.get_temp::<usize>(revealed_id)));
        if let Some(jump) = jump {
            selected = captured.contains(&jump).then_some(jump);
            if selected.is_some() {
                follow_latest = false;
            }
        }
        let checked_id = ui.make_persistent_id("checked-log-records");
        let mut checked = ui
            .data_mut(|data| data.remove_temp::<HashSet<usize>>(checked_id))
            .unwrap_or_default();
        checked.retain(|id| captured.contains(id));
        let exclusions_id = ui.make_persistent_id("log-exclusions");
        let mut exclusions = ui.data(|data| {
            data.get_temp::<String>(exclusions_id)
                .unwrap_or_else(|| DEFAULT_LOG_EXCLUSIONS.to_owned())
        });
        let mut clear = false;
        ui.horizontal_wrapped(|ui| {
            portal_search(ui, &mut self.log_view.filter, "Search logs", 210.0);
            ui.label("Exclude");
            if portal_search(ui, &mut exclusions, "Text to exclude", 145.0)
                .on_hover_text(
                    "Separate rules with commas. Use target[message] for an exact target and message match.",
                )
                .changed()
            {
                ui.data_mut(|data| data.insert_temp(exclusions_id, exclusions.clone()));
            }
            ComboBox::from_id_salt("minimum-log-level")
                .selected_text(level_text(self.log_view.minimum_level))
                .show_ui(ui, |ui| {
                    for level in [
                        LogLevel::Trace,
                        LogLevel::Debug,
                        LogLevel::Info,
                        LogLevel::Warn,
                        LogLevel::Error,
                    ] {
                        ui.selectable_value(
                            &mut self.log_view.minimum_level,
                            level,
                            level_text(level),
                        );
                    }
                });
            ui.checkbox(&mut follow_latest, "Follow latest");
            if ui
                .add_enabled(
                    !checked.is_empty(),
                    Button::new(format!("Copy selected ({})", checked.len())),
                )
                .on_hover_text("Copy checked log entries with their details")
                .clicked()
            {
                ui.ctx().copy_text(selected_log_text(&records, start, &checked));
            }
            clear = ui.button("Clear").clicked();
        });
        if clear {
            self.log_capture.acknowledge_through(end);
            ui.data_mut(|data| {
                data.remove::<usize>(selected_id);
                data.remove::<usize>(revealed_id);
            });
            return;
        }
        if revealed.is_some_and(|id| !captured.contains(&id)) {
            ui.weak("This log entry is no longer available.");
        }

        if let Some(record) = selected
            .and_then(|selected| selected.checked_sub(start))
            .and_then(|index| records.get(index))
        {
            let details_height = (ui.available_height() * 0.38).min(220.0);
            Panel::bottom(ui.make_persistent_id("log-detail-pane"))
                .exact_size(details_height)
                .frame(Frame::NONE.inner_margin(4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("Log details");
                        if ui
                            .small_button("Copy")
                            .on_hover_text("Copy this log entry with its details")
                            .clicked()
                        {
                            ui.ctx().copy_text(log_record_text(record));
                        }
                        if ui.small_button("Close").clicked() {
                            selected = None;
                        }
                    });
                    ScrollArea::both()
                        .id_salt(("log-record-details", selected))
                        .auto_shrink([false, false])
                        .min_scrolled_height(0.0)
                        .max_height(ui.available_height())
                        .show(ui, |ui| show_record(ui, record));
                });
        } else {
            selected = None;
        }

        let excludes = PortalLogExclusion::parse_list(&exclusions);
        let visible = visible_log_records(
            &records,
            start,
            &self.log_view,
            &excludes,
            revealed.filter(|&id| selected == Some(id)),
        );
        show_log_records(
            ui,
            &visible,
            follow_latest,
            jump,
            &mut selected,
            &mut checked,
        );
        ui.data_mut(|data| {
            data.insert_temp(follow_id, follow_latest);
            if let Some(selected) = selected {
                data.insert_temp(selected_id, selected);
            } else {
                data.remove::<usize>(selected_id);
            }
            if let Some(revealed) = revealed.filter(|id| {
                selected == Some(*id) || (selected.is_none() && !captured.contains(id))
            }) {
                data.insert_temp(revealed_id, revealed);
            } else {
                data.remove::<usize>(revealed_id);
            }
            if !checked.is_empty() {
                data.insert_temp(checked_id, checked);
            }
        });
    }
}

fn visible_log_records<'a>(
    records: &'a [LogRecord],
    start: usize,
    view: &LogView,
    exclusions: &[PortalLogExclusion<'_>],
    revealed: Option<usize>,
) -> Vec<(usize, &'a LogRecord)> {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| (start + index, record))
        .filter(|(id, record)| {
            Some(*id) == revealed
                || (view.matches(record) && !exclusions.iter().any(|filter| filter.matches(record)))
        })
        .collect()
}

fn show_log_records(
    ui: &mut Ui,
    visible: &[(usize, &LogRecord)],
    follow_latest: bool,
    jump: Option<usize>,
    selected: &mut Option<usize>,
    checked: &mut HashSet<usize>,
) {
    const HEADER_HEIGHT: f32 = 26.0;
    const ROW_HEIGHT: f32 = 26.0;
    let row_spacing = ui.spacing().item_spacing.y;
    let height = (ui.available_height() - HEADER_HEIGHT - row_spacing).max(0.0);
    let tail_id = ui.make_persistent_id("log-table-tail");
    let tail = visible
        .last()
        .filter(|_| follow_latest)
        .map(|(identity, _)| (*identity, visible.len(), height.to_bits()));
    let jump_to_tail = ui.data_mut(|data| {
        let previous = data.remove_temp::<(usize, usize, u32)>(tail_id);
        if let Some(tail) = tail {
            data.insert_temp(tail_id, tail);
        }
        tail.is_some() && tail != previous
    });
    let row_pitch = ROW_HEIGHT + row_spacing;
    let rows_height = visible.len() as f32 * row_pitch - row_spacing;
    let max_offset = (rows_height - height).max(0.0);
    let scroll_offset = jump
        .and_then(|id| visible.binary_search_by_key(&id, |(id, _)| *id).ok())
        .map(|row| ((row as f32 + 0.5) * row_pitch - height * 0.5).clamp(0.0, max_offset))
        .or_else(|| jump_to_tail.then_some(max_offset));

    ui.push_id("log-table", |ui| {
        show_inventory_table(ui, |table| {
            let table = table
                .id_salt("log-records")
                .stick_to_bottom(follow_latest)
                .max_scroll_height(height)
                .column(Column::exact(32.0))
                .column(Column::initial(96.0).at_least(80.0).clip(true))
                .column(Column::initial(56.0).at_least(48.0).clip(true))
                .column(Column::initial(156.0).at_least(80.0).clip(true))
                .column(Column::remainder().at_least(60.0).clip(true));
            let table = if let Some(offset) = scroll_offset {
                // Establish the offset before the table chooses which rows to draw.
                table.vertical_scroll_offset(offset)
            } else {
                table
            };
            table
                .header(HEADER_HEIGHT, |mut header| {
                    header.col(|ui| {
                        let count = visible
                            .iter()
                            .filter(|(id, _)| checked.contains(id))
                            .count();
                        let mut all = !visible.is_empty() && count == visible.len();
                        let indeterminate = count > 0 && !all;
                        if ui
                            .add_enabled(
                                !visible.is_empty(),
                                Checkbox::without_text(&mut all).indeterminate(indeterminate),
                            )
                            .on_hover_text("Select all matching logs")
                            .changed()
                        {
                            for &(id, _) in visible {
                                if all {
                                    checked.insert(id);
                                } else {
                                    checked.remove(&id);
                                }
                            }
                            ui.ctx().request_repaint();
                        }
                    });
                    for title in ["Time", "Level", "Target", "Message"] {
                        header.col(|ui| {
                            ui.strong(title);
                        });
                    }
                })
                .body(|mut body| {
                    let background = TableRowBackground::new(&mut body);
                    body.rows(ROW_HEIGHT, visible.len(), |mut row| {
                        let (identity, record) = visible[row.index()];
                        background.first_cell(&mut row, |ui| {
                            let mut is_checked = checked.contains(&identity);
                            if ui
                                .add(Checkbox::without_text(&mut is_checked))
                                .on_hover_text("Select log entry")
                                .changed()
                            {
                                if is_checked {
                                    checked.insert(identity);
                                } else {
                                    checked.remove(&identity);
                                }
                                ui.ctx().request_repaint();
                            }
                            is_checked
                        });
                        row.col(|ui| {
                            ui.add(
                                Label::new(
                                    Timestamp::try_from(record.timestamp)
                                        .map(|time| {
                                            time.to_zoned_datetime(TimeZone::system())
                                                .strftime("%H:%M:%S%.3f")
                                                .to_string()
                                        })
                                        .unwrap_or_else(|_| "Unknown time".into()),
                                )
                                .truncate(),
                            );
                        });
                        row.col(|ui| {
                            ui.label(
                                RichText::new(level_text(record.level))
                                    .color(level_color(record.level)),
                            );
                        });
                        row.col(|ui| {
                            let target = escape_display(&record.target);
                            ui.add(Label::new(&target).truncate()).on_hover_text(target);
                        });
                        row.col(|ui| {
                            let message = escape_display(&record.message);
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                            let label = RichText::new(&message);
                            let label = if *selected == Some(identity) {
                                label.strong()
                            } else {
                                label
                            };
                            if ui.link(label).on_hover_text(message).clicked() {
                                *selected = (*selected != Some(identity)).then_some(identity);
                                ui.ctx().request_repaint();
                            }
                        });
                    });
                });
        });
    });
}

fn selected_log_text(records: &[LogRecord], start: usize, checked: &HashSet<usize>) -> String {
    records
        .iter()
        .enumerate()
        .filter(|(index, _)| checked.contains(&(start + index)))
        .map(|(_, record)| log_record_text(record))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn log_record_text(record: &LogRecord) -> String {
    let timestamp = Timestamp::try_from(record.timestamp)
        .map(|time| {
            time.to_zoned_datetime(TimeZone::system())
                .strftime("%Y-%m-%dT%H:%M:%S%.3f%:z")
                .to_string()
        })
        .unwrap_or_else(|_| "Unknown time".into());
    let target = escape_display(&record.target);
    format!("[{timestamp}] {target}\n{}", format_record_plain(record))
}

fn show_record(ui: &mut Ui, record: &LogRecord) {
    ui.label(escape_display(&record.message));
    ui.monospace(escape_display(&record.target));
    if let Some(file) = &record.file {
        let file = escape_display(file);
        ui.monospace(
            record
                .line
                .map_or_else(|| file.to_owned(), |line| format!("{file}:{line}")),
        );
    }
    show_fields(ui, &record.fields);
    for (index, span) in record.spans.iter().enumerate() {
        let name = escape_display(&span.name);
        if span.fields.is_empty() {
            ui.monospace(name);
        } else {
            CollapsingHeader::new(name)
                .id_salt(index)
                .show(ui, |ui| show_fields(ui, &span.fields));
        }
    }
}

fn show_fields(ui: &mut Ui, fields: &[(String, LogValue)]) {
    for (name, value) in fields {
        let name = escape_display(name);
        match value {
            LogValue::Text(value) => {
                ui.monospace(format!("{name} · {}", escape_display(value)));
            }
            LogValue::Report(report) => {
                ui.collapsing(name, |ui| DiagnosticReportView::new(report).show(ui));
            }
        }
    }
}

fn level_color(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Trace | LogLevel::Debug => Color32::GRAY,
        LogLevel::Info => Color32::from_rgb(0, 112, 190),
        LogLevel::Warn => Color32::from_rgb(153, 94, 0),
        LogLevel::Error => Color32::from_rgb(185, 35, 38),
    }
}

fn level_text(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "Trace",
        LogLevel::Debug => "Debug",
        LogLevel::Info => "Info",
        LogLevel::Warn => "Warn",
        LogLevel::Error => "Error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use eframe::egui::Vec2;
    use std::sync::Arc;
    use std::time::SystemTime;

    fn synthetic_log(message: &str) -> LogRecord {
        LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Warn,
            target: Arc::from("test"),
            message: Arc::from(message),
            fields: Vec::new(),
            spans: Vec::new(),
            file: None,
            line: None,
        }
    }

    #[test]
    fn copying_checked_entries_retains_details_and_capture_order() {
        let mut miss = synthetic_log("Cache miss");
        miss.fields
            .push(("reason".into(), LogValue::Text("stale".into())));
        let records = [
            miss,
            synthetic_log("Unselected"),
            synthetic_log("Fetched response"),
        ];
        let copy = selected_log_text(&records, 8, &HashSet::from([10, 7, 8]));

        assert!(copy.contains("Cache miss, reason: stale"));
        assert!(!copy.contains("Unselected"));
        assert!(copy.find("Cache miss").unwrap() < copy.find("Fetched response").unwrap());
    }

    #[test]
    fn default_exclusion_hides_native_and_bridged_simplecss_warnings() {
        let filters = PortalLogExclusion::parse_list(DEFAULT_LOG_EXCLUSIONS);
        let excludes = |record: &LogRecord| filters.iter().any(|filter| filter.matches(record));
        let mut record = synthetic_log("Selector skipped");
        record.target = Arc::from("simplecss::selector");
        assert!(excludes(&record));

        record.target = Arc::from("log");
        record
            .fields
            .push(("log.target".into(), LogValue::Text("simplecss".into())));
        assert!(excludes(&record));

        record.fields.clear();
        assert!(!excludes(&record));
    }

    #[test]
    fn default_figue_exclusion_requires_the_exact_message_and_target() {
        let filters = PortalLogExclusion::parse_list(DEFAULT_LOG_EXCLUSIONS);
        let excludes = |record: &LogRecord| filters.iter().any(|filter| filter.matches(record));
        let message = "extract_field_default: successfully extracted default";
        let target = "figue::schema::from_schema";
        let mut record = synthetic_log(message);
        assert!(!excludes(&record));

        record.target = Arc::from(target);
        assert!(excludes(&record));

        record.message = Arc::from(format!("{message} from an override"));
        assert!(!excludes(&record));

        record.target = Arc::from("log");
        record.message = Arc::from(message);
        record
            .fields
            .push(("log.target".into(), LogValue::Text(target.into())));
        assert!(excludes(&record));
    }

    #[test]
    fn bracketed_log_exclusions_retain_commas_inside_messages() {
        let filters =
            PortalLogExclusion::parse_list("worker[Ignored [body, headers], retrying], simplecss");
        let excludes = |record: &LogRecord| filters.iter().any(|filter| filter.matches(record));
        let mut record = synthetic_log("Ignored [body, headers], retrying");
        record.target = Arc::from("worker");
        assert!(excludes(&record));

        record.target = Arc::from("other");
        assert!(!excludes(&record));

        record.target = Arc::from("simplecss::selector");
        record.message = Arc::from("Selector skipped");
        assert!(excludes(&record));
    }

    #[test]
    fn follow_latest_jumps_to_the_end_after_enabling_and_appending() {
        let context = Context::default();
        let mut records: Vec<_> = (0..100)
            .map(|index| synthetic_log(&format!("Line {index}")))
            .collect();
        let draw = |records: &[LogRecord], follow, expected_message: &str| {
            let rows: Vec<_> = records.iter().enumerate().collect();
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 260.0))),
                ..Default::default()
            };
            let mut output = context.run_ui(input, |ui| {
                show_log_records(ui, &rows, follow, None, &mut None, &mut HashSet::new());
            });
            output.textures_delta.clear();
            output.shapes.iter().any(|clipped| {
                matches!(&clipped.shape, Shape::Text(text) if text.galley.text() == expected_message
                    && clipped.clip_rect.contains(text.visual_bounding_rect().center()))
            })
        };

        assert!(!draw(&records, false, "Line 99"));
        draw(&records, true, "Line 99");
        assert!(draw(&records, true, "Line 99"));

        records.push(synthetic_log("Appended"));
        draw(&records, true, "Appended");
        assert!(draw(&records, true, "Appended"));
    }

    #[test]
    fn jumping_to_a_filtered_log_reveals_it_at_the_requested_position() {
        const START: usize = 300;
        const TARGET: usize = START + 50;
        const MESSAGE: &str = "Hidden record";
        let context = Context::default();
        let mut records: Vec<_> = (0..100)
            .map(|index| synthetic_log(&format!("Visible record {index}")))
            .collect();
        records[TARGET - START] = synthetic_log(MESSAGE);
        records[TARGET - START].level = LogLevel::Info;
        let view = LogView {
            filter: "Visible".to_owned(),
            minimum_level: LogLevel::Warn,
            ..Default::default()
        };
        let exclusions = PortalLogExclusion::parse_list("test[Hidden record]");
        let draw = |revealed, jump| {
            let visible = visible_log_records(&records, START, &view, &exclusions, revealed);
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 260.0))),
                ..Default::default()
            };
            let mut selected = revealed;
            let mut output = context.run_ui(input, |ui| {
                show_log_records(
                    ui,
                    &visible,
                    false,
                    jump,
                    &mut selected,
                    &mut HashSet::new(),
                );
            });
            output.textures_delta.clear();
            output.shapes.iter().any(|clipped| {
                matches!(&clipped.shape, Shape::Text(text) if text.galley.text() == MESSAGE
                    && clipped.clip_rect.contains(text.visual_bounding_rect().center()))
            })
        };

        let filtered = visible_log_records(&records, START, &view, &exclusions, None);
        assert!(!filtered.iter().any(|(id, _)| *id == TARGET));
        assert!(!draw(None, None));
        assert!(draw(Some(TARGET), Some(TARGET)));
        assert!(draw(Some(TARGET), None));
    }

    #[test]
    fn rest_debug_events_stay_quiet() {
        use crate::portal_log_toast::PortalLogToast;
        use std::time::Instant;

        let now = Instant::now();
        let mut record = LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Debug,
            target: Arc::from("cloud_terrastodon_rest::request_execution"),
            message: Arc::from("Replayed cached REST response"),
            fields: Vec::new(),
            spans: Vec::new(),
            file: None,
            line: None,
        };
        assert!(PortalLogToast::from_record(0, &record, now).is_none());

        record.message = Arc::from("Executing REST request");
        record
            .fields
            .push(("network_request".into(), LogValue::Text("true".into())));
        assert!(PortalLogToast::from_record(1, &record, now).is_none());
    }
}
