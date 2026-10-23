use facet::Facet;

/// [File discriminator](https://docs.excalidraw.com/docs/codebase/json-schema#excalidraw-files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum ExcalidrawSceneType {
    #[facet(rename = "excalidraw")]
    Excalidraw,
}
