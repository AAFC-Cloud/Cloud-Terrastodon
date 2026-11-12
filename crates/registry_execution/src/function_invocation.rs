use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::any::Any;
use std::any::type_name;

/// Reflection erases request-specific failures into reports at this boundary.
/// Direct `ExecutableRequest` execution retains its associated error type.
pub type InvocationFuture<'a> = ExecutionFuture<'a, Box<dyn Any + Send>>;

/// An erased result or pending operation that cannot outlive its bound executor.
pub enum FunctionInvocation<'a> {
    Pending(InvocationFuture<'a>),
    Ready(Box<dyn Any + Send>),
}

/// Invoke a data-only request through the same scoped path as direct execution.
///
/// Reflection owns its boxed input and output, so those values are static. This
/// boundary does not impose `Sync` or static resource lifetimes on execution.
/// Its error conversion is required only for registry invocation; the request's
/// `Into<eyre::Report>` implementation owns preserving domain context and sources.
#[track_caller]
pub fn invoke_executable_request<'a, R, O>(
    input: Box<dyn Any + Send>,
    executor: &'a CloudTerrastodonExecutor,
) -> InvocationFuture<'a>
where
    R: ExecutableRequest<Output = O> + Any + Send + 'static,
    R::Error: Into<eyre::Report> + 'static,
    O: Facet<'static> + Any + Send + 'static,
{
    match input.downcast::<R>() {
        Ok(request) => {
            let execution = executor.execute(*request);
            Box::pin(async move {
                let output = execution.await.map_err(Into::into)?;
                Ok(Box::new(output) as Box<dyn Any + Send>)
            })
        }
        Err(_) => {
            let error = eyre::eyre!(
                "invocation expected input type {}, but received a different boxed value",
                type_name::<R>()
            );
            Box::pin(async move { Err(error) })
        }
    }
}
