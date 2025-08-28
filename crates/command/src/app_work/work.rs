use super::AppboundWorkFinishedMessage;
use super::StateMutator;
use crate::app_work::AppWorkTracker;
use eyre::eyre;
use std::panic::Location;
use tokio::task::JoinHandle;
use tracing::error;

pub struct Work<State, OnEnqueue, OnWork, WorkSuccess, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut State),
    OnWork: Future<Output = eyre::Result<WorkSuccess>> + Send + 'static,
    OnWork::Output: Send + 'static,
    WorkSuccess: StateMutator<State> + 'static,
    OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator<State> + 'static,
{
    /// The location in code where this work was created.
    pub location: &'static Location<'static>,
    /// The function to call when the work is enqueued.
    pub on_enqueue: OnEnqueue,
    /// The work itself that produces some result
    pub on_work: OnWork,
    /// The function to call when the work fails.
    pub on_failure: OnFailure,
    /// If true, the work should be ran to completion before the app exits
    pub is_err_if_discarded: bool,
    /// A description of the work, for logging and debugging purposes. Similar to a thread name.
    pub description: String,
    pub _marker: std::marker::PhantomData<State>,
}

#[derive(Debug)]
pub struct WorkHandle {
    /// The JoinHandle for the work task.
    pub join_handle: JoinHandle<eyre::Result<()>>,
    /// A description of the work, for logging and debugging purposes. Similar to a thread name.
    pub description: String,
    /// If true, the app should run this work to completion before exiting.
    pub is_err_if_discarded: bool,
}

pub trait AcceptsWork<State>: Sized {
    fn work_tracker(&self) -> AppWorkTracker<State>;
    fn work_finished_message_sender(&self) -> impl AppboundWorkFinishedMessageSender<State>;
    fn runtime(&self) -> tokio::runtime::Handle {
        tokio::runtime::Handle::current()
    }
}

pub trait AppboundWorkFinishedMessageSender<State>: Sized + Send + 'static {
    fn send(&self, msg: AppboundWorkFinishedMessage<State>) -> eyre::Result<()>;
}

pub struct UnboundedWorkFinishedMessageSender<State> {
    sender: tokio::sync::mpsc::UnboundedSender<AppboundWorkFinishedMessage<State>>,
}
impl<T> UnboundedWorkFinishedMessageSender<T> {
    pub fn new(
        sender: tokio::sync::mpsc::UnboundedSender<AppboundWorkFinishedMessage<T>>,
    ) -> UnboundedWorkFinishedMessageSender<T> {
        UnboundedWorkFinishedMessageSender { sender }
    }
}
impl<State: 'static> AppboundWorkFinishedMessageSender<State>
    for UnboundedWorkFinishedMessageSender<State>
{
    fn send(&self, msg: AppboundWorkFinishedMessage<State>) -> eyre::Result<()> {
        self.sender
            .send(msg)
            .map_err(|e| eyre!("Error sending work message: {:#?}", e))
    }
}

impl<State, OnEnqueue, OnWork, WorkSuccess, WorkFailureMutator, OnFailure>
    Work<State, OnEnqueue, OnWork, WorkSuccess, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut State),
    OnWork: Future<Output = eyre::Result<WorkSuccess>> + Send + 'static,
    OnWork::Output: Send + 'static,
    WorkSuccess: StateMutator<State> + 'static,
    OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator<State> + 'static,
{
    pub fn enqueue(
        self,
        work_acceptor: &impl AcceptsWork<State>,
        state: &mut State,
    ) -> eyre::Result<()>
    where
        OnEnqueue: Fn(&mut State),
        OnWork: Future<Output = eyre::Result<WorkSuccess>> + Send + 'static,
        OnWork::Output: Send + 'static,
        WorkSuccess: StateMutator<State> + 'static,
        OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
        WorkFailureMutator: StateMutator<State> + 'static,
    {
        let work = self;
        let runtime = work_acceptor.runtime();
        let tx = work_acceptor.work_finished_message_sender();
        let description = work.description;
        let join_handle = runtime.spawn(async move {
            match work.on_work.await {
                Ok(result) => {
                    let msg = AppboundWorkFinishedMessage::<State>::StateChange(Box::new(result));
                    if let Err(e) = tx.send(msg)
                        && work.is_err_if_discarded
                    {
                        return Err(eyre!("Error sending message for work success: {}", e));
                    }
                }
                Err(error) => {
                    let error = error
                        .wrap_err(format!("Work location: {}", work.location))
                        .wrap_err("Error encountered in worker thread");
                    error!("{:?}", error);

                    let state_mutator: WorkFailureMutator = (work.on_failure)(error);
                    let msg =
                        AppboundWorkFinishedMessage::<State>::StateChange(Box::new(state_mutator));
                    if let Err(e) = tx.send(msg) {
                        return Err(eyre!("Error sending message for work failure: {:#?}", e));
                    }
                }
            }
            Ok(())
        });
        (work.on_enqueue)(state);
        work_acceptor.work_tracker().track(WorkHandle {
            description,
            join_handle,
            is_err_if_discarded: work.is_err_if_discarded,
        })?;
        Ok(())
    }
}
