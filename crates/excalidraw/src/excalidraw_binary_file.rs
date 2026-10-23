use facet::Facet;

/// Embedded image metadata from the
/// [scene file schema](https://docs.excalidraw.com/docs/codebase/json-schema#excalidraw-files).
/// Discovery diagrams use no images and emit an empty files map.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct ExcalidrawBinaryFile {
    pub id: String,
    pub mime_type: String,
    #[facet(rename = "dataURL")]
    pub data_url: String,
    pub created: u64,
    pub last_retrieved: Option<u64>,
}
