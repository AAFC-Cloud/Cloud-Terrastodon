pub(super) struct HandlerCompletion {
    pub(super) generation: u64,
    pub(super) is_startup: bool,
    pub(super) result: eyre::Result<()>,
}
