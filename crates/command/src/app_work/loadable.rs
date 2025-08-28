use std::time::Instant;

#[derive(Debug, Default)]
pub enum Loadable<T, E = eyre::Error> {
    #[default]
    NotLoaded,
    Loading {
        started_at: Instant,
    },
    Loaded {
        value: T,
        started_at: Instant,
        finished_at: Instant,
    },
    Failed {
        error: E,
        started_at: Instant,
        finished_at: Instant,
    },
}
impl<T, E> Loadable<T, E> {
    pub fn map<N, F>(self, mapper: F) -> Loadable<N, E>
    where
        F: FnOnce(T) -> N,
    {
        match self {
            Self::Loaded {
                value,
                started_at,
                finished_at,
            } => Loadable::Loaded {
                value: mapper(value),
                started_at,
                finished_at,
            },
            Self::NotLoaded => Loadable::NotLoaded,
            Self::Loading { started_at } => Loadable::Loading { started_at },
            Self::Failed {
                error,
                started_at,
                finished_at,
            } => Loadable::Failed {
                error,
                started_at,
                finished_at,
            },
        }
    }
    pub fn as_loaded(&self) -> Option<&T> {
        match self {
            Self::NotLoaded | Self::Loading { .. } | Self::Failed { .. } => None,
            Self::Loaded { value, .. } => Some(value),
        }
    }
}
