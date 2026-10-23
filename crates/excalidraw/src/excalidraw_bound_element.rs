use crate::ExcalidrawElementId;
use facet::Facet;

/// Reverse attachment from a container to its text or arrows. Both directions
/// are emitted so moving a node in the editor keeps its connections attached.
/// See [BoundElement](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(tag = "type", rename_all = "lowercase")]
#[repr(u8)]
pub enum ExcalidrawBoundElement {
    Text { id: ExcalidrawElementId },
    Arrow { id: ExcalidrawElementId },
}
