use std::future::Future;
use std::future::IntoFuture;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationScope;

macro_rules! define_scoped_join {
    (
        $(#[$attribute:meta])*
        $name:ident,
        $(($branch:ident, $factory:ident, $future:ident, $output:ident, $index:literal)),+
    ) => {
        $(#[$attribute])*
        #[track_caller]
        pub fn $name<$($factory, $future, $output,)+ E>(
            cancellation: &CancellationScope,
            $($branch: $factory,)+
        ) -> impl Future<Output = Result<($($output,)+), E>>
            + use<$($factory, $future, $output,)+ E>
        where
            $($factory: FnOnce(CancellationScope) -> $future,
            $future: IntoFuture<Output = Result<$output, E>>,)+
            E: From<CancellationError>,
        {
            let (_, future) = cancellation.with_async_child_scope(move |group| async move {
                // Each branch keeps its own result. Only its index is shared,
                // so preserving the first error needs neither cloning nor
                // requiring the caller's error type to be Sync.
                let first_error = AtomicUsize::new(usize::MAX);
                $(let $branch = group.cancellable_scope($branch);)+
                let ($($branch,)+) = tokio::join!(
                    $(async {
                        let result = $branch.await;
                        if result.is_err()
                            && first_error.compare_exchange(
                                usize::MAX,
                                $index,
                                Ordering::Relaxed,
                                Ordering::Relaxed,
                            ).is_ok()
                        {
                            group.request_cancel("a concurrent operation failed");
                        }
                        result
                    },)+
                );

                // Visit the recorded failure before collecting the tuple.
                // A cancellation error from another branch must not replace it.
                let first_error = first_error.load(Ordering::Relaxed);
                $(let $branch = if first_error == $index {
                    Ok($branch?)
                } else {
                    $branch
                };)+
                Ok(($($branch?,)+))
            });
            future
        }
    };
}

define_scoped_join! {
    /// Run two operations concurrently, cancelling and draining siblings on error.
    ///
    /// Each factory receives its own linked child scope and must use that scope
    /// for its work. The first observed error requests cancellation for this
    /// group, leaving the parent and unrelated work unaffected. Unprotected
    /// siblings stop; deferred work finishes before the original error returns.
    /// The results retain argument order when every operation succeeds.
    ///
    /// Parent cancellation also reaches every branch. Required descendants keep
    /// their owning automatic scopes polling while the cooperative group drains.
    /// Callers must retain this future: explicit drops, task aborts, panics, and
    /// enclosing fail-fast combinators can still abandon required completion.
    /// The entire group is deliberately not deferred.
    ///
    /// The futures are polled inline, without spawning tasks. They may borrow
    /// inputs and return different types; errors need not implement Clone.
    ///
    /// ```
    /// use cloud_terrastodon_execution::join2;
    /// use teamy_cancellation::CancellationError;
    /// use teamy_cancellation::CancellationToken;
    ///
    /// # async fn example() -> Result<(), CancellationError> {
    /// let label = String::from("example");
    /// let borrowed_label = label.as_str();
    /// let pair = CancellationToken::new()
    ///     .with_async_scope(|scope| async move {
    ///         join2(
    ///             &scope,
    ///             |_child| async { Ok::<_, CancellationError>(borrowed_label) },
    ///             |_child| async { Ok::<_, CancellationError>(42_u32) },
    ///         )
    ///         .await
    ///     })
    ///     .await?;
    /// assert_eq!(pair, ("example", 42));
    /// # Ok(())
    /// # }
    /// ```
    join2,
    (a, A, FutureA, OutputA, 0),
    (b, B, FutureB, OutputB, 1)
}

define_scoped_join! {
    /// Run three operations concurrently, cancelling and draining siblings on error.
    ///
    /// This has the same scope ownership, first-error, borrowing, and required
    /// completion guarantees and limitations as [`join2`]. Each factory must
    /// use its supplied child scope, and callers must retain the returned future
    /// until it finishes draining.
    join3,
    (a, A, FutureA, OutputA, 0),
    (b, B, FutureB, OutputB, 1),
    (c, C, FutureC, OutputC, 2)
}

define_scoped_join! {
    /// Run four operations under [`join2`]'s cancellation and draining contract.
    join4,
    (a, A, FutureA, OutputA, 0),
    (b, B, FutureB, OutputB, 1),
    (c, C, FutureC, OutputC, 2),
    (d, D, FutureD, OutputD, 3)
}

define_scoped_join! {
    /// Run five operations under [`join2`]'s cancellation and draining contract.
    join5,
    (a, A, FutureA, OutputA, 0),
    (b, B, FutureB, OutputB, 1),
    (c, C, FutureC, OutputC, 2),
    (d, D, FutureD, OutputD, 3),
    (e, F, FutureE, OutputE, 4)
}
