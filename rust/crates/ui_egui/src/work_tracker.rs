use eyre::bail;
use std::cell::RefCell;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::info;

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
    pub async fn finish(&self) -> eyre::Result<()> {
        let mut remaining_work = self.remaining_work.borrow_mut();
        if remaining_work.is_empty() {
            return Ok(());
        }
        info!(
            "Waiting for {} background tasks to finish...",
            remaining_work.len()
        );
        let mut pass = true;
        while let Some(work) = remaining_work.pop() {
            match work.await? {
                Ok(()) => {}
                Err(e) => {
                    pass = false;
                    error!("Error encountered when waiting for background work to finish: {e:?}");
                }
            }
            info!(
                "Waiting for {} background tasks to finish...",
                remaining_work.len()
            );
        }
        if pass {
            Ok(())
        } else {
            bail!("Failed to complete all work. See above for details.");
        }
    }
}
