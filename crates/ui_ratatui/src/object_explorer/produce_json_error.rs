use super::arena_query_context::ArenaQueryContextError;
use cloud_terrastodon_command::CancellationError;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum ProduceJsonError {
    Cancelled(CancellationError),
    Query(ArenaQueryContextError),
    Io {
        path: PathBuf,
        operation: &'static str,
        source: std::io::Error,
    },
}

impl fmt::Display for ProduceJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled(error) => error.fmt(formatter),
            Self::Query(error) => error.fmt(formatter),
            Self::Io {
                path,
                operation,
                source,
            } => write!(
                formatter,
                "could not {operation} {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for ProduceJsonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Cancelled(error) => Some(error),
            Self::Query(error) => Some(error),
            Self::Io { source, .. } => Some(source),
        }
    }
}

impl From<ArenaQueryContextError> for ProduceJsonError {
    fn from(value: ArenaQueryContextError) -> Self {
        Self::Query(value)
    }
}

impl From<CancellationError> for ProduceJsonError {
    fn from(value: CancellationError) -> Self {
        Self::Cancelled(value)
    }
}
