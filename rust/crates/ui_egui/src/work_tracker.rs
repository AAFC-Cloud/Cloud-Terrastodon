use eyre::bail;
use std::cell::RefCell;
use tracing::error;
use tracing::info;

use crate::work::WorkHandle;

#[derive(Debug, Default)]
pub struct WorkTracker {
    remaining_work: RefCell<Vec<WorkHandle>>,
}

impl WorkTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track(&self, work: WorkHandle) {
        let mut remaining_work = self.remaining_work.borrow_mut();
        remaining_work.push(work);
    }
    pub fn prune(&self) {
        let mut remaining_work = self.remaining_work.borrow_mut();
        remaining_work.retain(|work| !work.join_handle.is_finished());
    }
    pub async fn finish(&self) -> eyre::Result<()> {
        let mut remaining_work = self.remaining_work.borrow_mut();
        if remaining_work.is_empty() {
            return Ok(());
        }
        let mut pass = true;
        while let Some(work) = remaining_work.pop() {
            match work.is_err_if_discarded {
                true => {
                    info!(
                        "Waiting for task {:?} to finish, {} remain...",
                        work.description,
                        remaining_work.len()
                    );
                    match work.join_handle.await? {
                        Ok(()) => {}
                        Err(e) => {
                            pass = false;
                            error!(
                                "Error encountered when waiting for background work to finish: {e:?}"
                            );
                        }
                    }
                }
                false => {
                    info!(
                        "Discarding work {:?} because it is not marked as important, {} remain...",
                        work.description,
                        remaining_work.len()
                    );
                    work.join_handle.abort();
                }
            }
        }
        if pass {
            Ok(())
        } else {
            bail!("Failed to complete all work. See above for details.");
        }
    }
}
