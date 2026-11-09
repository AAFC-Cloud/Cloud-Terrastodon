use super::AppWorkTracker;
use super::AppboundWorkFinishedMessage;
use super::StateMutator;
use crate::CancellationReportExt;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::eyre;
use std::future::Future;
use std::marker::PhantomData;
use std::panic::Location;
use teamy_cancellation::CancellationScope;
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;
use tracing::error;

/// Background work whose factory receives its own linked execution scope.
///
/// Successful results are UI snapshots, so a closed receiver may discard them.
/// Failures are logged and sent to the UI; failure-delivery errors are retained
/// by the task tracker. Required completion is determined by scope deferrals,
/// independently of whether the UI still needs the result.
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
    pub description: String,
    pub _marker: PhantomData<State>,
}

impl<State: 'static, OnEnqueue, OnWork, WorkFuture, WorkSuccess, WorkFailureMutator, OnFailure>
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
        tracker: &AppWorkTracker,
        sender: UnboundedSender<AppboundWorkFinishedMessage<State>>,
        state: &mut State,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        tracker.spawn(
            cancellation,
            &Handle::current(),
            self.description,
            async move |scope| {
                match (self.on_work)(scope).await {
                    Ok(result) => {
                        let msg =
                            AppboundWorkFinishedMessage::<State>::StateChange(Box::new(result));
                        if sender.send(msg).is_err() {
                            debug!(
                                "UI receiver closed before background work success delivery"
                            );
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
                        sender.send(AppboundWorkFinishedMessage::<State>::StateChange(Box::new(
                            state_mutator,
                        )))
                        .map_err(|_| eyre!("Cannot deliver background work failure because its UI receiver closed"))?;
                    }
                }
                Ok(())
            },
        )?;
        (self.on_enqueue)(state);
        Ok(())
    }
}
