use super::WorkHandle;
use eyre::bail;
use eyre::eyre;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::error;
use tracing::info;

#[derive(Debug)]
pub struct AppWorkTracker<State> {
    // We use interior mutability here to avoid annoyances when passing around context when doing immediate mode UI stuff
    remaining_work: Arc<Mutex<RefCell<Vec<WorkHandle>>>>,
    _marker: std::marker::PhantomData<State>,
}
impl<T> Clone for AppWorkTracker<T> {
    fn clone(&self) -> Self {
        Self {
            remaining_work: Arc::clone(&self.remaining_work),
            _marker: std::marker::PhantomData,
        }
    }
}
impl<T> Default for AppWorkTracker<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<State> AppWorkTracker<State> {
    pub fn new() -> Self {
        Self {
            remaining_work: Default::default(),
            _marker: std::marker::PhantomData,
        }
    }
    pub fn track(&self, work: WorkHandle) -> eyre::Result<()> {
        let remaining_work = self
            .remaining_work
            .lock()
            .map_err(|e| eyre!("Failed to get lock: {e:?}"))?;
        let mut remaining_work = remaining_work
            .borrow_mut();
        remaining_work.push(work);
        Ok(())
    }
    pub fn prune(&self) -> eyre::Result<()> {
        let remaining_work = self
            .remaining_work
            .lock()
            .map_err(|e| eyre!("Failed to get lock: {e:?}"))?;
        let mut remaining_work = remaining_work
            .borrow_mut();
        remaining_work.retain(|work| !work.join_handle.is_finished());
        Ok(())
    }
    pub async fn finish(self) -> eyre::Result<()> {
        let remaining_work = self
            .remaining_work
            .lock()
            .map_err(|e| eyre!("Failed to get lock: {e:?}"))?;
        let mut remaining_work = remaining_work
            .take();
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
