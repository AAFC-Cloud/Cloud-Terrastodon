use std::fmt;

use super::slot_id::SlotId;
use super::value_path::{ValuePath, ValuePathSegment};

/// Logical identity of either an arena root or a reflected descendant.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ValueAddress {
    root: SlotId,
    path: ValuePath,
}

impl fmt::Display for ValueAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "slot {}", self.root)?;
        for segment in self.path.segments() {
            match segment {
                ValuePathSegment::Field(field) => write!(formatter, ".{field}")?,
                ValuePathSegment::Index(index) => write!(formatter, "[{index}]")?,
                ValuePathSegment::Key(key) => write!(formatter, "[{key:?}]")?,
            }
        }
        Ok(())
    }
}

impl ValueAddress {
    pub(crate) fn root(root: SlotId) -> Self {
        Self {
            root,
            path: ValuePath::default(),
        }
    }

    pub(crate) fn child(&self, segment: ValuePathSegment) -> Self {
        Self {
            root: self.root,
            path: self.path.child(segment),
        }
    }

    pub(crate) const fn root_id(&self) -> SlotId {
        self.root
    }

    pub(crate) fn path(&self) -> &ValuePath {
        &self.path
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        Some(Self {
            root: self.root,
            path: self.path.parent()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_selection_identity_uses_key_not_ordinal() {
        let address = ValueAddress::root(SlotId::new(5))
            .child(ValuePathSegment::Field("permissions".to_owned()))
            .child(ValuePathSegment::Key("Project Administrators".to_owned()));

        assert_eq!(address.root_id(), SlotId::new(5));
        assert_eq!(
            address.path().segments(),
            [
                ValuePathSegment::Field("permissions".to_owned()),
                ValuePathSegment::Key("Project Administrators".to_owned()),
            ]
        );
        assert!(
            !address
                .path()
                .segments()
                .iter()
                .any(|segment| matches!(segment, ValuePathSegment::Index(_))),
            "map identity must not depend on its current display ordinal"
        );
    }

    #[test]
    fn sequence_identity_uses_an_index_path_segment() {
        let address = ValueAddress::root(SlotId::new(5)).child(ValuePathSegment::Index(42));

        assert_eq!(address.path().segments(), [ValuePathSegment::Index(42)]);
    }
}
