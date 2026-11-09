use super::FieldUpdaterWorkFailureMutator;
use super::FieldUpdaterWorkSuccessMutator;
use super::Loadable;
use super::Work;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::bail;
use std::fmt::Debug;
use std::fmt::Display;
use std::future::Future;
use std::panic::Location;
use std::sync::Arc;
use std::time::Instant;
use teamy_cancellation::CancellationScope;

pub type Setter<State, FinishedWorkData> =
    Arc<dyn Fn(&mut State, Loadable<FinishedWorkData, eyre::Error>) + Send + Sync>;
type LoadableWorkFactory<T> =
    Box<dyn FnOnce(CancellationScope) -> ExecutionFuture<'static, T> + Send>;

/// Builds background work that publishes its result to an application field.
pub struct LoadableWorkBuilder<State, FinishedWorkData>
where
    FinishedWorkData: Debug + Send,
{
    setter: Option<Setter<State, FinishedWorkData>>,
    on_work: Option<(
        &'static Location<'static>,
        LoadableWorkFactory<FinishedWorkData>,
    )>,
    description: String,
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
            setter: None,
            on_work: None,
            description: String::new(),
        }
    }

    pub fn description(mut self, description: impl Display) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn setter<G>(mut self, setter: G) -> Self
    where
        G: Fn(&mut State, Loadable<FinishedWorkData, eyre::Error>) + Send + Sync + 'static,
    {
        self.setter = Some(Arc::new(setter));
        self
    }

    /// Supply a factory that uses the task's own linked cancellation scope.
    #[track_caller]
    pub fn work<C, F>(mut self, factory: C) -> Self
    where
        C: FnOnce(CancellationScope) -> F + Send + 'static,
        F: Future<Output = eyre::Result<FinishedWorkData>> + Send + 'static,
    {
        self.on_work = Some((
            Location::caller(),
            Box::new(|scope| Box::pin(factory(scope))),
        ));
        self
    }

    #[allow(clippy::type_complexity)]
    pub fn build(
        self,
    ) -> eyre::Result<
        Work<
            State,
            impl Fn(&mut State),
            impl FnOnce(
                CancellationScope,
            ) -> ExecutionFuture<
                'static,
                FieldUpdaterWorkSuccessMutator<State, FinishedWorkData>,
            > + Send,
            FieldUpdaterWorkFailureMutator<State, FinishedWorkData>,
            impl Fn(eyre::Error) -> FieldUpdaterWorkFailureMutator<State, FinishedWorkData> + Send,
        >,
    > {
        let Some(setter) = self.setter else {
            bail!("Setter was not set!");
        };
        let Some((location, on_work)) = self.on_work else {
            bail!("Work factory was not set!");
        };
        let setter_for_enqueue = Arc::clone(&setter);
        let setter_for_failure = Arc::clone(&setter);
        let started_at = Instant::now();
        Ok(Work {
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
            _marker: std::marker::PhantomData,
        })
    }
}
