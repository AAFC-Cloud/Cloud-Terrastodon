use eyre::bail;
use tokio::task::JoinHandle;
use tracing::error;

use crate::app::MyApp;
use crate::app_message::AppMessage;
use crate::loadable::Loadable;
use crate::state_mutator::StateMutator;

pub struct Work<OnEnqueue, OnWork, WorkSuccess, WorkFailure, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut MyApp) -> (),
    OnWork: Future<Output = Result<WorkSuccess, WorkFailure>> + Send + 'static,
    OnWork::Output: Send + 'static,
    WorkSuccess: StateMutator + 'static,
    WorkFailure: std::fmt::Debug,
    OnFailure: Fn(WorkFailure) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator + 'static,
{
    pub on_enqueue: OnEnqueue,
    pub on_work: OnWork,
    pub on_failure: OnFailure,
}

pub struct WorkResult {}

impl<OnEnqueue, OnWork, WorkSuccess, WorkFailure, WorkFailureMutator, OnFailure>
    Work<OnEnqueue, OnWork, WorkSuccess, WorkFailure, WorkFailureMutator, OnFailure>
where
    OnEnqueue: Fn(&mut MyApp) -> (),
    OnWork: Future<Output = Result<WorkSuccess, WorkFailure>> + Send + 'static,
    OnWork::Output: Send + 'static,
    WorkSuccess: StateMutator + 'static,
    WorkFailure: std::fmt::Debug,
    OnFailure: Fn(WorkFailure) -> WorkFailureMutator + Send + 'static,
    WorkFailureMutator: StateMutator + 'static,
{
    pub fn enqueue(self, app: &mut MyApp) -> JoinHandle<()>
    where
        OnEnqueue: Fn(&mut MyApp) -> (),
        OnWork: Future<Output = Result<WorkSuccess, WorkFailure>> + Send + 'static,
        OnWork::Output: Send + 'static,
        WorkSuccess: StateMutator + 'static,
        WorkFailure: std::fmt::Debug,
        OnFailure: Fn(WorkFailure) -> WorkFailureMutator + Send + 'static,
        WorkFailureMutator: StateMutator + 'static,
    {
        let work = self;
        let runtime = tokio::runtime::Handle::current();
        let tx = app.tx.clone();
        let handle = runtime.spawn(async move {
            match work.on_work.await {
                Ok(result) => {
                    let msg = AppMessage::StateChange(Box::new(result));
                    if let Err(e) = tx.send(msg) {
                        panic!("Error sending message for work success: {:#?}", e);
                    }
                }
                Err(e) => {
                    error!("Error in work: {:#?}", e);
                    let state_mutator: WorkFailureMutator = (work.on_failure)(e);
                    let msg = AppMessage::StateChange(Box::new(state_mutator));
                    if let Err(e) = tx.send(msg) {
                        panic!("Error sending message for work failure: {:#?}", e);
                    }
                }
            }
        });
        (work.on_enqueue)(app);
        handle
    }
}
