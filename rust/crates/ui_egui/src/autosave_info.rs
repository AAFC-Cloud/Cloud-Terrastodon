use cloud_terrastodon_core_config::iconfig::IConfig;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::error;

#[derive(Debug)]
pub struct AutoSaveBehaviour<T: IConfig> {
    pub interval: Duration,
    pub last_save: Option<(Instant, T)>,
}
impl<T> AutoSaveBehaviour<T>
where
    T: IConfig,
{
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_save: None,
        }
    }
    pub fn apply(&mut self, config: &T) {
        let (last_save, last_save_copy) = self
            .last_save
            .get_or_insert_with(|| (Instant::now(), config.clone()));
        if last_save.elapsed() > self.interval {
            *last_save = Instant::now();
            if last_save_copy != config {
                *last_save_copy = config.clone();
                debug!("Detected config change during auto save, scheduling write");
                let config_copy = config.clone();
                tokio::runtime::Handle::current().spawn(async move {
                    let result = config_copy.save().await;
                    if let Err(e) = &result {
                        error!("Error in message thread: {:#?}", e)
                    }
                    result
                });
            }
        }
    }
}
impl<T> Default for AutoSaveBehaviour<T>
where
    T: IConfig,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
