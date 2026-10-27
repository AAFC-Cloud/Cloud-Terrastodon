use crate::object_explorer::ArenaQueryContextError;
use crate::object_explorer::ExplorerHandleError;
use std::error::Error;
use std::fmt;

/// Preserve the command rejection boundary so an unsuccessful user action can
/// be displayed without mistaking a lost engine or UI invariant for a retry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ObjectBrowserControllerError {
    Request(ExplorerHandleError),
    Query(ArenaQueryContextError),
    Engine(String),
    NoActiveTab,
}

impl fmt::Display for ObjectBrowserControllerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(error) => error.fmt(formatter),
            Self::Query(error) => error.fmt(formatter),
            Self::Engine(message) => formatter.write_str(message),
            Self::NoActiveTab => formatter.write_str("the object browser has no active tab"),
        }
    }
}

impl Error for ObjectBrowserControllerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(error) => Some(error),
            Self::Query(error) => Some(error),
            Self::Engine(_) | Self::NoActiveTab => None,
        }
    }
}

impl From<ArenaQueryContextError> for ObjectBrowserControllerError {
    fn from(value: ArenaQueryContextError) -> Self {
        Self::Query(value)
    }
}

impl From<ExplorerHandleError> for ObjectBrowserControllerError {
    fn from(value: ExplorerHandleError) -> Self {
        Self::Request(value)
    }
}
