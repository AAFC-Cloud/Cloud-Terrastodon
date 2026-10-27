use crate::CancellationScope;
use crate::DiscoveryPlan;
use crate::RegisteredFunction;
use crate::RegisteredShape;
use crate::known_functions;
use facet::Facet;
use std::future::Future;
use std::pin::Pin;

/// Build an offline discovery graph from editable seed and target shapes.
///
/// Unlike the resulting [`DiscoveryPlan`], this is the plan's input contract.
/// Seed shapes are explicit assumptions, not values or credentials. Building
/// the plan does not construct those values or execute any catalog operation.
/// The invocation host supplies cancellation. The reflected `functions` default
/// opts into the linked registry, and callers can replace it with a subset.
#[derive(Clone, Debug, Facet)]
pub struct DiscoveryPlanBuildRequest {
    #[facet(default = DiscoveryPlanBuildRequest::default_functions())]
    pub functions: Vec<RegisteredFunction>,
    pub seeds: Vec<RegisteredShape>,
    pub targets: Vec<RegisteredShape>,
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
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

impl IntoFuture for DiscoveryPlanBuildRequest {
    type Output = eyre::Result<DiscoveryPlan>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            DiscoveryPlan::build(
                self.functions.iter().map(|function| function.function()),
                self.seeds.iter().map(|seed| seed.shape()),
                self.targets.iter().map(|target| target.shape()),
                &self.cancellation,
            )
        })
    }
}

crate::register_thing!(DiscoveryPlanBuildRequest);
crate::register_into_future!(DiscoveryPlanBuildRequest => DiscoveryPlan, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuntimeValue;
    use std::future::Ready;
    use teamy_cancellation::CancellationToken;

    #[derive(Facet)]
    struct Input;

    #[derive(Facet)]
    struct Output;

    #[derive(Facet)]
    struct InventoryRequest {
        input: Input,
    }

    impl IntoFuture for InventoryRequest {
        type Output = eyre::Result<Output>;
        type IntoFuture = Ready<Self::Output>;

        fn into_future(self) -> Self::IntoFuture {
            panic!("building an offline plan must not execute its requests")
        }
    }

    crate::register_into_future!(InventoryRequest => Output, effects = [Read]);

    #[tokio::test]
    async fn editable_seeds_and_catalog_control_reachability_without_invoking_requests()
    -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                for (functions, seeds, reachable) in [
                    (
                        vec![RegisteredFunction::from_function(
                            InventoryRequest::REGISTERED_INTO_FUTURE,
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
                            InventoryRequest::REGISTERED_INTO_FUTURE,
                        )],
                        vec![],
                        false,
                    ),
                ] {
                    let plan = DiscoveryPlanBuildRequest {
                        functions,
                        seeds,
                        targets: vec![RegisteredShape::from_shape(Output::SHAPE)],
                        cancellation: cancellation.clone(),
                    }
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
                // set the chosen lists and host scope, then apply field defaults.
                let request =
                    RuntimeValue::build_with(DiscoveryPlanBuildRequest::SHAPE, |partial| {
                        Ok(partial
                            .begin_field("seeds")?
                            .set(vec![RegisteredShape::from_shape(Input::SHAPE)])?
                            .end()?
                            .begin_field("targets")?
                            .set(vec![RegisteredShape::from_shape(Output::SHAPE)])?
                            .end()?
                            .begin_field("cancellation")?
                            .set(cancellation)?
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
                            InventoryRequest::REGISTERED_INTO_FUTURE
                        ))
                );
                let plan = (*request).await?;
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
