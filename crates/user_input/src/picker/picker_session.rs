use std::future::Future;
use std::future::IntoFuture;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationScope;

/// Retain terminal restoration and handler draining without protecting reads.
///
/// The session owner and the terminal-completion deferral are siblings. Work
/// started from the supplied session scope therefore remains cancellable, while
/// the separate deferral keeps automatic ancestors polling through UI cleanup.
#[track_caller]
pub(super) fn run_picker_session<C, F, T, E>(
    cancellation: &CancellationScope,
    factory: C,
) -> impl Future<Output = Result<T, E>> + use<C, F, T, E>
where
    C: FnOnce(CancellationScope) -> F,
    F: IntoFuture<Output = Result<T, E>>,
    E: From<CancellationError>,
{
    // Construct the owner before the deferral; polling it inside the deferred
    // future must not make handler scopes descendants of terminal protection.
    let session = cancellation.with_async_scope(factory);
    cancellation.deferred(
        "restore picker terminal and drain its handlers",
        async move |_terminal| session.await,
    )
}
