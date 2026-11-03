use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::any::Any;
use std::any::type_name;
use std::future::IntoFuture;

pub type InvocationFuture<'a> = ExecutionFuture<'a, Box<dyn Any + Send>>;

/// An erased result or pending operation that cannot outlive its bound executor.
pub enum FunctionInvocation<'a> {
    Pending(InvocationFuture<'a>),
    Ready(Box<dyn Any + Send>),
}

pub fn invoke_result_future<T, O, E>(input: Box<dyn Any + Send>) -> InvocationFuture<'static>
where
    T: IntoFuture<Output = Result<O, E>> + Any + Send + 'static,
    T::IntoFuture: Send + 'static,
    E: Into<eyre::Error> + Send + 'static,
    O: Facet<'static> + Any + Send + 'static,
{
    Box::pin(async move {
        let request = input.downcast::<T>().map_err(|_| {
            eyre::eyre!(
                "invocation expected input type {}, but received a different boxed value",
                type_name::<T>()
            )
        })?;
        let output = (*request).into_future().await.map_err(Into::into)?;
        Ok(Box::new(output) as Box<dyn Any + Send>)
    })
}
/// Invoke a data-only request through the same scoped path as direct execution.
///
/// Reflection owns its boxed input and output, so those values are static. This
/// boundary does not impose `Sync` or static resource lifetimes on execution.
#[track_caller]
pub fn invoke_executable_request<'a, R, O>(
    input: Box<dyn Any + Send>,
    executor: &'a CloudTerrastodonExecutor,
) -> InvocationFuture<'a>
where
    R: ExecutableRequest<Output = O> + Any + Send + 'static,
    O: Facet<'static> + Any + Send + 'static,
{
    match input.downcast::<R>() {
        Ok(request) => {
            let execution = executor.execute(*request);
            Box::pin(async move {
                let output = execution.await?;
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
