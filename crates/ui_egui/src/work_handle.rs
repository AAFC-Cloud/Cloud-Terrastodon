use tokio::task::JoinHandle;

/// One background task retained until its completion has been observed.
#[derive(Debug)]
pub struct WorkHandle {
    pub join_handle: JoinHandle<eyre::Result<()>>,
    pub description: String,
}
