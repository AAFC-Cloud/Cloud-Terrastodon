use crate::app::MyApp;
use crate::app_message::AppMessage;
use crate::state_mutator::StateMutator;
use eyre::eyre;
use std::panic::Location;
use tokio::task::JoinHandle;
use tracing::error;

pub struct Work<OnEnqueue, OnWork, WorkSuccess, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut MyApp),
    OnWork: Future<Output = eyre::Result<WorkSuccess>> + Send + 'static,
    OnWork::Output: Send + 'static,
    WorkSuccess: StateMutator + 'static,
    OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator + 'static,
{
    pub location: &'static Location<'static>,
    pub on_enqueue: OnEnqueue,
    pub on_work: OnWork,
    pub on_failure: OnFailure,
    pub is_err_if_discarded: bool,
    pub description: String,
}

#[derive(Debug)]
pub struct WorkHandle {
    pub join_handle: JoinHandle<eyre::Result<()>>,
    pub description: String,
    pub is_err_if_discarded: bool,
}

impl<OnEnqueue, OnWork, WorkSuccess, WorkFailureMutator, OnFailure>
    Work<OnEnqueue, OnWork, WorkSuccess, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut MyApp),
    OnWork: Future<Output = eyre::Result<WorkSuccess>> + Send + 'static,
    OnWork::Output: Send + 'static,
    WorkSuccess: StateMutator + 'static,
    OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator + 'static,
{
    pub fn enqueue(self, app: &mut MyApp)
    where
        OnEnqueue: Fn(&mut MyApp),
        OnWork: Future<Output = eyre::Result<WorkSuccess>> + Send + 'static,
        OnWork::Output: Send + 'static,
        WorkSuccess: StateMutator + 'static,
        OnFailure: Fn(eyre::Error) -> WorkFailureMutator + Send + 'static,
        WorkFailureMutator: StateMutator + 'static,
    {
        let work = self;
        let runtime = tokio::runtime::Handle::current();
        let tx = app.tx.clone();
        let description = work.description;
        let join_handle = runtime.spawn(async move {
            match work.on_work.await {
                Ok(result) => {
                    let msg = AppMessage::StateChange(Box::new(result));
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
                    let msg = AppMessage::StateChange(Box::new(state_mutator));
                    if let Err(e) = tx.send(msg) {
                        return Err(eyre!("Error sending message for work failure: {:#?}", e));
                    }
                }
            }
            Ok(())
        });
        (work.on_enqueue)(app);
        app.work_tracker.track(WorkHandle {
            description,
            join_handle,
            is_err_if_discarded: work.is_err_if_discarded,
        });
    }
}
