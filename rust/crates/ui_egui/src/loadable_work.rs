use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::state_mutator::StateMutator;
use crate::work::Work;
use eyre::bail;
use std::fmt::Debug;
use std::future::Future;
use std::panic::Location;
use std::pin::Pin;
use std::sync::Arc;

/// A small helper that builds a `Work` for "fetch data -> store in app field".
pub struct LoadableWorkBuilder<T>
where
    T: Debug + Send,
{
    /// A function that, given `&mut MyApp`, returns the `&mut Loadable<T,E>` that we want to update.
    setter: Option<Arc<dyn Fn(&mut MyApp, Loadable<T, eyre::Error>) -> () + Send + Sync>>,
    /// The async work that fetches a `T` or errors with `E`.
    on_work: Option<(
        &'static Location<'static>,
        Pin<Box<dyn Future<Output = eyre::Result<T>> + Send>>,
    )>,
}

impl<T> LoadableWorkBuilder<T>
where
    T: Debug + Send,
{
    pub fn new() -> Self {
        Self {
            setter: None,
            on_work: None,
        }
    }

    pub fn setter<G>(mut self, setter: G) -> Self
    where
        G: Fn(&mut MyApp, Loadable<T, eyre::Error>) -> () + Send + Sync + 'static,
    {
        self.setter = Some(Arc::new(setter));
        self
    }

    /// Provide the actual future that fetches a T (or errors with E).
    /// For example: `.work(async { fetch_all_subscriptions().await })`
    #[track_caller]
    pub fn work<F>(mut self, future: F) -> Self
    where
        F: Future<Output = eyre::Result<T>> + Send + 'static,
    {
        self.on_work = Some((std::panic::Location::caller(), Box::pin(future)));
        self
    }

    /// Build a `Work` that, when enqueued, sets the chosen field to Loading,
    /// calls the async future, then sets the field to Loaded or Failed.
    pub fn build(
        self,
    ) -> eyre::Result<
        Work<
            // on_enqueue
            impl Fn(&mut MyApp),
            // on_work
            impl Future<Output = eyre::Result<FieldUpdaterWorkSuccessMutator<T>>> + Send,
            // WorkSuccess
            FieldUpdaterWorkSuccessMutator<T>,
            // WorkFailureMutator
            FieldUpdaterWorkFailureMutator<T>,
            // on_failure
            impl Fn(eyre::Error) -> FieldUpdaterWorkFailureMutator<T> + Send,
        >,
    > {
        let Some(setter) = self.setter else {
            bail!("Setter was not set!");
        };
        let Some((location, on_work)) = self.on_work else {
            bail!("Work future was not set!");
        };

        // We'll clone the `Arc` so the future closure and the on_enqueue closure can both own it.
        let setter_for_enqueue = setter.clone();
        let setter_for_failure = setter.clone();

        // Build the final `Work`:
        let work = Work {
            location,
            on_enqueue: move |app: &mut MyApp| {
                (setter_for_enqueue)(app, Loadable::Loading);
            },
            on_work: async move {
                let data = on_work.await?;
                Ok(FieldUpdaterWorkSuccessMutator { data, setter })
            },
            on_failure: move |err| FieldUpdaterWorkFailureMutator {
                err,
                setter: setter_for_failure.clone(),
            },
        };
        Ok(work)
    }
}

/// A success mutator that just sets the chosen field to Loaded(data).
pub struct FieldUpdaterWorkSuccessMutator<T> {
    pub data: T,
    pub setter: Arc<dyn Fn(&mut MyApp, Loadable<T, eyre::Error>) -> () + Send + Sync>,
}
impl<T> Debug for FieldUpdaterWorkSuccessMutator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldUpdaterWorkSuccessMutator")
            // .field("data", &self.data)
            // .field("getter", &self.getter)
            .finish()
    }
}

/// A failure mutator that sets the chosen field to Failed(err).
pub struct FieldUpdaterWorkFailureMutator<T> {
    pub err: eyre::Error,
    pub setter: Arc<dyn Fn(&mut MyApp, Loadable<T, eyre::Error>) -> () + Send + Sync>,
}
impl<T> Debug for FieldUpdaterWorkFailureMutator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldUpdaterWorkSuccessMutator")
            // .field("data", &self.data)
            // .field("getter", &self.getter)
            .finish()
    }
}

impl<T> StateMutator for FieldUpdaterWorkSuccessMutator<T>
where
    T: std::fmt::Debug + Send + 'static,
{
    fn mutate_state(self: Box<Self>, state: &mut MyApp) {
        (self.setter)(state, Loadable::Loaded(self.data))
    }
}

impl<T> StateMutator for FieldUpdaterWorkFailureMutator<T>
where
    T: std::fmt::Debug + Send + 'static,
{
    fn mutate_state(self: Box<Self>, state: &mut MyApp) {
        (self.setter)(state, Loadable::Failed(self.err))
    }
}
