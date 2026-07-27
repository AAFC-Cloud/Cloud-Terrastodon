pub(super) struct HandlerCompletion {
    pub(super) is_startup: bool,
    pub(super) result: eyre::Result<()>,
}
