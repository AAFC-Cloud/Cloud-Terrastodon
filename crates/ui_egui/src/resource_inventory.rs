use crate::portal_tab_open::PortalTabOpen;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_group::ReflectedTableGroup;
use crate::reflected_table_rows::ReflectedTableRows;
use crate::resource_inventory_grouping::ResourceInventoryGrouping;
use crate::resource_inventory_row::ResourceInventoryRow;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure::SubscriptionId;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr;
use std::rc::Rc;
use std::rc::Weak as RcWeak;
use std::sync::Arc;
use std::sync::Weak;

/// Prepared rows for the current resource inventory view. Scrolling reuses the
/// projections and their order while the loaded snapshots and filters match.
pub(crate) struct ResourceInventory {
    // Weak references keep allocation identity without retaining old API results.
    resources: Weak<Vec<Arc<Resource>>>,
    subscriptions: Option<RcWeak<Vec<Arc<Subscription>>>>,
    view: Id,
    search: String,
    selected_subscriptions: HashSet<SubscriptionId>,
    grouping: Option<ResourceInventoryGrouping>,
    // Keep the projections and index order together so the indices stay valid.
    rows: Vec<ResourceInventoryRow>,
    order: Option<ReflectedTableRows>,
    pub total: usize,
}

impl ResourceInventory {
    /// Reuse the current view when both source allocations and the normalized
    /// filters match. Keep one projection across frames and release it on a miss.
    pub fn get_or_build<'a>(
        inventory: &'a mut Option<Self>,
        resources: &Arc<Vec<Arc<Resource>>>,
        subscriptions: Option<&Rc<Vec<Arc<Subscription>>>>,
        view: Id,
        search: &str,
        selected_subscriptions: &HashSet<SubscriptionId>,
        prepare: impl FnOnce() -> (Vec<ResourceInventoryRow>, usize),
    ) -> &'a mut Self {
        if inventory.as_ref().is_some_and(|current| {
            !ptr::eq(current.resources.as_ptr(), Arc::as_ptr(resources))
                || current.subscriptions.as_ref().map(RcWeak::as_ptr)
                    != subscriptions.map(Rc::as_ptr)
                || current.view != view
                || current.search != search
                || &current.selected_subscriptions != selected_subscriptions
        }) {
            *inventory = None;
        }
        inventory.get_or_insert_with(|| {
            let (rows, total) = prepare();
            Self {
                resources: Arc::downgrade(resources),
                subscriptions: subscriptions.map(Rc::downgrade),
                view,
                search: search.to_owned(),
                selected_subscriptions: selected_subscriptions.clone(),
                grouping: None,
                rows,
                total,
                order: None,
            }
        })
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        table: &ReflectedTable<'_>,
        checked_rows: &mut HashMap<Id, bool>,
        grouping: Option<ResourceInventoryGrouping>,
    ) -> Option<PortalTabOpen> {
        if self.grouping != grouping {
            self.grouping = grouping;
            self.order = None;
        }
        let group = grouping.map(|grouping| move |row: &ResourceInventoryRow| grouping.group(row));
        table.show_cached(
            ui,
            &self.rows,
            checked_rows,
            &mut self.order,
            |row| Id::new(("resource-row", &row.id)),
            group
                .as_ref()
                .map(|group| group as &dyn Fn(&ResourceInventoryRow) -> ReflectedTableGroup),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn repaints_reuse_the_prepared_rows_until_a_snapshot_or_filter_changes() {
        let mut inventory = None;
        let resources = Arc::new(Vec::new());
        let refreshed = Arc::new(Vec::new());
        let subscriptions = Rc::new(Vec::new());
        let view = Id::new("all resources");
        let selected = HashSet::new();
        let builds = Cell::new(0);
        let prepare = || {
            builds.set(builds.get() + 1);
            (Vec::new(), 0)
        };

        for (resources, subscriptions, search, expected_builds) in [
            (&resources, None, "", 1),
            (&resources, None, "", 1),
            (&resources, Some(&subscriptions), "", 2),
            (&resources, Some(&subscriptions), "storage", 3),
            (&resources, Some(&subscriptions), "storage", 3),
            (&refreshed, Some(&subscriptions), "storage", 4),
        ] {
            ResourceInventory::get_or_build(
                &mut inventory,
                resources,
                subscriptions,
                view,
                search,
                &selected,
                prepare,
            );
            assert_eq!(builds.get(), expected_builds);
        }
    }
}
