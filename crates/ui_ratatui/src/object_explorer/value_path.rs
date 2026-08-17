/// One reflected operation below an ownership-bearing arena root.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, facet::Facet)]
#[repr(C)]
pub(crate) enum ValuePathSegment {
    Field(String),
    Index(usize),
    Key(String),
}

/// A reflected path below a root. An empty path addresses the root itself.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ValuePath(Vec<ValuePathSegment>);

impl ValuePath {
    pub(crate) fn child(&self, segment: ValuePathSegment) -> Self {
        let mut segments = self.0.clone();
        segments.push(segment);
        Self(segments)
    }

    pub(crate) fn segments(&self) -> &[ValuePathSegment] {
        &self.0
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        let (_, parent) = self.0.split_last()?;
        Some(Self(parent.to_vec()))
    }
}
