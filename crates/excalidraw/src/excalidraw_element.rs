use crate::ExcalidrawArrowBinding;
use crate::ExcalidrawArrowhead;
use crate::ExcalidrawElementBase;
use crate::ExcalidrawElementId;
use crate::ExcalidrawTextAlignment;
use crate::ExcalidrawVerticalAlignment;
use facet::Facet;

/// The [scene elements](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts)
/// supported by this exporter. Variant-specific fields cannot leak into another
/// element kind. The common fields are flattened into each JSON element.
/// Absent nullable properties are omitted: the editor's
/// [restore boundary](https://github.com/excalidraw/excalidraw/blob/master/packages/excalidraw/data/restore.ts)
/// supplies their null defaults. This also avoids the pinned Facet decoder's
/// handling of null composite values in tagged enums with flattened fields.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(tag = "type", rename_all = "lowercase")]
#[repr(u8)]
pub enum ExcalidrawElement {
    Rectangle {
        #[facet(flatten)]
        base: ExcalidrawElementBase,
    },
    Text {
        #[facet(flatten)]
        base: ExcalidrawElementBase,
        text: String,
        #[facet(rename = "fontSize")]
        font_size: f64,
        /// Excalidraw font ID; diagrams use 3 (Cascadia, monospace).
        #[facet(rename = "fontFamily")]
        font_family: u8,
        #[facet(rename = "textAlign")]
        text_align: ExcalidrawTextAlignment,
        #[facet(rename = "verticalAlign")]
        vertical_align: ExcalidrawVerticalAlignment,
        #[facet(rename = "containerId", default, skip_serializing_if = Option::is_none)]
        container_id: Option<ExcalidrawElementId>,
        #[facet(rename = "originalText")]
        original_text: String,
        #[facet(rename = "autoResize")]
        auto_resize: bool,
        #[facet(rename = "lineHeight")]
        line_height: f64,
    },
    Arrow {
        #[facet(flatten)]
        base: ExcalidrawElementBase,
        /// Points are relative to (base.x, base.y), with the first point [0, 0].
        points: Vec<[f64; 2]>,
        #[facet(rename = "lastCommittedPoint", default, skip_serializing_if = Option::is_none)]
        last_committed_point: Option<[f64; 2]>,
        #[facet(rename = "startBinding", default, skip_serializing_if = Option::is_none)]
        start_binding: Option<ExcalidrawArrowBinding>,
        #[facet(rename = "endBinding", default, skip_serializing_if = Option::is_none)]
        end_binding: Option<ExcalidrawArrowBinding>,
        #[facet(rename = "startArrowhead", default, skip_serializing_if = Option::is_none)]
        start_arrowhead: Option<ExcalidrawArrowhead>,
        #[facet(rename = "endArrowhead", default, skip_serializing_if = Option::is_none)]
        end_arrowhead: Option<ExcalidrawArrowhead>,
        elbowed: bool,
    },
}

impl ExcalidrawElement {
    pub fn base(&self) -> &ExcalidrawElementBase {
        match self {
            Self::Rectangle { base } | Self::Text { base, .. } | Self::Arrow { base, .. } => base,
        }
    }
}
