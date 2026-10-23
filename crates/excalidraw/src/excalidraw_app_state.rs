use facet::Facet;

/// Portable canvas preferences from the
/// [scene file schema](https://docs.excalidraw.com/docs/codebase/json-schema#excalidraw-files).
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct ExcalidrawAppState {
    pub grid_size: Option<u32>,
    pub view_background_color: String,
}
