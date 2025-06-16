use crate::work::WorkHandle;
use crate::work_tracker::WorkTracker;
use cloud_terrastodon_config::Config;
use std::any::type_name;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::error;

#[derive(Debug)]
pub struct AutoSaveBehaviour<T: Config> {
    pub interval: Duration,
    pub last_save: Option<(Instant, T)>,
}
impl<T> AutoSaveBehaviour<T>
where
    T: Config,
{
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_save: None,
        }
    }
    pub fn apply(&mut self, config: &T, work_tracker: Rc<WorkTracker>) {
        let (last_save, last_save_copy) = self
            .last_save
            .get_or_insert_with(|| (Instant::now(), config.clone()));
        if last_save.elapsed() > self.interval {
            *last_save = Instant::now();
            if last_save_copy != config {
                *last_save_copy = config.clone();
                debug!("Detected config change during auto save, scheduling write");
                let config_copy = config.clone();
                let description = format!("Autosave for {}", type_name::<T>());
                let join_handle = tokio::runtime::Handle::current().spawn(async move {
                    let result = config_copy.save().await;
                    if let Err(e) = &result {
                        error!("Error in message thread: {:#?}", e)
                    }
                    result
                });
                work_tracker.track(WorkHandle {
                    join_handle,
                    description,
                    is_err_if_discarded: true,
                });
            }
        }
    }
}
impl<T> Default for AutoSaveBehaviour<T>
where
    T: Config,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
