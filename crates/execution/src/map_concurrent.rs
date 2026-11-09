use futures::StreamExt;
use futures::stream::FuturesUnordered;
use std::future::Future;
use std::future::IntoFuture;
use std::num::NonZeroUsize;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationScope;
use tokio::sync::Mutex;

/// Map a finite input collection with bounded concurrency and linked child scopes.
///
/// Results retain input order. Each factory receives its own scope and must use
/// it for the operation. Futures run inline and may borrow inputs; neither task
/// spawning nor `Clone`/`Send`/`'static` bounds are imposed by this function.
///
/// The first observed failure stops admission, cancels this group, and drains
/// active children before returning the original error. Parent cancellation
/// propagates through linked scopes; inherited deferrals remain effective.
/// To collect operation failures as data, return `Ok(operation_result)` from the
/// factory and inspect the inner results after the map has drained. These data
/// results are returned in input order; cancellation of the batch returns an
/// error instead of partial results, including any collected operation errors.
///
/// This owns coordination, not arbitrary caller drops: explicit drop, panic,
/// task abort, or an enclosing fail-fast combinator can still abandon the group.
#[track_caller]
pub(crate) fn map_concurrent<I, C, F, T, E>(
    cancellation: &CancellationScope,
    items: I,
    limit: NonZeroUsize,
    factory: C,
) -> impl Future<Output = Result<Vec<T>, E>> + use<I, C, F, T, E>
where
    I: IntoIterator,
    C: FnMut(CancellationScope, I::Item) -> F,
    F: IntoFuture<Output = Result<T, E>>,
    E: From<CancellationError>,
{
    let (_, future) = cancellation.with_async_child_scope(move |group| async move {
        let mut items = items.into_iter().enumerate();
        // Collection iterators report their exact length here. Use the lower
        // bound so filtered iterators do not reserve a speculative upper bound.
        let (minimum_len, _) = items.size_hint();
        // Serialize only lazy factory invocation, not the resulting work. This
        // supports FnMut without holding a guard across the operation's await,
        // and allows Send futures when the factory is Send but not Sync.
        let factory = Mutex::new(factory);
        let mut active = FuturesUnordered::new();
        let mut completed_indices = Vec::with_capacity(minimum_len);
        let mut completed = Vec::with_capacity(minimum_len);
        let mut first_error = None;
        let mut exhausted = false;
        loop {
            while first_error.is_none() && !exhausted && active.len() < limit.get() {
                if let Err(error) = group.bail_if_cancelled() {
                    first_error = Some(error.into());
                    break;
                }
                let Some((index, item)) = items.next() else {
                    exhausted = true;
                    break;
                };
                let branch = group.cancellable_scope(async |child| {
                    let work = {
                        let mut factory = factory.lock().await;
                        factory(child, item)
                    };
                    work.into_future().await
                });
                active.push(async move { (index, branch.await) });
            }

            let Some((index, result)) = active.next().await else {
                break;
            };
            match result {
                Ok(value) => {
                    completed_indices.push(index);
                    completed.push(value);
                }
                Err(error) if first_error.is_none() => {
                    first_error = Some(error);
                    group.request_cancel("a concurrent operation failed");
                }
                Err(_) => {}
            }
            // Ready-only inputs must also allow unrelated tasks to run and
            // request cancellation on a single-threaded runtime.
            tokio::task::yield_now().await;
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        // Successful completion makes the indices a permutation of 0..completed.len().
        // Swap both vectors together; each swap places a value at its final index.
        // This restores input order in O(n) without allocating another values buffer.
        for position in 0..completed_indices.len() {
            while completed_indices[position] != position {
                let destination = completed_indices[position];
                completed_indices.swap(position, destination);
                completed.swap(position, destination);
            }
        }
        Ok(completed)
    });
    future
}
