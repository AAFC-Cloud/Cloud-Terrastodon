use cloud_terrastodon_registry::InvocationFuture;
use std::any::Any;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct InvocationId(u64);

impl InvocationId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

pub(crate) enum InvocationHostPoll {
    Pending,
    Ready(Box<dyn Any + Send>),
    Failed(String),
    Cancelled,
}

/// Owns execution mechanics while InvocationController owns engine metadata.
///
/// Implementations may spawn, hold, or deterministically fake futures. They
/// never receive Arena, RuntimeValue, Facet Peek, or borrow-lease ownership.
pub(crate) trait InvocationHost {
    fn start(&mut self, id: InvocationId, future: InvocationFuture);
    fn is_ready(&self, id: InvocationId) -> bool;
    fn poll(&mut self, id: InvocationId) -> InvocationHostPoll;
    /// Synchronously destroy this invocation's future and any unclaimed output
    /// before returning. Scheduling an asynchronous abort is insufficient.
    fn cancel(&mut self, id: InvocationId) -> bool;
    /// Synchronously destroy every held future and unclaimed output. Once this
    /// returns, the host owns no invocation data that could borrow arena values.
    fn shutdown(&mut self);
}

#[cfg(test)]
pub(crate) struct FakeInvocationHost {
    jobs: std::collections::BTreeMap<InvocationId, FakeJob>,
}

#[cfg(test)]
enum FakeJob {
    Held(InvocationFuture),
    Ready(Box<dyn Any + Send>),
    Failed(String),
    Cancelled,
}

#[cfg(test)]
impl Default for FakeInvocationHost {
    fn default() -> Self {
        Self {
            jobs: std::collections::BTreeMap::new(),
        }
    }
}

#[cfg(test)]
impl FakeInvocationHost {
    pub(crate) fn complete<T: Any + Send>(&mut self, id: InvocationId, value: T) {
        let Some(FakeJob::Held(future)) = self.jobs.remove(&id) else {
            panic!("invocation {id:?} is not held");
        };
        drop(future);
        self.jobs.insert(id, FakeJob::Ready(Box::new(value)));
    }

    pub(crate) fn fail(&mut self, id: InvocationId, message: impl Into<String>) {
        let Some(FakeJob::Held(future)) = self.jobs.remove(&id) else {
            panic!("invocation {id:?} is not held");
        };
        drop(future);
        self.jobs.insert(id, FakeJob::Failed(message.into()));
    }

    pub(crate) fn finish_cancelled(&mut self, id: InvocationId) {
        let Some(FakeJob::Held(future)) = self.jobs.remove(&id) else {
            panic!("invocation {id:?} is not held");
        };
        drop(future);
        self.jobs.insert(id, FakeJob::Cancelled);
    }

    pub(crate) fn contains(&self, id: InvocationId) -> bool {
        self.jobs.contains_key(&id)
    }
}

#[cfg(test)]
impl InvocationHost for FakeInvocationHost {
    fn start(&mut self, id: InvocationId, future: InvocationFuture) {
        assert!(self.jobs.insert(id, FakeJob::Held(future)).is_none());
    }

    fn is_ready(&self, id: InvocationId) -> bool {
        matches!(
            self.jobs.get(&id),
            Some(FakeJob::Ready(_) | FakeJob::Failed(_) | FakeJob::Cancelled)
        )
    }

    fn poll(&mut self, id: InvocationId) -> InvocationHostPoll {
        match self.jobs.get(&id) {
            Some(FakeJob::Held(_)) => InvocationHostPoll::Pending,
            Some(FakeJob::Ready(_)) => {
                let Some(FakeJob::Ready(value)) = self.jobs.remove(&id) else {
                    unreachable!()
                };
                InvocationHostPoll::Ready(value)
            }
            Some(FakeJob::Failed(_)) => {
                let Some(FakeJob::Failed(message)) = self.jobs.remove(&id) else {
                    unreachable!()
                };
                InvocationHostPoll::Failed(message)
            }
            Some(FakeJob::Cancelled) => {
                self.jobs.remove(&id);
                InvocationHostPoll::Cancelled
            }
            None => InvocationHostPoll::Cancelled,
        }
    }

    fn cancel(&mut self, id: InvocationId) -> bool {
        self.jobs.remove(&id).is_some()
    }

    fn shutdown(&mut self) {
        self.jobs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;

    struct DropFlag(Arc<AtomicBool>);

    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn fake_shutdown_destroys_held_futures_and_completed_outputs() {
        let future_dropped = Arc::new(AtomicBool::new(false));
        let output_dropped = Arc::new(AtomicBool::new(false));
        let future_value = DropFlag(future_dropped.clone());
        let mut host = FakeInvocationHost::default();
        host.start(
            InvocationId::new(1),
            Box::pin(async move {
                let _value = future_value;
                std::future::pending().await
            }),
        );
        host.start(InvocationId::new(2), Box::pin(std::future::pending()));
        host.complete(InvocationId::new(2), DropFlag(output_dropped.clone()));
        host.shutdown();
        assert!(future_dropped.load(Ordering::SeqCst));
        assert!(output_dropped.load(Ordering::SeqCst));
        assert!(!host.contains(InvocationId::new(1)));
        assert!(!host.contains(InvocationId::new(2)));
    }
}
