use cloud_terrastodon_registry::RuntimeValue;

/// Lifecycle of one ownership-bearing arena root.
pub(crate) enum ArenaSlotState {
    Building,
    Ready(RuntimeValue),
    Pending,
    Failed(String),
    Cancelled,
    Consumed,
    Tombstone { previous: &'static str },
}

impl ArenaSlotState {
    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Self::Building => "Building",
            Self::Ready(_) => "Ready",
            Self::Pending => "Pending",
            Self::Failed(_) => "Failed",
            Self::Cancelled => "Cancelled",
            Self::Consumed => "Consumed",
            Self::Tombstone { .. } => "Tombstone",
        }
    }

    pub(crate) fn ready_value(&self) -> Option<&RuntimeValue> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }
}
