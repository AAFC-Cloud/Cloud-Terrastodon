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

#[derive(Default)]
pub struct FieldUpdaterWorkBuilder<T, E, Getter, OnEnqueue, OnWork>
where
    Getter: Fn(&mut MyApp) -> &mut Loadable<T, E>,
{
    pub getter: Option<Getter>,
    pub on_enqueue: Option<OnEnqueue>,
}
impl<T, E, Getter, OnEnqueue, OnWork> FieldUpdaterWorkBuilder<T, E, Getter, OnEnqueue, OnWork>
where
    Getter: Fn(&mut MyApp) -> &mut Loadable<T, E>,
    OnWork: Future<Output = Result<T, E>> + Send + 'static,
    OnWork::Output: Send + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn field(&mut self, getter: Getter) -> &mut Self {
        self.getter = Some(getter);
        self
    }
    pub fn build(
        self,
    ) -> eyre::Result<
        Work<_, _, FieldUpdaterWorkSuccessMutator<T>, _, _, FieldUpdaterWorkFailureMutator<E>>,
    > {
        let Some(getter) = self.getter else {
            bail!("Getter was not set!");
        };
        let work = Work {
            on_enqueue: |app| {
                let field = getter(app);
                *field = Loadable::Loading;
            },
            on_work: async { Ok(()) },
            on_failure: |e| {},
        };
        Ok(work)
    }
}

#[derive(Debug)]
struct FieldUpdaterWorkSuccessMutator<T>(T);
impl<T: std::fmt::Debug + Send + Sync> StateMutator for FieldUpdaterWorkSuccessMutator<T> {
    fn mutate_state(self: Box<Self>, state: &mut MyApp) {
        state.subscriptions = Loadable::Loaded(self.0.into_iter().map(|x| (false, x)).collect());
    }
}

#[derive(Debug)]
struct FieldUpdaterWorkFailureMutator<E>(E);
impl<E: std::fmt::Debug + Send + Sync> StateMutator for FieldUpdaterWorkFailureMutator<E> {
    fn mutate_state(self: Box<Self>, state: &mut MyApp) {
        state.subscriptions = Loadable::Failed(self.0)
    }
}
