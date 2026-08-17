use facet::Shape;
use facet_reflect::Peek;

use super::revision::RootRevision;

/// A reflected read tied to the Arena borrow that produced it.
///
/// The root revision is observation metadata, not part of ValueAddress
/// identity. Callers that retain derived snapshots can later reject them
/// through ArenaAddressSource::resolve_at_revision.
#[derive(Clone, Copy)]
pub(crate) struct ResolvedValue<'arena> {
    peek: Peek<'arena, 'static>,
    root_revision: RootRevision,
}

impl<'arena> ResolvedValue<'arena> {
    pub(crate) const fn new(peek: Peek<'arena, 'static>, root_revision: RootRevision) -> Self {
        Self {
            peek,
            root_revision,
        }
    }

    pub(crate) const fn peek(self) -> Peek<'arena, 'static> {
        self.peek
    }

    pub(crate) const fn shape(self) -> &'static Shape {
        self.peek.shape()
    }

    pub(crate) const fn root_revision(self) -> RootRevision {
        self.root_revision
    }
}
