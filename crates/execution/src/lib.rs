//! Explicit runtime resources and bounded execution for data-only requests.
//!
//! The application supplies a cache, authentication provider, and an already
//! owned cancellation scope. Request inspection and construction perform no I/O.
//! Credential implementations and production transport adapters live above this
//! crate; neither they nor registry invocation are dependencies here.

mod authentication_provider;
mod cloud_terrastodon_executor;
mod disabled_authentication_provider;
mod executable_request;
mod execution_future;
mod execution_mode;
mod request_execution;

pub use authentication_provider::AuthenticationProvider;
pub use cloud_terrastodon_executor::CloudTerrastodonExecutor;
pub use disabled_authentication_provider::DisabledAuthenticationProvider;
pub use executable_request::ExecutableRequest;
pub use execution_future::ExecutionFuture;
pub use request_execution::RequestExecution;
