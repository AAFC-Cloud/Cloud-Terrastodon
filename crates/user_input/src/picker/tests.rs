use super::candidate_message::CandidateMessage;
use super::choice_pool::ChoicePool;
use super::handler_completion::HandlerCompletion;
use super::picker_event_state::PickerEventState;
use super::picker_tui::PickerToast;
use super::picker_tui::PickerTui;
use super::picker_tui::advance_toasts;
use super::picker_tui::empty_picker_message;
use super::picker_tui::handle_handler_completion;
use super::picker_tui::handle_key;
use super::picker_tui::new_nucleo;
use super::picker_tui::process_candidate_message;
use super::picker_tui::render_toasts;
use super::picker_tui::row_style;
use super::picker_tui::should_show_as_toast;
use super::picker_tui::startup_events;
use super::preserved_selection::preserved_selection;
use super::query_debouncer::QueryDebouncer;
use super::query_event::QueryEvent;
use super::should_warn_for_tab::should_warn_for_tab;
use crate::Choice;
use crate::PickerLogLevel;
use crate::PickerLogRecord;
use compact_str::CompactString;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rustc_hash::FxHashSet;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use tui_textarea::TextArea;

fn log_record(level: PickerLogLevel, message: &str) -> PickerLogRecord {
    PickerLogRecord {
        timestamp: SystemTime::UNIX_EPOCH,
        level,
        message: Arc::from(message),
        target: Arc::from("test"),
        fields: Vec::new(),
        spans: Vec::new(),
        file: None,
        line: None,
    }
}

#[test]
fn unions_candidates_and_latest_value_wins() {
    let mut pool = ChoicePool::default();
    assert_eq!(
        pool.inject(
            [
                Choice {
                    key: "one".into(),
                    value: 1,
                },
                Choice {
                    key: "two".into(),
                    value: 2,
                },
            ],
            |_| {},
        ),
        true
    );

    assert_eq!(
        pool.inject(
            [
                Choice {
                    key: "one".into(),
                    value: 10,
                },
                Choice {
                    key: "three".into(),
                    value: 3,
                },
            ],
            |_| {},
        ),
        true
    );
    assert_eq!(pool.len(), 3);
    assert_eq!(pool.get("one"), Some(&10));
    let mut keys = pool.keys().map(CompactString::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(keys, ["one", "three", "two"]);
}

#[test]
fn duplicate_candidate_keys_are_injected_once_but_latest_values_are_retained() {
    let mut state = PickerEventState::default();
    let mut nucleo = new_nucleo();
    let mut warned = FxHashSet::default();

    let (changed, _) = process_candidate_message(
        CandidateMessage {
            generation: state.generation,
            choices: vec![Choice {
                key: "same".into(),
                value: 1,
            }],
        },
        state.generation,
        &mut state,
        &mut nucleo,
        &mut warned,
    );
    assert!(changed);

    let (changed, _) = process_candidate_message(
        CandidateMessage {
            generation: state.generation,
            choices: vec![Choice {
                key: "same".into(),
                value: 2,
            }],
        },
        state.generation,
        &mut state,
        &mut nucleo,
        &mut warned,
    );
    assert!(!changed);
    assert_eq!(state.candidates.get("same"), Some(&2));

    nucleo.tick(100);
    assert_eq!(nucleo.snapshot().matched_items(..).count(), 1);
}

#[test]
fn completed_results_from_older_queries_remain_in_the_candidate_union() {
    let mut state = PickerEventState::default();
    let mut nucleo = new_nucleo();
    let mut warned = FxHashSet::default();

    for key in ["old query result", "new query result"] {
        let (changed, _) = process_candidate_message(
            CandidateMessage {
                generation: state.generation,
                choices: vec![Choice {
                    key: key.into(),
                    value: key.to_string(),
                }],
            },
            state.generation,
            &mut state,
            &mut nucleo,
            &mut warned,
        );
        assert!(changed);
    }

    assert_eq!(state.candidates.len(), 2);
    assert!(state.candidates.contains_key("old query result"));
    assert!(state.candidates.contains_key("new query result"));
}

#[test]
fn stale_query_results_remain_discarded_after_reload() {
    let mut state = PickerEventState::default();
    let mut nucleo = new_nucleo();
    let mut warned = FxHashSet::default();
    let (changed, warning) = process_candidate_message(
        CandidateMessage {
            generation: 0,
            choices: vec![Choice {
                key: "initial".into(),
                value: 1,
            }],
        },
        state.generation,
        &mut state,
        &mut nucleo,
        &mut warned,
    );
    assert!(changed);
    assert!(warning.is_empty());
    assert_eq!(state.candidates.get("initial"), Some(&1));

    state.reload();
    let (changed, _) = process_candidate_message(
        CandidateMessage {
            generation: 0,
            choices: vec![Choice {
                key: "stale".into(),
                value: 2,
            }],
        },
        state.generation,
        &mut state,
        &mut nucleo,
        &mut warned,
    );
    assert!(!changed);
    assert!(state.candidates.is_empty());

    let (changed, _) = process_candidate_message(
        CandidateMessage {
            generation: state.generation,
            choices: vec![Choice {
                key: "fresh".into(),
                value: 3,
            }],
        },
        state.generation,
        &mut state,
        &mut nucleo,
        &mut warned,
    );
    assert!(changed);
    assert_eq!(state.candidates.get("fresh"), Some(&3));
}

#[test]
fn handler_errors_are_returned_and_counts_are_released() {
    let mut pending = 1;
    let mut startup = 1;
    let result = handle_handler_completion(
        HandlerCompletion {
            is_startup: true,
            result: Err(eyre::eyre!("handler failed")),
        },
        &mut pending,
        &mut startup,
    );
    assert!(result.is_err());
    assert_eq!(pending, 0);
    assert_eq!(startup, 0);
}

#[test]
fn clearing_removes_all_candidates() {
    let mut pool = ChoicePool::default();
    pool.inject(
        [Choice {
            key: "one".into(),
            value: 1,
        }],
        |_| {},
    );
    pool.clear();
    assert_eq!(pool.len(), 0);
    assert_eq!(pool.keys().count(), 0);
}

#[test]
fn reload_increments_generation_and_clears_marked_candidates() {
    let mut state = PickerEventState::default();
    state.candidates.inject(
        [Choice {
            key: "one".into(),
            value: 1,
        }],
        |_| {},
    );
    state.marked.insert("one".into());

    state.reload();

    assert_eq!(state.generation, 1);
    assert_eq!(state.candidates.len(), 0);
    assert!(state.marked.is_empty());
}

#[test]
fn picker_accepts_initial_multi_selection_keys() {
    let picker = PickerTui::<String>::new().set_initial_selected(["first", "third", "first"]);

    assert_eq!(
        picker.initial_selected_keys(),
        &FxHashSet::from_iter([CompactString::from("first"), CompactString::from("third"),])
    );
}

#[test]
fn preserves_selection_when_result_order_changes() {
    let selected = CompactString::from("two");
    let keys = vec![CompactString::from("one"), CompactString::from("two")];
    assert_eq!(preserved_selection(Some(&selected), &keys), Some(1));
    assert_eq!(
        preserved_selection(Some(&selected), &[CompactString::from("one")]),
        Some(0)
    );
    let empty_keys: Vec<CompactString> = Vec::new();
    assert_eq!(preserved_selection(None, &empty_keys), None);
}

#[test]
fn query_changes_are_debounced_and_latest_query_wins() {
    let now = Instant::now();
    let mut debouncer = QueryDebouncer::default();
    debouncer.schedule("a".into(), now);
    debouncer.schedule("ab".into(), now + Duration::from_millis(10));

    assert_eq!(debouncer.take_due(now + Duration::from_millis(50)), None);
    assert_eq!(
        debouncer.take_due(now + Duration::from_millis(60)),
        Some(QueryEvent::Changed("ab".into()))
    );
    debouncer.schedule(String::new(), now + Duration::from_millis(70));
    assert_eq!(
        debouncer.take_due(now + Duration::from_millis(120)),
        Some(QueryEvent::Cleared)
    );
}

#[test]
fn typing_a_query_reanchors_the_picker_at_the_first_result() {
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(40));
    *list_state.offset_mut() = 40;
    let mut marked_for_return = FxHashSet::default();
    let mut query_text_area = TextArea::default();
    let mut query_changed = false;
    let mut selection_needs_reset = false;
    let mut query_debouncer = QueryDebouncer::default();
    let mut return_reason = None;

    let _effects = handle_key(
        KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
        false,
        &mut list_state,
        &[],
        &mut marked_for_return,
        &mut query_text_area,
        &mut query_changed,
        &mut selection_needs_reset,
        &mut query_debouncer,
        &mut return_reason,
    );

    assert!(query_changed);
    assert!(selection_needs_reset);
    assert_eq!(list_state.selected(), Some(0));
    assert_eq!(list_state.offset(), 0);
}

#[test]
fn startup_events_load_before_a_nonempty_default_query() {
    assert_eq!(
        startup_events("smith"),
        vec![
            super::picker_event::PickerEvent::InitialLoad,
            super::picker_event::PickerEvent::QueryChanged(Arc::from("smith")),
        ]
    );
    assert_eq!(
        startup_events(""),
        vec![super::picker_event::PickerEvent::InitialLoad]
    );
}

#[test]
fn tab_warning_is_deduplicated_by_exact_key() {
    let mut warned = FxHashSet::default();
    let malformed = CompactString::from("Smith\tJoe");
    let normal = CompactString::from("Smith Joe");

    assert!(should_warn_for_tab(&mut warned, &malformed));
    assert!(!should_warn_for_tab(&mut warned, &malformed));
    assert!(!should_warn_for_tab(&mut warned, &normal));
}

#[test]
fn candidate_batches_queue_each_new_tab_warning() {
    let mut state = PickerEventState::default();
    let mut nucleo = new_nucleo();
    let mut warned = FxHashSet::default();
    let (changed, warnings) = process_candidate_message(
        CandidateMessage {
            generation: state.generation,
            choices: vec![
                Choice {
                    key: "first\tmalformed".into(),
                    value: 1,
                },
                Choice {
                    key: "second\tmalformed".into(),
                    value: 2,
                },
            ],
        },
        state.generation,
        &mut state,
        &mut nucleo,
        &mut warned,
    );

    assert!(changed);
    assert_eq!(
        warnings,
        vec![
            CompactString::from("first\tmalformed"),
            CompactString::from("second\tmalformed"),
        ]
    );
}

#[test]
fn toast_projection_filters_levels_and_keeps_repeated_messages() {
    assert!(should_show_as_toast(PickerLogLevel::Info));
    assert!(should_show_as_toast(PickerLogLevel::Warn));
    assert!(should_show_as_toast(PickerLogLevel::Error));
    assert!(!should_show_as_toast(PickerLogLevel::Debug));

    let now = Instant::now();
    let mut toasts = vec![
        PickerToast {
            record: log_record(PickerLogLevel::Info, "same message"),
            expires_at: now + Duration::from_secs(4),
        },
        PickerToast {
            record: log_record(PickerLogLevel::Info, "same message"),
            expires_at: now + Duration::from_secs(4),
        },
    ];
    assert_eq!(toasts.len(), 2);
    assert!(!advance_toasts(&mut toasts, now));
}

#[test]
fn expired_toasts_are_removed_and_active_toasts_stack_upward() {
    let now = Instant::now();
    let mut toasts = vec![PickerToast {
        record: log_record(PickerLogLevel::Info, "expired"),
        expires_at: now - Duration::from_millis(1),
    }];
    assert!(advance_toasts(&mut toasts, now));
    assert!(toasts.is_empty());

    let mut toasts = vec![
        PickerToast {
            record: log_record(PickerLogLevel::Info, "bottom"),
            expires_at: now + Duration::from_secs(4),
        },
        PickerToast {
            record: log_record(PickerLogLevel::Warn, "top"),
            expires_at: now + Duration::from_secs(4),
        },
    ];
    let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 5));
    for x in 20..30 {
        buffer[(x, 3)].set_symbol("underlying");
        buffer[(x, 4)].set_symbol("underlying");
    }
    render_toasts(&mut buffer, Rect::new(0, 0, 30, 5), &toasts);
    let bottom = buffer.content().iter().any(|cell| cell.symbol() == "b");
    let top = buffer.content().iter().any(|cell| cell.symbol() == "t");
    assert!(bottom && top);
    assert_eq!(buffer[(23, 4)].symbol(), "t");
    assert_eq!(buffer[(20, 3)].symbol(), "b");
    assert_eq!(buffer[(26, 3)].symbol(), " ");
    assert_eq!(buffer[(26, 4)].symbol(), " ");
    assert!(toasts.pop().is_some());
}

#[test]
fn empty_picker_message_distinguishes_initial_and_query_empty_states() {
    assert_eq!(
        empty_picker_message(true),
        "No choices yet, try typing to search"
    );
    assert_eq!(empty_picker_message(false), "No results");
}

#[test]
fn picker_row_styles_distinguish_marked_and_cursor_states() {
    assert_eq!(row_style(false, false).bg, None);
    assert_eq!(row_style(false, true).bg, Some(ratatui::style::Color::Blue));
    assert_eq!(
        row_style(true, false).bg,
        Some(ratatui::style::Color::DarkGray)
    );
    assert_eq!(
        row_style(true, true).bg,
        Some(ratatui::style::Color::Magenta)
    );
}

#[test]
fn marked_picker_rows_are_indented_and_have_a_visible_dot() {
    let keys = vec![
        Arc::new(CompactString::from("cursor")),
        Arc::new(CompactString::from("marked")),
    ];
    let heights = vec![1, 1];
    let marked = FxHashSet::from_iter([CompactString::from("marked")]);
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(0));
    let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 5));

    super::picker_tui::render_picker_list(
        &mut buffer,
        Rect::new(0, 0, 24, 5),
        None,
        true,
        &keys,
        &heights,
        &marked,
        &mut list_state,
    );

    assert_eq!(buffer[(1, 1)].symbol(), "c");
    assert_eq!(buffer[(3, 2)].symbol(), "•");
    assert_eq!(buffer[(5, 2)].symbol(), "m");
}

#[test]
fn large_candidate_pool_only_formats_the_visible_rows() {
    let keys = (0..100_000)
        .map(|index| {
            let key = if (49_990..=50_010).contains(&index) {
                "VISIBLE"
            } else {
                "OFFSCREEN"
            };
            Arc::new(CompactString::from(key))
        })
        .collect::<Vec<_>>();
    let heights = vec![1; keys.len()];
    let marked = FxHashSet::default();
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(50_000));
    let mut buffer = Buffer::empty(Rect::new(0, 0, 32, 12));

    super::picker_tui::render_picker_list(
        &mut buffer,
        Rect::new(0, 0, 32, 12),
        None,
        false,
        &keys,
        &heights,
        &marked,
        &mut list_state,
    );

    assert!(buffer.content().iter().any(|cell| cell.symbol() == "V"));
    assert!(!buffer.content().iter().any(|cell| cell.symbol() == "O"));
    assert!(list_state.offset() >= 50_000 - 10);
}

#[test]
fn large_candidate_batch_injection_does_not_recurse_on_the_picker_stack() {
    let mut state = PickerEventState::default();
    let mut nucleo = new_nucleo();
    let mut warned = FxHashSet::default();
    let choices = (0..100_000)
        .map(|index| Choice {
            key: format!("user-{index}"),
            value: index,
        })
        .collect::<Vec<_>>();

    let (changed, warnings) = process_candidate_message(
        CandidateMessage {
            generation: state.generation,
            choices,
        },
        state.generation,
        &mut state,
        &mut nucleo,
        &mut warned,
    );

    assert!(changed);
    assert!(warnings.is_empty());
    assert_eq!(state.candidates.len(), 100_000);
    nucleo.tick(10);
}
