use super::invocation_host::InvocationHost;
use super::invocation_host::InvocationHostPoll;
use super::invocation_host::InvocationId;
use cloud_terrastodon_registry::InvocationFuture;
use std::any::Any;
use std::collections::BTreeMap;
use std::future::poll_fn;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::task::Context;
use std::task::Poll;
use tokio::task::JoinHandle;

/// Tokio production mechanism with context attachment supplied by the UI host.
///
/// The context attachment function must wrap its input without detaching it or
/// transferring its ownership anywhere except the returned future.
pub(crate) struct TokioInvocationHost {
    attach: fn(InvocationFuture) -> InvocationFuture,
    jobs: BTreeMap<InvocationId, InvocationJob>,
}

struct InvocationJob {
    state: Arc<Mutex<JobState>>,
    task: Option<JoinHandle<()>>,
}

/// All borrow-carrying ownership stays here, never in a task's return value.
/// User polling, attachment, formatting, and destruction run while locked.
struct JobState {
    future: Option<InvocationFuture>,
    outcome: Option<InvocationHostPoll>,
}

impl InvocationJob {
    fn new(future: InvocationFuture) -> Self {
        Self {
            state: Arc::new(Mutex::new(JobState {
                future: Some(future),
                outcome: None,
            })),
            task: None,
        }
    }

    fn cancel(&mut self) {
        // Abort alone only schedules asynchronous destruction. Remove and
        // destroy both the future and any unclaimed output before returning.
        lock_state(&self.state).clear();
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

impl Drop for InvocationJob {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn lock_state(state: &Mutex<JobState>) -> MutexGuard<'_, JobState> {
    // User panics are caught before releasing the guard. Recovering a poison
    // still lets shutdown dispose ownership if an internal panic ever occurs.
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Consume even a custom panic payload while the caller holds the state lock.
/// A payload's destructor can itself panic; its replacement payload must not
/// escape the lock either. Only an independently owned message is returned.
fn consume_panic(mut payload: Box<dyn Any + Send>) -> String {
    let message = if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else {
        "non-string panic payload".to_owned()
    };
    loop {
        match catch_unwind(AssertUnwindSafe(|| drop(payload))) {
            Ok(()) => return message,
            Err(next) => payload = next,
        }
    }
}

/// Drop user-owned state without allowing its panic payload to outlive the
/// lock. A destructor panic is reported as an owned diagnostic when useful.
fn dispose<T>(value: T) -> Option<String> {
    catch_unwind(AssertUnwindSafe(|| drop(value)))
        .err()
        .map(consume_panic)
}

impl JobState {
    fn attach(&mut self, attach: fn(InvocationFuture) -> InvocationFuture) {
        let future = self.future.take().expect("new job has its future");
        match catch_unwind(AssertUnwindSafe(|| attach(future))) {
            Ok(future) => self.future = Some(future),
            Err(payload) => {
                let message = consume_panic(payload);
                self.fail(format!("invocation context attachment panicked: {message}"));
            }
        }
    }

    fn poll(&mut self, context: &mut Context<'_>) -> Poll<()> {
        let Some(future) = self.future.as_mut() else {
            return Poll::Ready(());
        };
        match catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(context))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(result)) => {
                // Store completed ownership before dropping the future: if
                // that destructor panics, fail() will also destroy the output.
                self.outcome = Some(match result {
                    Ok(output) => InvocationHostPoll::Ready(output),
                    Err(error) => {
                        let message = match catch_unwind(AssertUnwindSafe(|| error.to_string())) {
                            Ok(message) => message,
                            Err(payload) => format!(
                                "invocation error formatting panicked: {}",
                                consume_panic(payload)
                            ),
                        };
                        let drop_panic = dispose(error);
                        InvocationHostPoll::Failed(drop_panic.map_or(message, |panic| {
                            format!("invocation error destruction panicked: {panic}")
                        }))
                    }
                });
                if let Some(message) = dispose(self.future.take()) {
                    self.fail(format!("invocation future destruction panicked: {message}"));
                }
                Poll::Ready(())
            }
            Err(payload) => {
                let message = consume_panic(payload);
                self.fail(format!("invocation panicked: {message}"));
                Poll::Ready(())
            }
        }
    }

    fn fail(&mut self, message: String) {
        self.clear();
        self.outcome = Some(InvocationHostPoll::Failed(message));
    }

    fn clear(&mut self) {
        dispose(self.future.take());
        dispose(self.outcome.take());
    }
}

impl TokioInvocationHost {
    pub(crate) fn new(attach: fn(InvocationFuture) -> InvocationFuture) -> Self {
        Self {
            attach,
            jobs: BTreeMap::new(),
        }
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.jobs.len()
    }
}

impl InvocationHost for TokioInvocationHost {
    fn start(&mut self, id: InvocationId, future: InvocationFuture) {
        let mut job = InvocationJob::new(future);
        assert!(
            !self.jobs.contains_key(&id),
            "invocation identities are unique"
        );
        lock_state(&job.state).attach(self.attach);
        if lock_state(&job.state).future.is_some() {
            let state = job.state.clone();
            // The spawned runner owns only the shared state handle. Actual
            // polling and terminal storage remain serialized with cancellation.
            job.task = Some(tokio::spawn(poll_fn(move |context| {
                lock_state(&state).poll(context)
            })));
        }
        self.jobs.insert(id, job);
    }

    fn is_ready(&self, id: InvocationId) -> bool {
        self.jobs.get(&id).is_some_and(|job| {
            let state = lock_state(&job.state);
            state.outcome.is_some() || job.task.as_ref().is_some_and(JoinHandle::is_finished)
        })
    }

    fn poll(&mut self, id: InvocationId) -> InvocationHostPoll {
        let Some(job) = self.jobs.get(&id) else {
            return InvocationHostPoll::Cancelled;
        };
        let outcome = {
            let mut state = lock_state(&job.state);
            if let Some(outcome) = state.outcome.take() {
                outcome
            } else if job.task.as_ref().is_some_and(JoinHandle::is_finished) {
                // Runtime shutdown can discard the runner without polling it
                // again. Its raw future remains ours until this cleanup.
                state.clear();
                InvocationHostPoll::Cancelled
            } else {
                return InvocationHostPoll::Pending;
            }
        };
        self.jobs.remove(&id);
        outcome
    }

    fn cancel(&mut self, id: InvocationId) -> bool {
        let Some(mut job) = self.jobs.remove(&id) else {
            return false;
        };
        job.cancel();
        true
    }

    fn shutdown(&mut self) {
        self.jobs.clear();
    }
}

impl Drop for TokioInvocationHost {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::task::Context;
    use std::task::Poll;
    use std::time::Duration;

    struct DropFlag(Arc<AtomicBool>);

    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    fn flag() -> (DropFlag, Arc<AtomicBool>) {
        let observed = Arc::new(AtomicBool::new(false));
        (DropFlag(observed.clone()), observed)
    }

    fn attach_nothing(future: InvocationFuture) -> InvocationFuture {
        future
    }

    fn held_future(value: DropFlag) -> InvocationFuture {
        Box::pin(async move {
            let _value = value;
            std::future::pending().await
        })
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancel_before_first_poll_destroys_future_synchronously() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (value, dropped) = flag();
        let id = InvocationId::new(1);
        host.start(id, held_future(value));
        assert!(host.cancel(id));
        assert!(dropped.load(Ordering::SeqCst));
        assert!(!host.cancel(id));
        assert_eq!(host.pending_count(), 0);
        assert!(matches!(host.poll(id), InvocationHostPoll::Cancelled));
    }

    #[test]
    fn runtime_shutdown_is_reported_after_synchronously_destroying_held_data() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (value, dropped) = flag();
        let id = InvocationId::new(1);
        {
            let _entered = runtime.enter();
            host.start(id, held_future(value));
        }
        // No task was polled. Dropping the runtime destroys only its runner;
        // the host still owns the raw invocation future until polled/cancelled.
        drop(runtime);
        assert!(!dropped.load(Ordering::SeqCst));
        assert!(host.is_ready(id));
        assert!(matches!(host.poll(id), InvocationHostPoll::Cancelled));
        assert!(dropped.load(Ordering::SeqCst));
        assert_eq!(host.pending_count(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancel_pending_future_destroys_it_before_returning() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (value, dropped) = flag();
        let (polled, observed) = tokio::sync::oneshot::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(async move {
                let _value = value;
                polled.send(()).unwrap();
                std::future::pending().await
            }),
        );
        observed.await.unwrap();
        assert!(!host.is_ready(id));
        assert!(host.cancel(id));
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancel_destroys_completed_but_unclaimed_output() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (output, dropped) = flag();
        let (completed, observed) = tokio::sync::oneshot::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(async move {
                completed.send(()).unwrap();
                Ok(Box::new(output) as Box<dyn Any + Send>)
            }),
        );
        observed.await.unwrap();
        assert!(host.is_ready(id));
        assert!(!dropped.load(Ordering::SeqCst));
        assert!(host.cancel(id));
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn drop_destroys_never_polled_futures_synchronously() {
        let (first, first_dropped) = flag();
        let (second, second_dropped) = flag();
        let mut host = TokioInvocationHost::new(attach_nothing);
        host.start(InvocationId::new(1), held_future(first));
        host.start(InvocationId::new(2), held_future(second));
        drop(host);
        assert!(first_dropped.load(Ordering::SeqCst));
        assert!(second_dropped.load(Ordering::SeqCst));
    }

    struct BlockingPoll {
        entered: Option<std::sync::mpsc::Sender<()>>,
        release: std::sync::mpsc::Receiver<()>,
        finished: Arc<AtomicBool>,
        _value: DropFlag,
    }

    impl Future for BlockingPoll {
        type Output = eyre::Result<Box<dyn Any + Send>>;

        fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            self.entered.take().unwrap().send(()).unwrap();
            self.release.recv_timeout(Duration::from_secs(10)).unwrap();
            self.finished.store(true, Ordering::SeqCst);
            Poll::Pending
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancel_waits_for_an_active_poll_before_destroying_the_future() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (value, dropped) = flag();
        let finished = Arc::new(AtomicBool::new(false));
        let (entered, entered_rx) = std::sync::mpsc::channel();
        let (release, release_rx) = std::sync::mpsc::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(BlockingPoll {
                entered: Some(entered),
                release: release_rx,
                finished: finished.clone(),
                _value: value,
            }),
        );
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let (cancelling, cancelling_rx) = std::sync::mpsc::channel();
        let cancel_thread = std::thread::spawn(move || {
            cancelling.send(()).unwrap();
            assert!(host.cancel(id));
            assert!(finished.load(Ordering::SeqCst));
            assert!(dropped.load(Ordering::SeqCst));
        });
        cancelling_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        release.send(()).unwrap();
        cancel_thread.join().unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn success_transfers_output_and_destroys_future_before_readiness() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (future_value, future_dropped) = flag();
        let (output, output_dropped) = flag();
        let (completed, observed) = tokio::sync::oneshot::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(async move {
                let _value = future_value;
                completed.send(()).unwrap();
                Ok(Box::new(output) as Box<dyn Any + Send>)
            }),
        );
        observed.await.unwrap();
        assert!(host.is_ready(id));
        assert!(future_dropped.load(Ordering::SeqCst));
        let InvocationHostPoll::Ready(output) = host.poll(id) else {
            panic!("expected completed output");
        };
        assert!(!output_dropped.load(Ordering::SeqCst));
        drop(output);
        assert!(output_dropped.load(Ordering::SeqCst));
        assert_eq!(host.pending_count(), 0);
    }

    #[derive(Debug)]
    struct TrackedError(Arc<AtomicBool>);

    impl std::fmt::Display for TrackedError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("tracked invocation failure")
        }
    }

    impl std::error::Error for TrackedError {}

    impl Drop for TrackedError {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn failure_owns_only_a_message_after_disposing_the_error() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let error_dropped = Arc::new(AtomicBool::new(false));
        let error = TrackedError(error_dropped.clone());
        let (completed, observed) = tokio::sync::oneshot::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(async move {
                completed.send(()).unwrap();
                Err(eyre::Report::new(error))
            }),
        );
        observed.await.unwrap();
        assert!(host.is_ready(id));
        assert!(error_dropped.load(Ordering::SeqCst));
        let InvocationHostPoll::Failed(message) = host.poll(id) else {
            panic!("expected owned failure message");
        };
        assert!(message.contains("tracked invocation failure"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn poll_panic_destroys_payload_and_future_before_reporting_failure() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (future_value, future_dropped) = flag();
        let (payload, payload_dropped) = flag();
        let (polled, observed) = tokio::sync::oneshot::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(async move {
                let _value = future_value;
                polled.send(()).unwrap();
                std::panic::panic_any(payload);
            }),
        );
        observed.await.unwrap();
        assert!(host.is_ready(id));
        assert!(future_dropped.load(Ordering::SeqCst));
        assert!(payload_dropped.load(Ordering::SeqCst));
        assert!(matches!(host.poll(id), InvocationHostPoll::Failed(_)));
    }

    fn attach_panics_with_future(future: InvocationFuture) -> InvocationFuture {
        std::panic::panic_any(future);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn attach_panic_payload_cannot_keep_the_invocation_future_alive() {
        let mut host = TokioInvocationHost::new(attach_panics_with_future);
        let (future, dropped) = flag();
        let id = InvocationId::new(1);
        host.start(id, held_future(future));
        assert!(dropped.load(Ordering::SeqCst));
        assert!(host.is_ready(id));
        assert!(matches!(host.poll(id), InvocationHostPoll::Failed(_)));
    }

    struct PanickingDropFuture {
        _value: DropFlag,
        payload: Option<DropFlag>,
        output: Option<Box<dyn Any + Send>>,
        completed: Option<tokio::sync::oneshot::Sender<()>>,
    }

    impl Future for PanickingDropFuture {
        type Output = eyre::Result<Box<dyn Any + Send>>;

        fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            if let Some(completed) = self.completed.take() {
                completed.send(()).unwrap();
            }
            self.output
                .take()
                .map_or(Poll::Pending, |output| Poll::Ready(Ok(output)))
        }
    }

    impl Drop for PanickingDropFuture {
        fn drop(&mut self) {
            std::panic::panic_any(self.payload.take().unwrap());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancellation_disposes_a_future_destructor_panic_payload() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (value, future_dropped) = flag();
        let (payload, payload_dropped) = flag();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(PanickingDropFuture {
                _value: value,
                payload: Some(payload),
                output: None,
                completed: None,
            }),
        );
        assert!(host.cancel(id));
        assert!(future_dropped.load(Ordering::SeqCst));
        assert!(payload_dropped.load(Ordering::SeqCst));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn completed_output_is_disposed_if_future_destruction_panics() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (value, future_dropped) = flag();
        let (payload, payload_dropped) = flag();
        let (output, output_dropped) = flag();
        let (completed, observed) = tokio::sync::oneshot::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(PanickingDropFuture {
                _value: value,
                payload: Some(payload),
                output: Some(Box::new(output)),
                completed: Some(completed),
            }),
        );
        observed.await.unwrap();
        assert!(host.is_ready(id));
        assert!(future_dropped.load(Ordering::SeqCst));
        assert!(payload_dropped.load(Ordering::SeqCst));
        assert!(output_dropped.load(Ordering::SeqCst));
        assert!(matches!(host.poll(id), InvocationHostPoll::Failed(_)));
    }

    struct PanickingDropOutput {
        _value: DropFlag,
        payload: Option<DropFlag>,
    }

    impl Drop for PanickingDropOutput {
        fn drop(&mut self) {
            std::panic::panic_any(self.payload.take().unwrap());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_destroys_pending_and_ready_jobs_despite_output_drop_panic() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let (output_value, output_dropped) = flag();
        let (payload, payload_dropped) = flag();
        let (pending_value, pending_dropped) = flag();
        let (completed, observed) = tokio::sync::oneshot::channel();
        host.start(
            InvocationId::new(1),
            Box::pin(async move {
                completed.send(()).unwrap();
                Ok(Box::new(PanickingDropOutput {
                    _value: output_value,
                    payload: Some(payload),
                }) as Box<dyn Any + Send>)
            }),
        );
        observed.await.unwrap();
        host.start(InvocationId::new(2), held_future(pending_value));
        host.shutdown();
        assert!(output_dropped.load(Ordering::SeqCst));
        assert!(payload_dropped.load(Ordering::SeqCst));
        assert!(pending_dropped.load(Ordering::SeqCst));
        assert_eq!(host.pending_count(), 0);
        host.shutdown();
    }

    #[derive(Debug)]
    struct PanickingDisplayError {
        dropped: Arc<AtomicBool>,
        payload_dropped: Arc<AtomicBool>,
    }

    impl std::fmt::Display for PanickingDisplayError {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::panic::panic_any(DropFlag(self.payload_dropped.clone()));
        }
    }

    impl std::error::Error for PanickingDisplayError {}

    impl Drop for PanickingDisplayError {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn error_display_panic_does_not_escape_or_retain_the_error() {
        let mut host = TokioInvocationHost::new(attach_nothing);
        let dropped = Arc::new(AtomicBool::new(false));
        let payload_dropped = Arc::new(AtomicBool::new(false));
        let error = PanickingDisplayError {
            dropped: dropped.clone(),
            payload_dropped: payload_dropped.clone(),
        };
        let (completed, observed) = tokio::sync::oneshot::channel();
        let id = InvocationId::new(1);
        host.start(
            id,
            Box::pin(async move {
                completed.send(()).unwrap();
                Err(eyre::Report::new(error))
            }),
        );
        observed.await.unwrap();
        assert!(host.is_ready(id));
        assert!(dropped.load(Ordering::SeqCst));
        assert!(payload_dropped.load(Ordering::SeqCst));
        let InvocationHostPoll::Failed(message) = host.poll(id) else {
            panic!("expected owned failure message");
        };
        assert!(message.contains("error formatting panicked"));
    }
}
