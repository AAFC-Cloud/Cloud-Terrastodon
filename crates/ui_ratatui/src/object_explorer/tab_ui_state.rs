use super::card_address::CardAddress;
use super::card_row_key::CardRowKey;
use super::end_scan::QueryTotal;
use super::open_tabs::OpenTabs;
use super::slot_id::SlotId;
use std::collections::BTreeMap;

/// Discardable presentation state for one arena-owned Tab.
///
/// Query truth deliberately cannot be stored here: the Tab's Breadcrumbs live
/// only in its ordinary arena value. Card and row locations are semantic
/// identities rather than flattened indices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TabUiState {
    selection: CardAddress,
    focused_row: Option<CardRowKey>,
    viewport_anchor: CardAddress,
    search_query: String,
    query_total: QueryTotal,
}

impl Default for TabUiState {
    fn default() -> Self {
        Self {
            selection: CardAddress::NewSlot,
            focused_row: None,
            viewport_anchor: CardAddress::NewSlot,
            search_query: String::new(),
            query_total: QueryTotal::Unknown,
        }
    }
}

impl TabUiState {
    pub(crate) const fn selection(&self) -> &CardAddress {
        &self.selection
    }

    pub(crate) fn select(&mut self, selection: CardAddress) {
        self.selection = selection;
    }

    pub(crate) const fn focused_row(&self) -> Option<&CardRowKey> {
        self.focused_row.as_ref()
    }

    pub(crate) fn focus_row(&mut self, focused_row: Option<CardRowKey>) {
        self.focused_row = focused_row;
    }

    pub(crate) const fn viewport_anchor(&self) -> &CardAddress {
        &self.viewport_anchor
    }

    pub(crate) fn set_viewport_anchor(&mut self, viewport_anchor: CardAddress) {
        self.viewport_anchor = viewport_anchor;
    }

    pub(crate) fn search_query(&self) -> &str {
        &self.search_query
    }

    pub(crate) fn set_search_query(&mut self, search_query: impl Into<String>) {
        self.search_query = search_query.into();
    }

    pub(crate) const fn query_total(&self) -> QueryTotal {
        self.query_total
    }

    pub(crate) fn set_query_total(&mut self, query_total: QueryTotal) {
        self.query_total = query_total;
    }
}

/// Session-local TabUiState index. SlotId is the only connection to a Tab.
#[derive(Debug, Default)]
pub(crate) struct TabUiStates {
    by_tab: BTreeMap<SlotId, TabUiState>,
}

impl TabUiStates {
    pub(crate) fn for_tab(&self, tab: SlotId) -> Option<&TabUiState> {
        self.by_tab.get(&tab)
    }

    pub(crate) fn for_tab_mut(&mut self, tab: SlotId) -> &mut TabUiState {
        self.by_tab.entry(tab).or_default()
    }

    pub(crate) fn discard(&mut self, tab: SlotId) -> Option<TabUiState> {
        self.by_tab.remove(&tab)
    }

    pub(crate) fn retain_open(&mut self, open_tabs: &OpenTabs) {
        self.by_tab.retain(|tab, _| open_tabs.ids().contains(tab));
    }

    pub(crate) fn len(&self) -> usize {
        self.by_tab.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::breadcrumb::Breadcrumb;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::end_scan::ScanProgress;
    use crate::object_explorer::tab::Tab;
    use crate::object_explorer::value_address::ValueAddress;
    use crate::object_explorer::value_path::ValuePathSegment;
    use cloud_terrastodon_registry::RuntimeValue;

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: facet::Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    fn cloned_tab(arena: &Arena, tab: SlotId) -> Box<Tab> {
        arena
            .ready_value(tab)
            .expect("Tab is Ready")
            .try_clone()
            .expect("Tab is reflectively cloneable")
            .into_box::<Tab>()
            .expect("cloned value retains Tab shape")
            .downcast::<Tab>()
            .expect("cloned value retains Tab type")
    }

    #[test]
    fn tabs_restore_logical_selection_and_query() {
        let first_query = Breadcrumbs::new(vec![Breadcrumb::ShapeFilter {
            included_shapes: vec!["AzureDevOpsProjectMember".to_owned()],
        }]);
        let second_query = Breadcrumbs::new(vec![Breadcrumb::Pop]);
        let mut arena = Arena::default();
        let first = arena
            .insert_ready(runtime(Tab::new("admins", first_query.clone())))
            .unwrap();
        let second = arena
            .insert_ready(runtime(Tab::new("members", second_query.clone())))
            .unwrap();
        let arena_revision = arena.arena_revision();
        let first_selection = CardAddress::Value(
            ValueAddress::root(SlotId::new(42))
                .child(ValuePathSegment::Field("permissionObjects".to_owned()))
                .child(ValuePathSegment::Index(4)),
        );
        let mut open_tabs = OpenTabs::default();
        let mut ui_states = TabUiStates::default();
        open_tabs.open(first);
        {
            let state = ui_states.for_tab_mut(first);
            state.select(first_selection.clone());
            state.focus_row(Some(CardRowKey::Field("displayName".to_owned())));
            state.set_viewport_anchor(first_selection.clone());
            state.set_search_query("project admin");
            state.set_query_total(QueryTotal::Scanning(ScanProgress {
                inspected: 80,
                matched: 3,
            }));
        }
        open_tabs.open(second);
        ui_states
            .for_tab_mut(second)
            .select(CardAddress::Value(ValueAddress::root(second)));

        assert!(open_tabs.activate(first));
        let restored = ui_states.for_tab(first).expect("first tab UI state");
        assert_eq!(restored.selection(), &first_selection);
        assert_eq!(
            restored.focused_row(),
            Some(&CardRowKey::Field("displayName".to_owned()))
        );
        assert_eq!(restored.viewport_anchor(), &first_selection);
        assert_eq!(restored.search_query(), "project admin");
        assert!(matches!(restored.query_total(), QueryTotal::Scanning(_)));

        let first_tab = cloned_tab(&arena, first);
        let second_tab = cloned_tab(&arena, second);
        assert_eq!(first_tab.breadcrumbs(), &first_query);
        assert_eq!(second_tab.breadcrumbs(), &second_query);
        assert_eq!(arena.arena_revision(), arena_revision);

        drop(ui_states);
        let recreated_ui_states = TabUiStates::default();
        assert!(recreated_ui_states.for_tab(first).is_none());
        assert_eq!(cloned_tab(&arena, first).breadcrumbs(), &first_query);
        assert_eq!(arena.arena_revision(), arena_revision);
    }

    #[test]
    fn closing_tabs_discards_only_session_local_ui_state() {
        let mut open_tabs = OpenTabs::default();
        let first = SlotId::new(2);
        let second = SlotId::new(9);
        open_tabs.open(first);
        open_tabs.open(second);
        let mut states = TabUiStates::default();
        states.for_tab_mut(first).set_search_query("first");
        states.for_tab_mut(second).set_search_query("second");

        assert!(open_tabs.close(first));
        states.retain_open(&open_tabs);

        assert_eq!(states.len(), 1);
        assert!(states.for_tab(first).is_none());
        assert_eq!(states.for_tab(second).unwrap().search_query(), "second");
        assert!(states.discard(second).is_some());
        assert_eq!(states.len(), 0);
    }
}
