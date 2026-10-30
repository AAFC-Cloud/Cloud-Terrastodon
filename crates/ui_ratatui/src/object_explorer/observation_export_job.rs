use super::arena::Arena;
use super::arena_address_source::ArenaAddressSource;
use super::diagram_batch::DiagramBatch;
use super::diagram_observation;
use super::query_cursor::QueryCursor;
use super::query_plan::QueryPlan;
use super::query_progress::QueryProgressState;
use super::revision::ArenaRevision;
use super::revision::QueryRevision;
use super::revision::ScanRevisionStamp;
use super::shape_batch::ShapeBatch;
use super::value_builder::BuilderStore;
use super::work_budget::WorkBudget;
use std::num::NonZeroUsize;

/// A lazy observation cursor held by the engine's existing coherent export lease.
/// Each call advances bounded work and retains only the current response batch.
pub(crate) struct ObservationExportJob<'source, 'arena> {
    source: &'source ArenaAddressSource<'arena>,
    cursor: QueryCursor<'source, 'arena>,
    stamp: ScanRevisionStamp,
}

impl<'source, 'arena: 'source> ObservationExportJob<'source, 'arena> {
    pub(crate) fn new(
        source: &'source ArenaAddressSource<'arena>,
        plan: QueryPlan,
        revision: ArenaRevision,
    ) -> Self {
        let stamp = ScanRevisionStamp {
            arena: revision,
            query: QueryRevision::default(),
        };
        Self {
            source,
            cursor: QueryCursor::new(source, plan, stamp, NonZeroUsize::MIN.saturating_add(1)),
            stamp,
        }
    }

    pub(crate) fn next_diagram_batch(
        &mut self,
        arena: &Arena,
        builders: &BuilderStore,
        max_work: usize,
        max_objects: usize,
        max_rows: usize,
    ) -> Result<DiagramBatch, String> {
        if max_work == 0 || max_objects == 0 || max_rows == 0 {
            return Err("diagram work, object, and field limits must be greater than zero".into());
        }
        let before = self.cursor.instrumentation().addressed;
        let mut budget = WorkBudget::new(max_work);
        let mut objects = Vec::new();
        let mut complete = false;
        while objects.len() < max_objects {
            match self
                .cursor
                .next(self.stamp, &mut budget)
                .map_err(|error| error.to_string())?
                .into_state()
            {
                QueryProgressState::Ready(address) => objects.push(diagram_observation::observe(
                    arena,
                    builders,
                    self.source,
                    address,
                    max_rows,
                )?),
                QueryProgressState::Pending => break,
                QueryProgressState::Complete => {
                    complete = true;
                    break;
                }
                QueryProgressState::Stale | QueryProgressState::Cancelled => {
                    return Err("coherent diagram observation was invalidated".into());
                }
            }
        }
        Ok(DiagramBatch {
            objects,
            inspected: self
                .cursor
                .instrumentation()
                .addressed
                .saturating_sub(before),
            complete,
        })
    }

    pub(crate) fn next_shape_batch(&mut self, max_work: usize) -> Result<ShapeBatch, String> {
        if max_work == 0 {
            return Err("shape observation work limit must be greater than zero".into());
        }
        let before = self.cursor.instrumentation().addressed;
        let mut budget = WorkBudget::new(max_work);
        let mut shapes = Vec::new();
        let mut complete = false;
        while shapes.len() < max_work {
            match self
                .cursor
                .next(self.stamp, &mut budget)
                .map_err(|error| error.to_string())?
                .into_state()
            {
                QueryProgressState::Ready(address) => shapes.push(
                    self.source
                        .resolve(&address)
                        .map_err(|error| error.to_string())?
                        .shape(),
                ),
                QueryProgressState::Pending => break,
                QueryProgressState::Complete => {
                    complete = true;
                    break;
                }
                QueryProgressState::Stale | QueryProgressState::Cancelled => {
                    return Err("coherent shape observation was invalidated".into());
                }
            }
        }
        Ok(ShapeBatch {
            shapes,
            inspected: self
                .cursor
                .instrumentation()
                .addressed
                .saturating_sub(before),
            complete,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::borrow_graph::BorrowGraph;
    use crate::object_explorer::breadcrumb::Breadcrumb;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    #[derive(Facet)]
    struct Project {
        name: String,
        count: usize,
    }

    fn runtime<T: Facet<'static> + Send + 'static>(value: T) -> RuntimeValue {
        RuntimeValue::from_box(Box::new(value)).unwrap()
    }

    #[test]
    fn diagram_batches_include_incomplete_builders_and_obey_work_and_card_limits() {
        let mut arena = Arena::default();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let (building, _) = builders
            .create_and_finalize(&mut arena, &mut borrows, Project::SHAPE)
            .unwrap();
        for index in 0..45 {
            arena
                .insert_ready(runtime(Project {
                    name: format!("project {index}"),
                    count: index,
                }))
                .unwrap();
        }
        let source = ArenaAddressSource::object_pool(&arena);
        let mut job = ObservationExportJob::new(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            arena.arena_revision(),
        );
        let mut all = Vec::new();
        loop {
            let batch = job.next_diagram_batch(&arena, &builders, 7, 3, 32).unwrap();
            assert!(batch.inspected <= 7);
            assert!(batch.objects.len() <= 3);
            all.extend(batch.objects);
            if batch.complete {
                break;
            }
        }
        let builder = all
            .iter()
            .find(|item| item.address == format!("slot {building}"))
            .unwrap();
        assert_eq!(builder.shape, "Project");
        assert!(
            builder
                .fields
                .iter()
                .any(|field| field.name == "name" && field.value == "unset")
        );
        assert!(
            builder
                .fields
                .iter()
                .any(|field| field.name == "status" && field.value == "Building")
        );
        assert_eq!(
            all.iter().filter(|item| item.shape == "Project").count(),
            46
        );
        assert!(arena.ready_value(building).is_none());
    }

    #[test]
    fn tab_diagram_and_shape_evidence_follow_the_query_beyond_a_viewport() {
        let mut arena = Arena::default();
        let builders = BuilderStore::default();
        arena
            .insert_ready(runtime(
                (0..53)
                    .map(|count| Project {
                        name: format!("project {count}"),
                        count,
                    })
                    .collect::<Vec<_>>(),
            ))
            .unwrap();
        arena
            .insert_ready(runtime(String::from("excluded")))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let query = QueryPlan::new(Breadcrumbs::new(vec![Breadcrumb::ShapeFilter {
            included_shapes: vec![cloud_terrastodon_registry::describe_shape(Project::SHAPE)],
        }]));
        let mut diagrams =
            ObservationExportJob::new(&source, query.clone(), arena.arena_revision());
        let mut count = 0;
        loop {
            let batch = diagrams
                .next_diagram_batch(&arena, &builders, 5, 2, 1)
                .unwrap();
            for object in batch.objects {
                assert_eq!(object.shape, "Project");
                assert_eq!(object.fields[0].name, "name");
                assert!(!object.fields_complete);
                count += 1;
            }
            if batch.complete {
                break;
            }
        }
        assert_eq!(count, 53);
        let mut shapes = ObservationExportJob::new(&source, query, arena.arena_revision());
        let mut count = 0;
        loop {
            let batch = shapes.next_shape_batch(5).unwrap();
            assert!(batch.inspected <= 5);
            assert!(
                batch
                    .shapes
                    .iter()
                    .all(|shape| shape.is_shape(Project::SHAPE))
            );
            count += batch.shapes.len();
            if batch.complete {
                break;
            }
        }
        assert_eq!(count, 53);
    }
}
