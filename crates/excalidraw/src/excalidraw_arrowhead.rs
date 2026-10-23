use facet::Facet;

/// Supported [arrow terminal](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum ExcalidrawArrowhead {
    #[facet(rename = "arrow")]
    Arrow,
}
