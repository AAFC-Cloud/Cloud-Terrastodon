use facet::Facet;

/// [Corner rounding](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
#[derive(Debug, Clone, PartialEq, Facet)]
pub struct ExcalidrawRoundness {
    #[facet(rename = "type")]
    pub kind: u8,
    pub value: Option<f64>,
}
