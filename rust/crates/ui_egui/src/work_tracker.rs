use std::cell::RefCell;

use tokio::task::JoinHandle;

#[derive(Debug, Default)]
pub struct WorkTracker {
    remaining_work: RefCell<Vec<JoinHandle<eyre::Result<()>>>>,
}

impl WorkTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track(&self, work: JoinHandle<eyre::Result<()>>) {
        let mut remaining_work = self.remaining_work.borrow_mut();
        remaining_work.push(work);
    }
}
