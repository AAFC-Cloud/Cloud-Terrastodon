use super::FieldUpdaterWorkFailureMutator;
use super::FieldUpdaterWorkSuccessMutator;
use super::Loadable;
use super::Work;
use cloud_terrastodon_execution::ExecutionFuture;
use std::fmt::Debug;
use std::fmt::Display;
use std::future::Future;
use std::marker::PhantomData;
use std::panic::Location;
use std::sync::Arc;
use std::time::Instant;
use teamy_cancellation::CancellationScope;
use teamy_tracing::DiagnosticReport;

pub type Setter<State, FinishedWorkData> =
    Arc<dyn Fn(&mut State, Loadable<FinishedWorkData>) + Send + Sync>;
type LocatedWorkFactory<T> = (
    &'static Location<'static>,
    Box<dyn FnOnce(CancellationScope) -> ExecutionFuture<'static, T> + Send>,
);

/// Builds background work that publishes its result to an application field.
/// Supply both a setter and a work factory, in either order, before calling `build`.
///
/// ```compile_fail
/// use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
///
/// LoadableWorkBuilder::<(), ()>::new()
///     .work(async |_| eyre::Ok(()))
///     .build(); // Missing the setter.
/// ```
///
/// ```compile_fail
/// use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
///
/// LoadableWorkBuilder::<(), ()>::new()
///     .setter(|_, _| {})
///     .build(); // Missing the work factory.
/// ```
pub struct LoadableWorkBuilder<State, FinishedWorkData, FieldSetter = (), WorkFactory = ()>
where
    FinishedWorkData: Debug + Send,
{
    setter: FieldSetter,
    on_work: WorkFactory,
    description: String,
    marker: PhantomData<fn(&mut State) -> FinishedWorkData>,
}

impl<State, FinishedWorkData> Default for LoadableWorkBuilder<State, FinishedWorkData>
where
    FinishedWorkData: Debug + Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<State, FinishedWorkData> LoadableWorkBuilder<State, FinishedWorkData>
where
    FinishedWorkData: Debug + Send,
{
    pub fn new() -> Self {
        Self {
            setter: (),
            on_work: (),
            description: String::new(),
            marker: PhantomData,
        }
    }
}

impl<State, FinishedWorkData, FieldSetter, WorkFactory>
    LoadableWorkBuilder<State, FinishedWorkData, FieldSetter, WorkFactory>
where
    FinishedWorkData: Debug + Send,
{
    pub fn description(mut self, description: impl Display) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn setter<G>(
        self,
        setter: G,
    ) -> LoadableWorkBuilder<State, FinishedWorkData, Setter<State, FinishedWorkData>, WorkFactory>
    where
        G: Fn(&mut State, Loadable<FinishedWorkData>) + Send + Sync + 'static,
    {
        LoadableWorkBuilder {
            setter: Arc::new(setter),
            on_work: self.on_work,
            description: self.description,
            marker: PhantomData,
        }
    }

    /// Supply a factory that uses the task's own linked cancellation scope.
    #[track_caller]
    pub fn work<C, F>(
        self,
        factory: C,
    ) -> LoadableWorkBuilder<
        State,
        FinishedWorkData,
        FieldSetter,
        LocatedWorkFactory<FinishedWorkData>,
    >
    where
        C: FnOnce(CancellationScope) -> F + Send + 'static,
        F: Future<Output = eyre::Result<FinishedWorkData>> + Send + 'static,
    {
        LoadableWorkBuilder {
            setter: self.setter,
            on_work: (
                Location::caller(),
                Box::new(|scope| Box::pin(factory(scope))),
            ),
            description: self.description,
            marker: PhantomData,
        }
    }
}

impl<State, FinishedWorkData>
    LoadableWorkBuilder<
        State,
        FinishedWorkData,
        Setter<State, FinishedWorkData>,
        LocatedWorkFactory<FinishedWorkData>,
    >
where
    FinishedWorkData: Debug + Send,
{
    #[allow(clippy::type_complexity)]
    pub fn build(
        self,
    ) -> Work<
        State,
        impl Fn(&mut State),
        impl FnOnce(
            CancellationScope,
        )
            -> ExecutionFuture<'static, FieldUpdaterWorkSuccessMutator<State, FinishedWorkData>>
        + Send,
        FieldUpdaterWorkFailureMutator<State, FinishedWorkData>,
        impl Fn(DiagnosticReport) -> FieldUpdaterWorkFailureMutator<State, FinishedWorkData> + Send,
    > {
        let setter = self.setter;
        let (location, on_work) = self.on_work;
        let setter_for_enqueue = Arc::clone(&setter);
        let setter_for_failure = Arc::clone(&setter);
        let started_at = Instant::now();
        Work {
            location,
            on_enqueue: move |state: &mut State| {
                (setter_for_enqueue)(state, Loadable::Loading { started_at });
            },
            on_work: move |scope| -> ExecutionFuture<'static, _> {
                Box::pin(async move {
                    let data = on_work(scope).await?;
                    Ok(FieldUpdaterWorkSuccessMutator {
                        data,
                        setter,
                        started_at,
                    })
                })
            },
            on_failure: move |err| FieldUpdaterWorkFailureMutator {
                err,
                setter: Arc::clone(&setter_for_failure),
                started_at,
            },
            description: self.description,
            _marker: PhantomData,
        }
    }
}
