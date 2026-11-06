use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Progress from a Microsoft Graph collection's pagination loop.
pub struct MicrosoftGraphProgress {
    pub total: Option<usize>,
    pub accumulated: usize,
    pub received_this_page: usize,
    pub time_remaining: Duration,
}

pub type MicrosoftGraphProgressHook = Box<dyn Fn(MicrosoftGraphProgress) + Send + Sync + 'static>;
