//! Invocation, cancellation binding and simulation for registered operations.
//!
//! Context-free reflection, metadata and pure registrations belong to
//! `cloud_terrastodon_registry`; consumers import them from that owner.

mod bind_request_cancellation;
mod default_production;
mod default_production_field;
mod default_production_kind;
mod default_production_plan;
mod default_production_plan_kind;
mod discovery_arbitrary;
mod discovery_input;
mod discovery_plan_build_request;
mod discovery_reflection;
mod discovery_simulation;
mod discovery_value;
mod function;
mod function_executor;
mod function_invocation;
mod known_functions;
mod known_functions_iter;
mod registered_function;
mod runtime_function;

#[cfg(test)]
mod discovery_plan_tests;
#[cfg(test)]
mod discovery_simulation_tests;
#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod value_catalog_tests;

pub use bind_request_cancellation::CancellationBindingFn;
#[doc(hidden)]
pub use bind_request_cancellation::bind_request_cancellation;
pub use default_production::default_production_plan;
pub use default_production::invoke_with_default_input;
pub use default_production::shape_can_be_produced_from_defaults;
pub use default_production_field::DefaultProductionField;
pub use default_production_plan::DefaultProductionPlan;
pub use default_production_plan_kind::DefaultProductionPlanKind;
pub use discovery_plan_build_request::DiscoveryPlanBuildRequest;
pub use discovery_simulation::DiscoverySimulation;
pub use discovery_value::DiscoveryValue;
pub use function::Function;
pub use function_executor::AsyncValueFn;
pub use function_executor::FunctionExecutor;
pub use function_invocation::FunctionInvocation;
pub use function_invocation::InvocationFuture;
pub use function_invocation::invoke_result_future;
pub use known_functions::KnownFunctions;
pub use known_functions::functions_from;
pub use known_functions::functions_from_to;
pub use known_functions::functions_to;
pub use known_functions::known_functions;
pub use known_functions_iter::KnownFunctionsIter;
pub use linkme::distributed_slice;
pub use registered_function::RegisteredFunction;
pub use runtime_function::KNOWN_RUNTIME_FUNCTIONS;
pub use runtime_function::RuntimeFunction;
pub use teamy_cancellation::CancellationScope;
