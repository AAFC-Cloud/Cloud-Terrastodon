use std::collections::BTreeMap;

use cloud_terrastodon_registry::InvocationFuture;
use futures::FutureExt;
use tokio::task::JoinHandle;

use super::invocation_host::InvocationHost;
use super::invocation_host::InvocationHostPoll;
use super::invocation_host::InvocationId;

/// Tokio production mechanism with context attachment supplied by the UI host.
pub(crate) struct TokioInvocationHost {
    attach: fn(InvocationFuture) -> InvocationFuture,
    jobs: BTreeMap<InvocationId, JoinHandle<eyre::Result<Box<dyn std::any::Any + Send>>>>,
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
        let task = tokio::spawn((self.attach)(future));
        assert!(
            self.jobs.insert(id, task).is_none(),
            "invocation identities are unique"
        );
    }

    fn is_ready(&self, id: InvocationId) -> bool {
        self.jobs.get(&id).is_some_and(JoinHandle::is_finished)
    }

    fn poll(&mut self, id: InvocationId) -> InvocationHostPoll {
        let Some(task) = self.jobs.get(&id) else {
            return InvocationHostPoll::Cancelled;
        };
        if !task.is_finished() {
            return InvocationHostPoll::Pending;
        }
        let task = self
            .jobs
            .remove(&id)
            .expect("finished task remains indexed");
        match task
            .now_or_never()
            .expect("a finished JoinHandle resolves immediately")
        {
            Ok(Ok(output)) => InvocationHostPoll::Ready(output),
            Ok(Err(error)) => InvocationHostPoll::Failed(error.to_string()),
            Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
            Err(error) if error.is_cancelled() => InvocationHostPoll::Cancelled,
            Err(error) => InvocationHostPoll::Failed(format!("invocation task failed: {error}")),
        }
    }

    fn cancel(&mut self, id: InvocationId) -> bool {
        let Some(task) = self.jobs.remove(&id) else {
            return false;
        };
        task.abort();
        true
    }
}

impl Drop for TokioInvocationHost {
    fn drop(&mut self) {
        for (_, task) in std::mem::take(&mut self.jobs) {
            task.abort();
        }
    }
}
