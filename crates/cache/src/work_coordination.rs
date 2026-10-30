use tokio::sync::Mutex;

/// One concrete entry's stable initializer lock, retained by every live caller.
/// The registry holds only weak references, independently of decoded values.
#[derive(Debug, Default)]
pub(crate) struct WorkCoordination {
    pub initializer: Mutex<()>,
}
