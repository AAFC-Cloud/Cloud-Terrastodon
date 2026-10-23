use facet::Facet;

/// [Element fill vocabulary](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum ExcalidrawFillStyle {
    #[facet(rename = "solid")]
    Solid,
    #[facet(rename = "hachure")]
    Hachure,
    #[facet(rename = "cross-hatch")]
    CrossHatch,
    #[facet(rename = "zigzag")]
    Zigzag,
}
