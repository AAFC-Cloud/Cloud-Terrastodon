use crate::RegisteredFunction;
use crate::known_functions;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::RegisteredShape;
use facet::Facet;

/// Build an offline discovery graph from editable seed and target shapes.
///
/// Unlike the resulting [`DiscoveryPlan`], this is the plan's input contract.
/// Seed shapes are explicit assumptions, not values or credentials. Building
/// the plan does not construct those values or execute any catalog operation.
/// The executor supplies cancellation. The reflected `functions` default
/// opts into the linked registry, and callers can replace it with a subset.
#[derive(Clone, Debug, Facet)]
pub struct DiscoveryPlanBuildRequest {
    #[facet(default = DiscoveryPlanBuildRequest::default_functions())]
    pub functions: Vec<RegisteredFunction>,
    pub seeds: Vec<RegisteredShape>,
    pub targets: Vec<RegisteredShape>,
}

impl DiscoveryPlanBuildRequest {
    /// Snapshot the linked operation catalog without invoking its registrations.
    pub fn default_functions() -> Vec<RegisteredFunction> {
        known_functions()
            .iter()
            .map(RegisteredFunction::from_function)
            .collect()
    }
}

impl ExecutableRequest for DiscoveryPlanBuildRequest {
    type Error = eyre::Report;
    type Output = DiscoveryPlan;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            DiscoveryPlan::build(
                self.functions
                    .iter()
                    .map(|function| function.function().descriptor()),
                self.seeds.iter().map(|seed| seed.shape()),
                self.targets.iter().map(|target| target.shape()),
                executor.cancellation(),
            )
        })
    }
}

cloud_terrastodon_registry::register_thing!(DiscoveryPlanBuildRequest);
crate::register_executable_request!(DiscoveryPlanBuildRequest => DiscoveryPlan, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_execution::UnreachableAuthenticationProvider;
    use cloud_terrastodon_registry::RuntimeValue;
    use std::sync::Arc;
    use teamy_cancellation::CancellationScope;
    use teamy_cancellation::CancellationToken;

    fn executor(cancellation: CancellationScope) -> eyre::Result<CloudTerrastodonExecutor> {
        Ok(CloudTerrastodonExecutor::new(
            WorkCache::temporary()?,
            Arc::new(UnreachableAuthenticationProvider),
            cancellation,
        ))
    }

    #[derive(Facet)]
    struct Input;

    #[derive(Facet)]
    struct Output;

    #[derive(Facet)]
    struct InventoryRequest {
        input: Input,
    }

    impl ExecutableRequest for InventoryRequest {
        type Error = eyre::Report;
        type Output = Output;

        fn run<'a>(self, _: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
        where
            Self: 'a,
            Self::Output: 'a,
        {
            panic!("building an offline plan must not execute its requests")
        }
    }

    crate::register_executable_request!(InventoryRequest => Output, effects = [Read]);

    #[tokio::test]
    async fn editable_seeds_and_catalog_control_reachability_without_invoking_requests()
    -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let executor = executor(cancellation)?;
                for (functions, seeds, reachable) in [
                    (
                        vec![RegisteredFunction::from_function(
                            InventoryRequest::REGISTERED_EXECUTABLE_REQUEST,
                        )],
                        vec![RegisteredShape::from_shape(Input::SHAPE)],
                        true,
                    ),
                    (
                        vec![],
                        vec![RegisteredShape::from_shape(Input::SHAPE)],
                        false,
                    ),
                    (
                        vec![RegisteredFunction::from_function(
                            InventoryRequest::REGISTERED_EXECUTABLE_REQUEST,
                        )],
                        vec![],
                        false,
                    ),
                ] {
                    let plan = DiscoveryPlanBuildRequest {
                        functions,
                        seeds,
                        targets: vec![RegisteredShape::from_shape(Output::SHAPE)],
                    }
                    .execute(&executor)
                    .await?;
                    assert_eq!(
                        plan.nodes
                            .iter()
                            .find(|node| node.target)
                            .unwrap()
                            .reachable,
                        reachable
                    );
                }
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn reflected_construction_defaults_only_the_catalog_and_keeps_seed_choices_explicit()
    -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                // This is the same reflected construction path the explorer uses:
                // set chosen data, then apply field defaults; execution is bound later.
                let request =
                    RuntimeValue::build_with(DiscoveryPlanBuildRequest::SHAPE, |partial| {
                        Ok(partial
                            .begin_field("seeds")?
                            .set(vec![RegisteredShape::from_shape(Input::SHAPE)])?
                            .end()?
                            .begin_field("targets")?
                            .set(vec![RegisteredShape::from_shape(Output::SHAPE)])?
                            .end()?)
                    })?;
                let request = request
                    .into_box::<DiscoveryPlanBuildRequest>()?
                    .downcast::<DiscoveryPlanBuildRequest>()
                    .map_err(|_| eyre::eyre!("reflected request had an unexpected type"))?;
                assert!(
                    request
                        .functions
                        .contains(&RegisteredFunction::from_function(
                            InventoryRequest::REGISTERED_EXECUTABLE_REQUEST
                        ))
                );
                let plan = executor(cancellation)?.execute(*request).await?;
                assert!(
                    plan.nodes
                        .iter()
                        .find(|node| node.target)
                        .unwrap()
                        .reachable
                );
                Ok(())
            })
            .await
    }
}
