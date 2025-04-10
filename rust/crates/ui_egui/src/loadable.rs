#[derive(Debug, Default)]
pub enum Loadable<T,E> {
    #[default]
    NotLoaded,
    Loading,
    Loaded(T),
    Failed(E)
}
