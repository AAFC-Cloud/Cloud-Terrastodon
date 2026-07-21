use super::choice_pool::ChoicePool;
use super::picker_event_state::PickerEventState;
use super::preserved_selection::preserved_selection;
use super::query_debouncer::QueryDebouncer;
use super::query_event::QueryEvent;
use super::should_warn_for_tab::should_warn_for_tab;
use crate::Choice;
use compact_str::CompactString;
use rustc_hash::FxHashSet;
use std::time::Duration;
use std::time::Instant;

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
fn preserves_selection_when_result_order_changes() {
    let selected = CompactString::from("two");
    let keys = vec![CompactString::from("one"), CompactString::from("two")];
    assert_eq!(preserved_selection(Some(&selected), &keys), Some(1));
    assert_eq!(
        preserved_selection(Some(&selected), &[CompactString::from("one")]),
        Some(0)
    );
    assert_eq!(preserved_selection(None, &[]), None);
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
fn tab_warning_is_deduplicated_by_exact_key() {
    let mut warned = FxHashSet::default();
    let malformed = CompactString::from("Smith\tJoe");
    let normal = CompactString::from("Smith Joe");

    assert!(should_warn_for_tab(&mut warned, &malformed));
    assert!(!should_warn_for_tab(&mut warned, &malformed));
    assert!(!should_warn_for_tab(&mut warned, &normal));
}
