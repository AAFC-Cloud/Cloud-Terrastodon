use crate::Function;
use crate::RuntimeTestObservation;
use crate::RuntimeTestOutputCoverage;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_registry::Effect;
use cloud_terrastodon_registry::RegistrationSite;
use eyre::Context;
use facet::Facet;
use linkme::distributed_slice;

/// An operator-invoked verification of a registered request's execution and decoding.
///
/// This catalogue is independent of discovery: verification is an operation's
/// purpose, not an effect or an invitation to discover inputs automatically.
/// Inputs are explicitly supplied and decoded to the owning request type.
pub struct RuntimeTestRegistration {
    pub name: &'static str,
    pub request: Function,
    pub registration_site: RegistrationSite,
    pub execute: for<'a> fn(
        &'a str,
        &'a CloudTerrastodonExecutor,
        bool,
    ) -> ExecutionFuture<'a, RuntimeTestObservation>,
}

impl RuntimeTestRegistration {
    /// Admission requires every declared effect, not merely one matching effect.
    pub fn effects_allowed(&self, allowed: &[Effect]) -> bool {
        self.request
            .effects
            .iter()
            .all(|effect| allowed.contains(effect))
    }

    #[doc(hidden)]
    pub fn execute_cached<'a, R>(
        input: &'a str,
        executor: &'a CloudTerrastodonExecutor,
        clean: bool,
    ) -> ExecutionFuture<'a, RuntimeTestObservation>
    where
        R: CacheInvalidatableRequest + Facet<'static> + 'static,
        R::Error: Into<eyre::Report> + From<eyre::Report> + 'static,
        R::Output: Facet<'static> + 'static,
    {
        Box::pin(async move {
            let request: R = facet_json::from_str(input)
                .wrap_err("runtime test input does not satisfy the request contract")?;
            let mut observation = RuntimeTestObservation::default();
            let output = request
                .execute(executor)
                .with_invalidation(clean)
                .observe(|event| observation.record(event))
                .await
                .map_err(Into::into)?;
            observation.output_coverage = RuntimeTestOutputCoverage::observe(&output);
            Ok(observation)
        })
    }

    #[doc(hidden)]
    pub fn execute_uncached<'a, R>(
        input: &'a str,
        executor: &'a CloudTerrastodonExecutor,
        _clean: bool,
    ) -> ExecutionFuture<'a, RuntimeTestObservation>
    where
        R: ExecutableRequest + Facet<'static> + 'static,
        R::Error: Into<eyre::Report> + 'static,
        R::Output: Facet<'static> + 'static,
    {
        Box::pin(async move {
            let request: R = facet_json::from_str(input)
                .wrap_err("runtime test input does not satisfy the request contract")?;
            let mut observation = RuntimeTestObservation::default();
            let output = request
                .execute(executor)
                .observe(|event| observation.record(event))
                .await
                .map_err(Into::into)?;
            observation.output_coverage = RuntimeTestOutputCoverage::observe(&output);
            Ok(observation)
        })
    }
}

/// Explicit verification registrations; enumerating these never constructs input
/// or resolves credentials, configuration, service scope, or cached data.
#[distributed_slice]
pub static RUNTIME_TEST_REGISTRATIONS: [RuntimeTestRegistration];

/// Register a request's operator-invoked execution/decoding verification.
/// Cached requests delegate refresh to their existing invalidation contract.
#[macro_export]
macro_rules! register_runtime_test {
    ($request:ty => $output:ty, name = $name:literal, effects = [$($effect:ident),+ $(,)?], $reuse:ident) => {
        const _: () = {
            // Verification-only requests retain a direct executable descriptor
            // without entering ordinary value discovery's operation catalogue.
            static EXECUTION: $crate::RuntimeFunction = $crate::RuntimeFunction::async_execution(
                <$request as facet::Facet<'static>>::SHAPE,
                <$output as facet::Facet<'static>>::SHAPE,
                cloud_terrastodon_registry::FunctionKind::AsyncInvoke,
                "verify",
                cloud_terrastodon_registry::FunctionTraitContract::ExecutableRequest,
                &[$(cloud_terrastodon_registry::Effect::$effect),+],
                $crate::invoke_executable_request::<$request, $output>,
                cloud_terrastodon_registry::runtime_from_boxed::<$output>,
                cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
            );
            impl $request {
                pub const REGISTERED_EXECUTABLE_REQUEST: $crate::Function = $crate::Function::Runtime(&EXECUTION);
            }
            $crate::register_runtime_test!($request, name = $name, $reuse);
        };
    };
    ($request:ty, name = $name:literal, cached) => {
        const _: () = {
            #[$crate::distributed_slice($crate::RUNTIME_TEST_REGISTRATIONS)]
            static REGISTRATION: $crate::RuntimeTestRegistration = $crate::RuntimeTestRegistration {
                name: $name,
                request: <$request>::REGISTERED_EXECUTABLE_REQUEST,
                registration_site: cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
                execute: $crate::RuntimeTestRegistration::execute_cached::<$request>,
            };
        };
    };
    ($request:ty, name = $name:literal, uncached) => {
        const _: () = {
            #[$crate::distributed_slice($crate::RUNTIME_TEST_REGISTRATIONS)]
            static REGISTRATION: $crate::RuntimeTestRegistration = $crate::RuntimeTestRegistration {
                name: $name,
                request: <$request>::REGISTERED_EXECUTABLE_REQUEST,
                registration_site: cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
                execute: $crate::RuntimeTestRegistration::execute_uncached::<$request>,
            };
        };
    };
}
