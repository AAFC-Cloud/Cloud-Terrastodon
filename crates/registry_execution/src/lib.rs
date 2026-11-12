//! Invocation, explicit execution and simulation for registered operations.
//!
//! Context-free reflection, metadata and pure registrations belong to
//! `cloud_terrastodon_registry`; consumers import them from that owner.

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
mod runtime_test_case_report;
mod runtime_test_catalogue;
mod runtime_test_observation;
mod runtime_test_output_coverage;
mod runtime_test_registration;
mod runtime_test_report;
mod runtime_test_result;

#[cfg(test)]
mod discovery_plan_tests;
#[cfg(test)]
mod discovery_simulation_tests;
#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod runtime_test_tests;
#[cfg(test)]
mod value_catalog_tests;

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
pub use function_executor::AsyncExecutionFn;
pub use function_executor::AsyncValueFn;
pub use function_executor::FunctionExecutor;
pub use function_invocation::FunctionInvocation;
pub use function_invocation::InvocationFuture;
pub use function_invocation::invoke_executable_request;
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
pub use runtime_test_case_report::RuntimeTestCaseReport;
pub use runtime_test_catalogue::runtime_tests_matching;
pub use runtime_test_observation::RuntimeTestObservation;
pub use runtime_test_output_coverage::RuntimeTestOutputCoverage;
pub use runtime_test_registration::RUNTIME_TEST_REGISTRATIONS;
pub use runtime_test_registration::RuntimeTestRegistration;
pub use runtime_test_report::RuntimeTestReport;
pub use runtime_test_result::RuntimeTestResult;
pub use teamy_cancellation::CancellationScope;
