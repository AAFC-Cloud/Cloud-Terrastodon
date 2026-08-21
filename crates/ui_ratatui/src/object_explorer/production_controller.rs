use super::arena::Arena;
use super::borrow_graph::BorrowGraph;
use super::field_binding::FieldBinding;
use super::invocation_controller::InvocationController;
use super::invocation_host::InvocationHost;
use super::production_job::ProductionBatch;
use super::production_job::ProductionJobId;
use super::production_job::ProductionJobState;
use super::production_job::ProductionStrategy;
use super::production_job::ProductionUpdate;
use super::production_node::ProductionNode;
use super::production_node::ProductionNodeAdvance;
use super::slot_id::SlotId;
use super::value_builder::BuilderStore;
use super::work_budget::WorkBudget;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::ProductionKind;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::default_production_plan;
use cloud_terrastodon_registry::describe_function;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::functions_from;
use facet::Facet;
use facet::Shape;
use std::collections::BTreeMap;

struct ProductionJob {
    id: ProductionJobId,
    destination: SlotId,
    field: usize,
    root: ProductionNode,
    latest_root: Option<SlotId>,
}

impl ProductionJob {
    fn update(&self, state: ProductionJobState) -> ProductionUpdate {
        ProductionUpdate::new(
            self.id,
            self.destination,
            self.field,
            self.root.outer_input(),
            state,
        )
    }

    fn running_update(&self) -> ProductionUpdate {
        self.update(ProductionJobState::Running {
            latest_root: self.latest_root,
        })
    }
}

#[derive(Default)]
pub(crate) struct ProductionController {
    next_job: u64,
    jobs: BTreeMap<ProductionJobId, ProductionJob>,
    next_poll: Option<ProductionJobId>,
}

impl ProductionController {
    #[expect(
        clippy::too_many_arguments,
        reason = "production startup coordinates the arena, builders, invocations, and strategy"
    )]
    pub(crate) fn start(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        invocations: &mut InvocationController,
        invocation_host: &mut dyn InvocationHost,
        destination: SlotId,
        field: usize,
        function: &'static Function,
        strategy: ProductionStrategy,
        max_work: usize,
    ) -> Result<ProductionBatch, String> {
        if max_work == 0 {
            return Err("producer startup must allow at least one unit of work".to_owned());
        }
        self.validate_destination(builders, destination, field, function)?;
        if self
            .jobs
            .values()
            .any(|job| job.destination == destination && job.field == field)
        {
            return Err(format!(
                "slot {destination} field {field} already has an active producer"
            ));
        }

        let (root, manual_input) = match strategy {
            ProductionStrategy::Default => {
                let plan = default_production_plan(function.input_shape).ok_or_else(|| {
                    format!(
                        "no default production path is registered for {}",
                        describe_shape(function.input_shape)
                    )
                })?;
                (
                    ProductionNode::invoke(function, ProductionNode::from_default_plan(&plan)),
                    None,
                )
            }
            ProductionStrategy::Manual => {
                let (input, _) = builders
                    .create_and_finalize(arena, borrow_graph, function.input_shape)
                    .map_err(|error| {
                        format!(
                            "could not create {} producer request: {error}",
                            describe_shape(function.input_shape)
                        )
                    })?;
                (
                    ProductionNode::invoke(function, ProductionNode::existing(input)),
                    Some(input),
                )
            }
            ProductionStrategy::Arbitrary { bytes } => {
                let constructor =
                    arbitrary_constructor_for(function.input_shape).ok_or_else(|| {
                        format!(
                            "no ArbitraryBytes constructor is registered for {}",
                            describe_shape(function.input_shape)
                        )
                    })?;
                let input =
                    ProductionNode::invoke(constructor, ProductionNode::arbitrary_bytes(bytes));
                (ProductionNode::invoke(function, input), None)
            }
        };

        if let Err(error) = builders.set_field_and_finalize(
            arena,
            borrow_graph,
            destination,
            field,
            FieldBinding::PendingProducer,
        ) {
            if let Some(input) = manual_input {
                let _ = builders.delete(arena, borrow_graph, input);
            }
            return Err(format!(
                "could not reserve slot {destination} field {field} for a producer: {error}"
            ));
        }

        let id = ProductionJobId::new(self.next_job);
        self.next_job = self
            .next_job
            .checked_add(1)
            .expect("production job identity space exhausted");
        let replaced = self.jobs.insert(
            id,
            ProductionJob {
                id,
                destination,
                field,
                root,
                latest_root: manual_input,
            },
        );
        debug_assert!(replaced.is_none(), "production job ids are monotonic");
        if self.next_poll.is_none() {
            self.next_poll = Some(id);
        }

        let mut work = WorkBudget::new(max_work);
        let mut latest = self
            .jobs
            .get(&id)
            .expect("new production job is indexed")
            .running_update();
        while work.try_consume() {
            let Some(update) = self.advance_job(
                arena,
                builders,
                borrow_graph,
                invocations,
                invocation_host,
                id,
            ) else {
                break;
            };
            let terminal = update.state().is_terminal();
            latest = update;
            if terminal {
                self.jobs.remove(&id);
                break;
            }
        }
        Ok(ProductionBatch::new(
            vec![latest],
            self.jobs.len(),
            work.spent(),
        ))
    }

    pub(crate) fn advance(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        invocations: &mut InvocationController,
        invocation_host: &mut dyn InvocationHost,
        max_work: usize,
    ) -> Result<ProductionBatch, String> {
        if max_work == 0 {
            return Err("producer polling must allow at least one unit of work".to_owned());
        }
        let mut work = WorkBudget::new(max_work);
        let mut updates = Vec::new();
        let jobs_at_start = self.jobs.len();
        let mut inspected = 0;
        while inspected < jobs_at_start && work.try_consume() {
            let Some(id) = self.next_job_id() else {
                break;
            };
            self.next_poll = next_identity(id);
            inspected += 1;
            let Some(update) = self.advance_job(
                arena,
                builders,
                borrow_graph,
                invocations,
                invocation_host,
                id,
            ) else {
                continue;
            };
            if update.state().is_terminal() {
                self.jobs.remove(&id);
            }
            updates.push(update);
        }
        if self.jobs.is_empty() {
            self.next_poll = None;
        }
        Ok(ProductionBatch::new(updates, self.jobs.len(), work.spent()))
    }

    pub(crate) fn active_count(&self) -> usize {
        self.jobs.len()
    }

    fn validate_destination(
        &self,
        builders: &BuilderStore,
        destination: SlotId,
        field: usize,
        function: &'static Function,
    ) -> Result<(), String> {
        let builder = builders
            .builder(destination)
            .ok_or_else(|| format!("slot {destination} has no defined builder"))?;
        let field_shape = builder
            .field_shape(field)
            .ok_or_else(|| format!("slot {destination} has no field {field}"))?;
        let source_shape = RuntimeValue::preferred_field_source_shape(field_shape);
        if function.production_kind(source_shape) != Some(ProductionKind::Exact) {
            return Err(format!(
                "{} does not exactly produce {} for slot {destination} field {field}",
                describe_function(function),
                describe_shape(source_shape)
            ));
        }
        Ok(())
    }

    fn next_job_id(&self) -> Option<ProductionJobId> {
        let Some(start) = self.next_poll else {
            return self.jobs.first_key_value().map(|(id, _)| *id);
        };
        self.jobs
            .range(start..)
            .next()
            .or_else(|| self.jobs.first_key_value())
            .map(|(id, _)| *id)
    }

    fn advance_job(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        invocations: &mut InvocationController,
        invocation_host: &mut dyn InvocationHost,
        id: ProductionJobId,
    ) -> Option<ProductionUpdate> {
        let job = self.jobs.get_mut(&id)?;
        if !matches!(
            builders
                .builder(job.destination)
                .and_then(|builder| builder.field_binding(job.field)),
            Some(FieldBinding::PendingProducer)
        ) {
            return Some(job.update(ProductionJobState::Failed {
                message: format!(
                    "slot {} field {} is no longer waiting for producer job {}",
                    job.destination, job.field, job.id
                ),
            }));
        }

        match job
            .root
            .advance_one(arena, builders, borrow_graph, invocations, invocation_host)
        {
            ProductionNodeAdvance::Progress(root) => {
                job.latest_root = Some(root);
                Some(job.running_update())
            }
            ProductionNodeAdvance::Waiting => None,
            ProductionNodeAdvance::Ready(output) => {
                let state = match builders.complete_pending_field_and_finalize(
                    arena,
                    borrow_graph,
                    job.destination,
                    job.field,
                    FieldBinding::MoveFrom(output),
                ) {
                    Ok(destination_transition) => ProductionJobState::Complete {
                        output,
                        destination_transition,
                    },
                    Err(error) => ProductionJobState::Failed {
                        message: format!(
                            "could not move producer output slot {output} into slot {} field {}: {error}",
                            job.destination, job.field
                        ),
                    },
                };
                Some(job.update(state))
            }
            ProductionNodeAdvance::Failed(message) => {
                if matches!(
                    builders
                        .builder(job.destination)
                        .and_then(|builder| builder.field_binding(job.field)),
                    Some(FieldBinding::PendingProducer)
                ) {
                    let _ = builders.unset_field_and_finalize(
                        arena,
                        borrow_graph,
                        job.destination,
                        job.field,
                    );
                }
                Some(job.update(ProductionJobState::Failed { message }))
            }
        }
    }
}

pub(crate) fn arbitrary_constructor_for(shape: &'static Shape) -> Option<&'static Function> {
    functions_from(ArbitraryBytes::SHAPE)
        .into_iter()
        .find(|function| function.production_kind(shape) == Some(ProductionKind::Exact))
}

fn next_identity(id: ProductionJobId) -> Option<ProductionJobId> {
    id.get().checked_add(1).map(ProductionJobId::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena_slot_state::ArenaSlotState;
    use crate::object_explorer::tokio_invocation_host::TokioInvocationHost;
    use crate::object_explorer::value_builder::BuilderTransition;
    use arbitrary::Arbitrary;
    use cloud_terrastodon_registry::RuntimeValue;
    use cloud_terrastodon_registry::functions_from;
    use facet::Facet;
    use std::future::Future;
    use std::future::IntoFuture;
    use std::pin::Pin;

    #[derive(Clone, Debug, Arbitrary, Facet)]
    #[repr(C)]
    struct ProducerRequest {
        marker: u8,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct ProducedValue {
        marker: u8,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct ProductionDestination {
        value: ProducedValue,
    }

    impl IntoFuture for ProducerRequest {
        type Output = eyre::Result<ProducedValue>;
        type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

        fn into_future(self) -> Self::IntoFuture {
            Box::pin(async move {
                Ok(ProducedValue {
                    marker: self.marker,
                })
            })
        }
    }

    cloud_terrastodon_registry::register_thing!(ProducerRequest);
    cloud_terrastodon_registry::register_thing!(ProducedValue);
    cloud_terrastodon_registry::register_into_future!(ProducerRequest => ProducedValue);
    cloud_terrastodon_registry::register_arbitrary!(ProducerRequest);

    fn producer() -> &'static Function {
        functions_from(ProducerRequest::SHAPE)
            .into_iter()
            .find(|function| {
                function.production_kind(ProducedValue::SHAPE) == Some(ProductionKind::Exact)
            })
            .expect("test producer is registered")
    }

    fn attach_nothing(
        future: cloud_terrastodon_registry::InvocationFuture,
    ) -> cloud_terrastodon_registry::InvocationFuture {
        future
    }

    async fn finish(
        production: &mut ProductionController,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrows: &mut BorrowGraph,
        invocations: &mut InvocationController,
        host: &mut TokioInvocationHost,
    ) -> ProductionUpdate {
        for _ in 0..64 {
            invocations.poll(arena, builders, borrows, host);
            let batch = production
                .advance(arena, builders, borrows, invocations, host, 1)
                .unwrap();
            assert!(batch.work_spent() <= 1);
            if let Some(update) = batch
                .updates()
                .iter()
                .find(|update| update.state().is_terminal())
            {
                return update.clone();
            }
            tokio::task::yield_now().await;
        }
        panic!("finite producer did not finish")
    }

    #[tokio::test]
    async fn default_production_uses_real_bounded_arena_roots_for_every_step() {
        let mut arena = Arena::default();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let mut invocations = InvocationController::default();
        let mut host = TokioInvocationHost::new(attach_nothing);
        let mut production = ProductionController::default();
        let (destination, transition) = builders
            .create_and_finalize(&mut arena, &mut borrows, ProductionDestination::SHAPE)
            .unwrap();
        assert_eq!(transition, BuilderTransition::Building);

        let start = production
            .start(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut invocations,
                &mut host,
                destination,
                0,
                producer(),
                ProductionStrategy::Default,
                1,
            )
            .unwrap();
        assert_eq!(start.work_spent(), 1);
        assert_eq!(start.active_jobs(), 1);
        let request = start.updates()[0]
            .input()
            .expect("first bounded step creates the request root");

        let complete = finish(
            &mut production,
            &mut arena,
            &mut builders,
            &mut borrows,
            &mut invocations,
            &mut host,
        )
        .await;
        let ProductionJobState::Complete {
            output,
            destination_transition,
        } = complete.state()
        else {
            panic!("production should complete: {:?}", complete.state())
        };
        assert_eq!(*destination_transition, BuilderTransition::Ready);
        assert!(matches!(
            arena.slot(request).unwrap().state(),
            ArenaSlotState::Consumed
        ));
        assert!(matches!(
            arena.slot(*output).unwrap().state(),
            ArenaSlotState::Consumed
        ));
        assert_eq!(arena.allocated_slot_count(), 4);
        let value = arena
            .ready_value(destination)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<ProductionDestination>()
            .unwrap()
            .downcast::<ProductionDestination>()
            .unwrap();
        assert_eq!(value.value.marker, 0);
    }

    #[tokio::test]
    async fn manual_production_waits_for_its_visible_request_builder_then_auto_invokes() {
        let mut arena = Arena::default();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let mut invocations = InvocationController::default();
        let mut host = TokioInvocationHost::new(attach_nothing);
        let mut production = ProductionController::default();
        let (destination, _) = builders
            .create_and_finalize(&mut arena, &mut borrows, ProductionDestination::SHAPE)
            .unwrap();

        let start = production
            .start(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut invocations,
                &mut host,
                destination,
                0,
                producer(),
                ProductionStrategy::Manual,
                1,
            )
            .unwrap();
        let request = start.updates()[0]
            .input()
            .expect("manual strategy returns its ordinary builder root");
        assert!(builders.builder(request).is_some());
        assert_eq!(production.active_count(), 1);
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::InlineOwned(
                    RuntimeValue::from_box(Box::new(37_u8)).expect("u8 runtime value"),
                ),
            )
            .unwrap();

        let complete = finish(
            &mut production,
            &mut arena,
            &mut builders,
            &mut borrows,
            &mut invocations,
            &mut host,
        )
        .await;
        assert!(matches!(
            complete.state(),
            ProductionJobState::Complete { .. }
        ));
        let value = arena
            .ready_value(destination)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<ProductionDestination>()
            .unwrap()
            .downcast::<ProductionDestination>()
            .unwrap();
        assert_eq!(value.value.marker, 37);
    }

    #[tokio::test]
    async fn arbitrary_production_is_the_same_visible_root_and_invocation_pipeline() {
        let mut arena = Arena::default();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let mut invocations = InvocationController::default();
        let mut host = TokioInvocationHost::new(attach_nothing);
        let mut production = ProductionController::default();
        let (destination, _) = builders
            .create_and_finalize(&mut arena, &mut borrows, ProductionDestination::SHAPE)
            .unwrap();
        assert!(arbitrary_constructor_for(ProducerRequest::SHAPE).is_some());

        production
            .start(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut invocations,
                &mut host,
                destination,
                0,
                producer(),
                ProductionStrategy::Arbitrary { bytes: vec![0; 64] },
                1,
            )
            .unwrap();
        let complete = finish(
            &mut production,
            &mut arena,
            &mut builders,
            &mut borrows,
            &mut invocations,
            &mut host,
        )
        .await;
        assert!(matches!(
            complete.state(),
            ProductionJobState::Complete { .. }
        ));
        assert_eq!(
            arena.allocated_slot_count(),
            4,
            "destination, ArbitraryBytes, request, and output are ordinary roots"
        );
    }

    #[test]
    fn production_polling_visits_at_most_one_step_per_active_job() {
        let mut controller = ProductionController::default();
        assert_eq!(controller.active_count(), 0);
        assert_eq!(
            next_identity(ProductionJobId::new(9)),
            Some(ProductionJobId::new(10))
        );
        assert_eq!(
            controller
                .advance(
                    &mut Arena::default(),
                    &mut BuilderStore::default(),
                    &mut BorrowGraph::default(),
                    &mut InvocationController::default(),
                    &mut TokioInvocationHost::new(attach_nothing),
                    8,
                )
                .unwrap()
                .work_spent(),
            0
        );
    }
}
