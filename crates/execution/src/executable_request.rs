use crate::CloudTerrastodonExecutor;
use crate::ExecutionFuture;
use crate::RequestExecution;

/// Operation data bound to explicit resources only when execution is awaited.
///
/// Put serializable selections on the request, and keep cancellation, providers,
/// cache handles, and caller callbacks in the bound execution. Only memoized
/// results need `Sync + 'static`; an ordinary result may borrow request data.
pub trait ExecutableRequest: Sized + Send {
    type Output: Send;

    /// Execute with the correctly bound child scope and shared runtime resources.
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a;

    /// Equivalent to `executor.execute(self)`, including cancellation ownership.
    #[track_caller]
    fn execute<'a>(self, executor: &'a CloudTerrastodonExecutor) -> RequestExecution<'a, Self>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        executor.execute(self)
    }
}
