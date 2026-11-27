use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrder;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrderId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Context;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use teamy_cancellation::CancellationToken;

const LOOKUP_DEBOUNCE: Duration = Duration::from_millis(600);

/// Tracks the active exact-ID lookup for one work order list.
#[derive(Default)]
pub(crate) struct PortalHelixWorkOrderLookup {
    search: String,
    id: Option<BmcHelixWorkOrderId>,
    generation: u64,
    deadline: Option<Instant>,
    cancellation: Option<CancellationToken>,
    pub result: Loadable<Arc<Vec<BmcHelixWorkOrder>>>,
}

impl PortalHelixWorkOrderLookup {
    pub fn search_due(
        &mut self,
        search: &str,
        now: Instant,
        ctx: &Context,
    ) -> Option<(u64, BmcHelixWorkOrderId)> {
        if self.search != search {
            self.search.clear();
            self.search.push_str(search);
            let id = search.trim().to_ascii_uppercase().parse().ok();
            if self.id != id {
                self.cancel();
                self.deadline = id.as_ref().map(|_| now + LOOKUP_DEBOUNCE);
                self.id = id;
                ctx.request_repaint();
            }
        }
        let deadline = self.deadline?;
        if deadline > now {
            ctx.request_repaint_after(deadline - now);
            return None;
        }
        self.deadline = None;
        Some((self.generation, self.id.clone()?))
    }

    pub fn refresh(&mut self) {
        if self.id.is_some() {
            self.cancel();
            self.deadline = Some(Instant::now());
        }
    }

    pub fn is_searching(&self) -> bool {
        self.deadline.is_some() || self.is_loading()
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.result, Loadable::Loading { .. })
    }

    pub fn track(&mut self, cancellation: CancellationToken) {
        self.cancellation = Some(cancellation);
    }

    pub fn set_result(&mut self, generation: u64, result: Loadable<Arc<Vec<BmcHelixWorkOrder>>>) {
        if self.generation == generation {
            if !matches!(result, Loadable::Loading { .. }) {
                self.cancellation = None;
            }
            self.result = result;
        }
    }

    pub fn matching_order(&self) -> Option<&BmcHelixWorkOrder> {
        let id = self.id.as_ref()?;
        self.result
            .as_loaded()?
            .iter()
            .find(|order| &order.id == id)
    }

    fn cancel(&mut self) {
        if let Some(cancel) = self.cancellation.take() {
            cancel.request_cancel("Helix work order search was replaced");
        }
        self.generation = self.generation.wrapping_add(1);
        self.result = Loadable::NotLoaded;
    }
}

impl Drop for PortalHelixWorkOrderLookup {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancellation.take() {
            cancel.request_cancel("Helix work order search view closed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_order_lookup_debounces_edits_and_rejects_stale_results() -> eyre::Result<()> {
        let ctx = Context::default();
        let start = Instant::now();
        let mut lookup = PortalHelixWorkOrderLookup::default();
        assert!(lookup.search_due("summary text", start, &ctx).is_none());
        assert!(!lookup.is_searching());
        assert!(lookup.search_due("wo1", start, &ctx).is_none());
        let edit = start + LOOKUP_DEBOUNCE / 2;
        assert!(lookup.search_due("wo12", edit, &ctx).is_none());
        assert!(
            lookup
                .search_due("wo12", start + LOOKUP_DEBOUNCE, &ctx)
                .is_none()
        );

        let now = edit + LOOKUP_DEBOUNCE;
        let (generation, id) = lookup.search_due(" wo12 ", now, &ctx).unwrap();
        assert_eq!(id.as_str(), "WO12");
        let cancellation = CancellationToken::new();
        lookup.track(cancellation.clone());
        assert!(lookup.search_due("wo123", now, &ctx).is_none());
        assert!(cancellation.is_cancelled());
        let (current, id) = lookup
            .search_due("wo123", now + LOOKUP_DEBOUNCE, &ctx)
            .unwrap();
        let loaded = |id| Loadable::Loaded {
            value: Arc::new(vec![BmcHelixWorkOrder {
                id,
                values: Default::default(),
            }]),
            started_at: now,
            finished_at: now,
        };
        lookup.set_result(generation, loaded("WO12".parse()?));
        assert!(lookup.result.as_loaded().is_none());
        lookup.set_result(current, loaded(id.clone()));
        assert_eq!(lookup.matching_order().map(|order| &order.id), Some(&id));
        assert!(lookup.search_due("", now, &ctx).is_none());
        assert!(lookup.matching_order().is_none());
        Ok(())
    }
}
