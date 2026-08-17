use super::value_address::ValueAddress;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FieldBindingError {
    NestedMoveSource(ValueAddress),
}

impl fmt::Display for FieldBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NestedMoveSource(address) => write!(
                formatter,
                "{address} is a reflected projection; Move currently requires an owned root address"
            ),
        }
    }
}

impl Error for FieldBindingError {}
