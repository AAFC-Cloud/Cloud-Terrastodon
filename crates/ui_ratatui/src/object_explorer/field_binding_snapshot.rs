use super::field_binding::FieldBinding;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;

/// UI-neutral, non-owning description of a builder field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FieldBindingSnapshot {
    Unset,
    Default,
    InlineOwned { shape: String },
    CloneFrom(ValueAddress),
    MoveFrom(SlotId),
    BorrowFrom(ValueAddress),
    PendingProducer,
}

impl From<&FieldBinding> for FieldBindingSnapshot {
    fn from(binding: &FieldBinding) -> Self {
        match binding {
            FieldBinding::Unset => Self::Unset,
            FieldBinding::Default => Self::Default,
            FieldBinding::InlineOwned(value) => Self::InlineOwned {
                shape: cloud_terrastodon_registry::describe_shape(value.shape()).to_owned(),
            },
            FieldBinding::CloneFrom(address) => Self::CloneFrom(address.clone()),
            FieldBinding::MoveFrom(slot) => Self::MoveFrom(*slot),
            FieldBinding::BorrowFrom(address) => Self::BorrowFrom(address.clone()),
            FieldBinding::PendingProducer => Self::PendingProducer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::value_path::ValuePathSegment;

    #[test]
    fn binding_snapshot_retains_generic_source_identity() {
        let source = ValueAddress::root(SlotId::new(4))
            .child(ValuePathSegment::Field("breadcrumbs".to_owned()));

        assert_eq!(
            FieldBindingSnapshot::from(&FieldBinding::CloneFrom(source.clone())),
            FieldBindingSnapshot::CloneFrom(source.clone())
        );
        assert_eq!(
            FieldBindingSnapshot::from(&FieldBinding::BorrowFrom(source.clone())),
            FieldBindingSnapshot::BorrowFrom(source)
        );
        assert_eq!(
            FieldBindingSnapshot::from(&FieldBinding::MoveFrom(SlotId::new(8))),
            FieldBindingSnapshot::MoveFrom(SlotId::new(8))
        );
    }
}
