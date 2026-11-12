use crate::CloudTerrastodonExecutor;
use crate::ExecutionFuture;
use crate::RequestExecution;
use teamy_cancellation::CancellationError;

/// Operation data bound to explicit resources only when execution is awaited.
///
/// Put serializable selections on the request, and keep cancellation, providers,
/// cache handles, and caller callbacks in the bound execution. Only memoized
/// results need `Sync + 'static`; an ordinary result may borrow request data.
pub trait ExecutableRequest: Sized + Send {
    type Output: Send;
    /// Preserve operation-specific failures, including any recoverable result data.
    /// The execution boundary converts scope cancellation into this type.
    type Error: Send + From<CancellationError>;

    /// Admit this request's authentication selection and retain its credential
    /// session through response handling and affected-cache invalidation.
    ///
    /// This runs on the request's child executor before invalidation or `run`.
    /// It does not change the caller's executor or choose a current account:
    /// the serializable request supplies the selection. The returned executor
    /// holds a lease so retirement can reject new work while already admitted
    /// required-completion work finishes. Token acquisition happens later at
    /// the transport boundary. Composite operations may additionally acquire
    /// leases for their child requests; unauthenticated operations use this
    /// default implementation.
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        Ok(executor)
    }

    /// Execute with the correctly bound child scope and shared runtime resources.
    fn run<'a>(
        self,
        executor: CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Self::Output, Self::Error>
    where
        Self: 'a,
        Self::Output: 'a,
        Self::Error: 'a;

    /// Equivalent to `executor.execute(self)`, including cancellation ownership.
    #[track_caller]
    fn execute<'a>(self, executor: &'a CloudTerrastodonExecutor) -> RequestExecution<'a, Self>
    where
        Self: 'a,
        Self::Output: 'a,
        Self::Error: 'a,
    {
        executor.execute(self)
    }
}
