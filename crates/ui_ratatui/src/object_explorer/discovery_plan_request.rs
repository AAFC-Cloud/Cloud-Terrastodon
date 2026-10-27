use super::arena_query_context::ArenaQueryContext;
use super::breadcrumbs::Breadcrumbs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::RegisteredShape;
use cloud_terrastodon_registry::known_functions;
use facet::Facet;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

/// Inspect the existing object pool and produce an offline discovery graph.
///
/// `inputs` is the same composable query used by explorer tabs and JSON export.
/// Only shapes of actual ready values selected by that query become seeds;
/// registering a type or declaring a missing request field does not supply it.
/// The resulting graph describes possible work. It never invokes cloud APIs,
/// defaults, arbitrary generators, or conversions, and holds no value bindings
/// or promises that the pool will remain unchanged for a later execution.
#[derive(Clone, Debug, Facet)]
pub(crate) struct DiscoveryPlanRequest {
    pub targets: Vec<RegisteredShape>,
    /// Defaults to the pool's ordinary unfiltered ready-value query.
    #[facet(default)]
    pub inputs: Breadcrumbs,
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
}

impl DiscoveryPlanRequest {
    /// Keep the engine capability and catalog explicit for non-UI callers and
    /// offline tests. Only IntoFuture opts into the installed host context.
    pub(crate) async fn run(
        self,
        context: ArenaQueryContext,
        functions: Vec<&'static Function>,
    ) -> eyre::Result<DiscoveryPlan> {
        let session = context.open_export(self.inputs).await?;
        let mut seeds = HashSet::new();
        loop {
            self.cancellation.bail_if_cancelled()?;
            let batch = session.next_shape_batch(256).await?;
            seeds.extend(batch.shapes);
            if batch.complete {
                break;
            }
        }
        session.complete().await?;
        self.cancellation.bail_if_cancelled()?;
        DiscoveryPlan::build(
            functions,
            seeds,
            self.targets.iter().map(|target| target.shape()),
            &self.cancellation,
        )
    }
}

impl IntoFuture for DiscoveryPlanRequest {
    type Output = eyre::Result<DiscoveryPlan>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Resolve the host capability when polled, after the invocation
            // host installs it. No new cancellation root or ambient auth lookup.
            self.run(
                ArenaQueryContext::current()?,
                known_functions().iter().collect(),
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(DiscoveryPlanRequest);
cloud_terrastodon_registry::register_into_future!(DiscoveryPlanRequest => DiscoveryPlan, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::super::arena::Arena;
    use super::super::breadcrumb::Breadcrumb;
    use super::super::explorer_engine::ExplorerEngine;
    use super::*;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_registry::RuntimeValue;
    use std::future::Ready;

    #[derive(Clone, Debug, Facet)]
    struct Auth;

    #[derive(Clone, Debug, Facet)]
    struct Inventory;

    #[derive(Facet)]
    struct InventoryRequest {
        auth: Auth,
    }

    impl IntoFuture for InventoryRequest {
        type Output = eyre::Result<Inventory>;
        type IntoFuture = Ready<Self::Output>;

        fn into_future(self) -> Self::IntoFuture {
            panic!("offline planning must never execute an inventory request")
        }
    }

    cloud_terrastodon_registry::register_into_future!(InventoryRequest => Inventory, effects = [Read]);

    #[tokio::test]
    async fn plan_uses_only_ready_values_selected_by_its_input_query() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                for (filter, reachable) in [("Auth", true), ("Inventory", false)] {
                    let mut arena = Arena::default();
                    arena.insert_ready(RuntimeValue::from_box(Box::new(Auth))?)?;
                    arena.reserve_builder()?;
                    let engine = ExplorerEngine::new(arena);
                    let (context, inbox) = ArenaQueryContext::channel(4);
                    let request = DiscoveryPlanRequest {
                        targets: vec![RegisteredShape::from_shape(Inventory::SHAPE)],
                        inputs: Breadcrumbs::new(vec![Breadcrumb::ShapeFilter {
                            included_shapes: vec![filter.to_owned()],
                        }]),
                        cancellation: cancellation.clone(),
                    };
                    let client =
                        request.run(context, vec![InventoryRequest::REGISTERED_INTO_FUTURE]);
                    let (_, plan) = tokio::join!(engine.run(inbox), client);
                    let plan = plan?;
                    assert_eq!(
                        plan.nodes
                            .iter()
                            .find(|node| node.target)
                            .unwrap()
                            .reachable,
                        reachable
                    );
                    assert_eq!(
                        plan.nodes
                            .iter()
                            .any(|node| node.seed && node.label == "Auth"),
                        reachable
                    );
                }
                Ok(())
            })
            .await
    }
}
