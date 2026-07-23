use std::sync::Arc;

pub(super) struct HandlerCompletion {
    pub(super) label: Arc<str>,
    pub(super) is_startup: bool,
    pub(super) result: eyre::Result<()>,
}
