use crate::ExcalidrawElementId;
use facet::Facet;

/// [Excalidraw 0.18 PointBinding](https://github.com/excalidraw/excalidraw/blob/v0.18.0/packages/excalidraw/element/types.ts).
/// This v2 scene binding is also used by the VS Code editor. Newer Excalidraw
/// versions migrate legacy focus/gap bindings when restoring a scene.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct ExcalidrawArrowBinding {
    pub element_id: ExcalidrawElementId,
    pub focus: f64,
    pub gap: f64,
}
