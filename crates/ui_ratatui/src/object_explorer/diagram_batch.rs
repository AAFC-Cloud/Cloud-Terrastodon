use cloud_terrastodon_excalidraw::ExcalidrawObjectSnapshot;

/// One bounded transfer of owned drawing observations from a coherent query.
#[derive(Debug)]
pub(crate) struct DiagramBatch {
    pub(crate) objects: Vec<ExcalidrawObjectSnapshot>,
    pub(crate) inspected: usize,
    pub(crate) complete: bool,
}
