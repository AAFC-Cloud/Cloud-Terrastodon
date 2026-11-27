use cloud_terrastodon_azure::MicrosoftGraphResponse;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Context;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use teamy_cancellation::CancellationToken;
use tokio::sync::mpsc::UnboundedReceiver;

const LOOKUP_DEBOUNCE: Duration = Duration::from_millis(600);
type DirectoryPage<T> = (usize, Arc<MicrosoftGraphResponse<Arc<T>>>);

/// A tab's primary paginated directory inventory and the active Graph lookup.
pub(crate) struct EntraDirectoryInventory<T, I> {
    pub inventory: Loadable<Arc<Vec<Arc<T>>>>,
    pub lookup: Loadable<Vec<Arc<T>>>,
    pub pages: Option<UnboundedReceiver<DirectoryPage<T>>>,
    pub revision: u64,
    pub total: Option<usize>,
    partial: Vec<Arc<T>>,
    inventory_ids: HashSet<I>,
    lookup_ids: HashSet<I>,
    lookup_term: String,
    lookup_generation: u64,
    lookup_due: Option<Instant>,
    invalidate_lookup: bool,
    lookup_cancellation: Option<CancellationToken>,
}

impl<T, I> Default for EntraDirectoryInventory<T, I> {
    fn default() -> Self {
        Self {
            inventory: Loadable::NotLoaded,
            lookup: Loadable::NotLoaded,
            pages: None,
            revision: 0,
            total: None,
            partial: Vec::new(),
            inventory_ids: HashSet::new(),
            lookup_ids: HashSet::new(),
            lookup_term: String::new(),
            lookup_generation: 0,
            lookup_due: None,
            invalidate_lookup: false,
            lookup_cancellation: None,
        }
    }
}

impl<T, I: Copy + Eq + Hash> EntraDirectoryInventory<T, I> {
    pub fn set_inventory(&mut self, load: Loadable<Arc<Vec<Arc<T>>>>) {
        match &load {
            Loadable::Loading { .. } => self.reset_partial(),
            Loadable::Loaded { value, .. } => {
                self.pages = None;
                self.partial = Vec::new();
                self.total = Some(value.len());
                self.inventory_ids = HashSet::new();
            }
            _ => {}
        }
        self.inventory = load;
        self.revision += 1;
    }

    pub fn receive_pages(&mut self, id: impl Fn(&T) -> I) {
        let Some(mut pages) = self.pages.take() else {
            return;
        };
        while let Ok((index, page)) = pages.try_recv() {
            if index == 0 {
                self.reset_partial();
            }
            self.total = page.count.or(self.total);
            self.partial.extend(
                page.value
                    .iter()
                    .filter(|row| self.inventory_ids.insert(id(row)))
                    .cloned(),
            );
            self.revision += 1;
        }
        if !pages.is_closed() {
            self.pages = Some(pages);
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.lookup, Loadable::Loading { .. })
            || matches!(self.inventory, Loadable::Loading { .. })
    }

    pub fn primary_rows(&self) -> &[Arc<T>] {
        self.inventory
            .as_loaded()
            .map_or(self.partial.as_slice(), |complete| complete.as_slice())
    }

    pub fn rows(&self, id: impl Fn(&T) -> I) -> impl Iterator<Item = &T> {
        self.primary_rows()
            .iter()
            .filter(move |row| self.lookup_ids.is_empty() || !self.lookup_ids.contains(&id(row)))
            .chain(self.lookup.as_loaded().into_iter().flatten())
            .map(Arc::as_ref)
    }

    /// Graph already matched these rows, including UUIDs with alternate formatting and
    /// service-side name comparisons that differ from the local text filter.
    pub fn lookup_matched(&self, search: &str, id: I) -> bool {
        self.lookup_ids.contains(&id) && self.lookup_term.trim().to_lowercase() == search
    }

    pub fn find(&self, needle: I, id: impl Fn(&T) -> I) -> Option<&T> {
        self.lookup
            .as_loaded()
            .into_iter()
            .flatten()
            .chain(self.primary_rows())
            .map(Arc::as_ref)
            .find(|row| id(row) == needle)
    }

    /// Return the search term after a short idle period and identify its result.
    pub fn lookup_due(
        &mut self,
        term: &str,
        now: Instant,
        ctx: &Context,
    ) -> Option<(u64, String, bool)> {
        if self.lookup_term != term {
            self.reset_lookup();
            self.lookup_term = term.to_owned();
            self.lookup_due = (!term.trim().is_empty()).then_some(now + LOOKUP_DEBOUNCE);
            self.invalidate_lookup = false;
        }
        let due = self.lookup_due?;
        if due > now {
            ctx.request_repaint_after(due - now);
            return None;
        }
        self.lookup_due = None;
        Some((
            self.lookup_generation,
            self.lookup_term.trim().to_owned(),
            std::mem::take(&mut self.invalidate_lookup),
        ))
    }

    pub fn refresh_lookup(&mut self) {
        if !self.lookup_term.trim().is_empty() {
            self.reset_lookup();
            self.lookup_due = Some(Instant::now());
            self.invalidate_lookup = true;
        }
    }

    pub fn track_lookup(&mut self, cancellation: Option<CancellationToken>) {
        self.lookup_cancellation = cancellation;
    }

    pub fn set_lookup(
        &mut self,
        generation: u64,
        mut load: Loadable<Vec<Arc<T>>>,
        id: impl Fn(&T) -> I,
    ) {
        if self.lookup_generation != generation {
            return;
        }
        if !matches!(load, Loadable::Loading { .. }) {
            self.lookup_cancellation = None;
        }
        let had_rows = !self.lookup_ids.is_empty();
        self.lookup_ids.clear();
        if let Some(rows) = load.as_loaded_mut() {
            rows.retain(|row| self.lookup_ids.insert(id(row)));
        }
        self.lookup = load;
        if had_rows || !self.lookup_ids.is_empty() {
            self.revision += 1;
        }
    }

    fn reset_lookup(&mut self) {
        if let Some(cancel) = self.lookup_cancellation.take() {
            cancel.request_cancel("Entra directory search was replaced");
        }
        self.lookup_generation += 1;
        self.lookup = Loadable::NotLoaded;
        if !self.lookup_ids.is_empty() {
            self.lookup_ids.clear();
            self.revision += 1;
        }
    }

    fn reset_partial(&mut self) {
        self.partial.clear();
        self.total = None;
        self.inventory_ids.clear();
    }
}

impl<T, I> Drop for EntraDirectoryInventory<T, I> {
    fn drop(&mut self) {
        if let Some(cancel) = self.lookup_cancellation.take() {
            cancel.request_cancel("Entra directory search view closed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entra_search_waits_after_each_edit_and_cancels_superseded_work() {
        let ctx = Context::default();
        let start = Instant::now();
        let mut inventory = EntraDirectoryInventory::<u8, u8>::default();

        assert!(inventory.lookup_due("a", start, &ctx).is_none());
        let edit = start + LOOKUP_DEBOUNCE / 2;
        assert!(inventory.lookup_due("ab", edit, &ctx).is_none());
        assert!(
            inventory
                .lookup_due("ab", start + LOOKUP_DEBOUNCE, &ctx)
                .is_none()
        );

        let ready = edit + LOOKUP_DEBOUNCE;
        let (generation, query, refresh) = inventory.lookup_due("ab", ready, &ctx).unwrap();
        assert_eq!(query, "ab");
        assert!(!refresh);
        assert!(inventory.lookup_due("ab", ready, &ctx).is_none());

        let cancellation = CancellationToken::new();
        inventory.track_lookup(Some(cancellation.clone()));
        assert!(inventory.lookup_due("abc", ready, &ctx).is_none());
        assert!(cancellation.is_cancelled());
        inventory.set_lookup(
            generation,
            Loadable::Loaded {
                value: vec![Arc::new(1)],
                started_at: ready,
                finished_at: ready,
            },
            |row| *row,
        );
        assert!(inventory.lookup.as_loaded().is_none());
        assert!(
            inventory
                .lookup_due("", ready + LOOKUP_DEBOUNCE, &ctx)
                .is_none()
        );
    }

    #[test]
    fn complete_inventory_replaces_partial_pages_and_keeps_distinct_current_lookup_rows() {
        let now = Instant::now();
        let mut inventory = EntraDirectoryInventory::<u8, u8>::default();
        inventory.set_inventory(Loadable::Loading { started_at: now });
        let (send, pages) = tokio::sync::mpsc::unbounded_channel();
        inventory.pages = Some(pages);
        let publish = |index, row| {
            send.send((
                index,
                Arc::new(MicrosoftGraphResponse {
                    context: None,
                    count: None,
                    tips: None,
                    next_link: None,
                    value: vec![Arc::new(row)],
                }),
            ))
            .unwrap();
        };
        publish(0, 1);
        publish(1, 2);
        inventory.receive_pages(|row| *row);
        assert_eq!(
            inventory.rows(|row| *row).copied().collect::<Vec<_>>(),
            [1, 2]
        );
        publish(0, 3);
        inventory.receive_pages(|row| *row);
        assert_eq!(inventory.rows(|row| *row).copied().collect::<Vec<_>>(), [3]);

        inventory.lookup_generation = 3;
        let lookup = |rows: &[u8]| Loadable::Loaded {
            value: rows.iter().copied().map(Arc::new).collect(),
            started_at: now,
            finished_at: now,
        };
        inventory.set_lookup(3, lookup(&[3, 4, 4]), |row| *row);
        inventory.set_lookup(1, lookup(&[9]), |row| *row);
        assert_eq!(
            inventory.rows(|row| *row).copied().collect::<Vec<_>>(),
            [3, 4]
        );

        inventory.set_inventory(Loadable::Loaded {
            value: Arc::new(vec![Arc::new(4), Arc::new(5)]),
            started_at: now,
            finished_at: now,
        });
        assert_eq!(
            inventory.rows(|row| *row).copied().collect::<Vec<_>>(),
            [5, 3, 4]
        );
        assert!(inventory.partial.is_empty());
    }
}
