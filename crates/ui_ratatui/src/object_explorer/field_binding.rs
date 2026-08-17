use super::field_binding_error::FieldBindingError;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use cloud_terrastodon_registry::RuntimeValue;

/// Explicit construction semantics for one reflected field.
#[derive(Debug)]
pub(crate) enum FieldBinding {
    Unset,
    Default,
    InlineOwned(RuntimeValue),
    CloneFrom(ValueAddress),
    MoveFrom(SlotId),
    BorrowFrom(ValueAddress),
    PendingProducer,
}

impl FieldBinding {
    pub(crate) fn move_from_address(address: ValueAddress) -> Result<Self, FieldBindingError> {
        if !address.path().segments().is_empty() {
            return Err(FieldBindingError::NestedMoveSource(address));
        }
        Ok(Self::MoveFrom(address.root_id()))
    }

    pub(crate) const fn is_resolved(&self) -> bool {
        matches!(
            self,
            Self::Default
                | Self::InlineOwned(_)
                | Self::CloneFrom(_)
                | Self::MoveFrom(_)
                | Self::BorrowFrom(_)
        )
    }

    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Self::Unset => "Unset",
            Self::Default => "Default",
            Self::InlineOwned(_) => "InlineOwned",
            Self::CloneFrom(_) => "CloneFrom",
            Self::MoveFrom(_) => "MoveFrom",
            Self::BorrowFrom(_) => "BorrowFrom",
            Self::PendingProducer => "PendingProducer",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::value_path::ValuePathSegment;

    #[test]
    fn move_binding_accepts_only_an_owned_root_address() {
        let root = ValueAddress::root(SlotId::new(4));
        assert!(matches!(
            FieldBinding::move_from_address(root),
            Ok(FieldBinding::MoveFrom(slot)) if slot == SlotId::new(4)
        ));

        let nested = ValueAddress::root(SlotId::new(4))
            .child(ValuePathSegment::Field("breadcrumbs".to_owned()));
        assert_eq!(
            FieldBinding::move_from_address(nested.clone())
                .expect_err("a projected field has no independent ownership to move"),
            FieldBindingError::NestedMoveSource(nested)
        );
    }
}
