use super::arena::ArenaError;
use super::revision::RootRevision;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_path::ValuePathSegment;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ValueResolutionError {
    Root(ArenaError),
    StaleRootRevision {
        root: SlotId,
        expected: RootRevision,
        actual: RootRevision,
    },
    MissingField {
        parent: ValueAddress,
        field: String,
        shape: String,
    },
    IndexOutOfRange {
        parent: ValueAddress,
        index: usize,
        len: usize,
        shape: String,
    },
    MissingKey {
        parent: ValueAddress,
        key: String,
        shape: String,
    },
    SegmentNotSupported {
        parent: ValueAddress,
        segment: ValuePathSegment,
        shape: String,
    },
    Reflection {
        parent: ValueAddress,
        operation: &'static str,
        message: String,
    },
}

impl fmt::Display for ValueResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root(error) => error.fmt(formatter),
            Self::StaleRootRevision {
                root,
                expected,
                actual,
            } => write!(
                formatter,
                "slot {root} changed from root revision {expected:?} to {actual:?}"
            ),
            Self::MissingField {
                parent,
                field,
                shape,
            } => write!(formatter, "{parent} ({shape}) has no field {field:?}"),
            Self::IndexOutOfRange {
                parent,
                index,
                len,
                shape,
            } => write!(
                formatter,
                "{parent} ({shape}) has length {len}; index {index} is out of range"
            ),
            Self::MissingKey { parent, key, shape } => {
                write!(formatter, "{parent} ({shape}) has no key {key:?}")
            }
            Self::SegmentNotSupported {
                parent,
                segment,
                shape,
            } => write!(
                formatter,
                "{parent} ({shape}) cannot apply path segment {segment:?}"
            ),
            Self::Reflection {
                parent,
                operation,
                message,
            } => write!(
                formatter,
                "could not {operation} while resolving {parent}: {message}"
            ),
        }
    }
}

impl Error for ValueResolutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Root(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ArenaError> for ValueResolutionError {
    fn from(value: ArenaError) -> Self {
        Self::Root(value)
    }
}
