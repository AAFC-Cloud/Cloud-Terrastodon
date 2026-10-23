use facet::Facet;

/// Opaque scene-local identity shared by elements and their bindings.
/// See [element identities](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(transparent)]
pub struct ExcalidrawElementId(pub String);
