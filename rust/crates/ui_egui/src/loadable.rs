#[derive(Debug, Default)]
pub enum Loadable<T, E> {
    #[default]
    NotLoaded,
    Loading,
    Loaded(T),
    Failed(E),
}
impl<T, E> Loadable<T, E> {
    pub fn map<N, F>(self, mapper: F) -> Loadable<N, E>
    where
        F: FnOnce(T) -> N,
    {
        match self {
            Self::NotLoaded => Loadable::NotLoaded,
            Self::Loading => Loadable::Loading,
            Self::Loaded(t) => Loadable::Loaded(mapper(t)),
            Self::Failed(e) => Loadable::Failed(e),
        }
    }
}
