use crate::ExcalidrawBoundElement;
use crate::ExcalidrawElementId;
use crate::ExcalidrawFillStyle;
use crate::ExcalidrawRoundness;
use crate::ExcalidrawStrokeStyle;
use facet::Facet;

/// Shared [element geometry and rendering state](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
/// The exporter uses stable seeds and no wall-clock timestamps for reproducible files.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct ExcalidrawElementBase {
    pub id: ExcalidrawElementId,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    pub stroke_color: String,
    pub background_color: String,
    pub fill_style: ExcalidrawFillStyle,
    pub stroke_width: f64,
    pub stroke_style: ExcalidrawStrokeStyle,
    pub roughness: f64,
    pub opacity: u8,
    pub group_ids: Vec<String>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub frame_id: Option<ExcalidrawElementId>,
    /// Omission lets the editor assign fractional ordering from the array order.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub index: Option<String>,
    // The pinned Facet tagged/flattened decoder eagerly enters Option<struct>.
    // Omit absent rounding rather than encoding null; Excalidraw's restore
    // boundary restores omitted roundness as null (square corners).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub roundness: Option<ExcalidrawRoundness>,
    pub seed: u32,
    pub version: u32,
    pub version_nonce: u32,
    pub is_deleted: bool,
    pub bound_elements: Vec<ExcalidrawBoundElement>,
    pub updated: u64,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub link: Option<String>,
    pub locked: bool,
}

impl ExcalidrawElementBase {
    pub fn new(id: ExcalidrawElementId, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            angle: 0.0,
            stroke_color: "#1e1e1e".into(),
            background_color: "transparent".into(),
            fill_style: ExcalidrawFillStyle::Solid,
            stroke_width: 1.0,
            stroke_style: ExcalidrawStrokeStyle::Solid,
            roughness: 0.0,
            opacity: 100,
            group_ids: Vec::new(),
            frame_id: None,
            index: None,
            roundness: None,
            seed: 1,
            version: 1,
            version_nonce: 1,
            is_deleted: false,
            bound_elements: Vec::new(),
            updated: 0,
            link: None,
            locked: false,
        }
    }
}
