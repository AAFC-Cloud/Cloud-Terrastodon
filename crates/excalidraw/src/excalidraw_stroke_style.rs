use facet::Facet;

/// [Element stroke vocabulary](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Facet)]
#[facet(rename_all = "lowercase")]
#[repr(u8)]
pub enum ExcalidrawStrokeStyle {
    Solid,
    Dashed,
    Dotted,
}
