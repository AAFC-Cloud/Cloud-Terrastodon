use super::AcceptsWork;
use super::AppboundWorkFinishedMessage;
use super::AppboundWorkFinishedMessageSender;
use super::StateMutator;
use crate::CancellationReportExt;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::WrapErr;
use std::future::Future;
use std::marker::PhantomData;
use std::panic::Location;
use teamy_cancellation::CancellationScope;
use tracing::error;

/// Background work whose factory receives its own linked execution scope.
pub struct Work<State, OnEnqueue, OnWork, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut State),
    OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator<State> + 'static,
{
    pub location: &'static Location<'static>,
    pub on_enqueue: OnEnqueue,
    pub on_work: OnWork,
    pub on_failure: OnFailure,
    /// Treat a closed UI receiver as an error when delivering success.
    /// All work is drained regardless of this setting; required completion is
    /// expressed by deferrals inside the work's supplied cancellation scope.
    pub require_delivery: bool,
    pub description: String,
    pub _marker: PhantomData<State>,
}

impl<State, OnEnqueue, OnWork, WorkFuture, WorkSuccess, WorkFailureMutator, OnFailure>
    Work<State, OnEnqueue, OnWork, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut State),
    OnWork: FnOnce(CancellationScope) -> WorkFuture + Send + 'static,
    WorkFuture: Future<Output = eyre::Result<WorkSuccess>> + Send + 'static,
    WorkSuccess: StateMutator<State> + 'static,
    OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator<State> + 'static,
{
    #[track_caller]
    pub fn enqueue(
        self,
        work_acceptor: &impl AcceptsWork<State>,
        state: &mut State,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        let runtime = work_acceptor.runtime();
        let tx = work_acceptor.work_finished_message_sender();
        work_acceptor.work_tracker().spawn(
            cancellation,
            &runtime,
            self.description,
            async move |scope| {
                match (self.on_work)(scope).await {
                    Ok(result) => {
                        let msg =
                            AppboundWorkFinishedMessage::<State>::StateChange(Box::new(result));
                        if let Err(error) = tx.send(msg)
                            && self.require_delivery
                        {
                            return Err(error)
                                .wrap_err("Failed to deliver background work success");
                        }
                    }
                    Err(error) => {
                        if error.is_cancellation() {
                            return Err(error);
                        }
                        let error = error
                            .wrap_err(format!(
                                "Work location: {}",
                                RelativeLocation::from(self.location)
                            ))
                            .wrap_err("Error encountered in worker thread");
                        error!(?error);
                        let state_mutator = (self.on_failure)(error);
                        tx.send(AppboundWorkFinishedMessage::<State>::StateChange(Box::new(
                            state_mutator,
                        )))
                        .wrap_err("Failed to deliver background work failure")?;
                    }
                }
                Ok(())
            },
        )?;
        (self.on_enqueue)(state);
        Ok(())
    }
}
