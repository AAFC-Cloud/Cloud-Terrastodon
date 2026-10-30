use crate::artifact_age::ArtifactAge;
use chrono::Utc;
use eyre::Result;
use eyre::eyre;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

/// One shared canonical value with an observation that hits cannot renew.
#[derive(Debug)]
pub(crate) struct CachedWorkValue {
    pub value: Arc<dyn Any + Send + Sync>,
    pub age: ArtifactAge,
}

impl CachedWorkValue {
    pub fn is_reusable(&self, validity: Duration) -> bool {
        self.age
            .reusable(&self.age.freshness, validity, Utc::now(), Instant::now())
    }

    pub fn load<T: Send + Sync + 'static>(&self, validity: Duration) -> Result<Option<Arc<T>>> {
        if !self.is_reusable(validity) {
            return Ok(None);
        }
        Arc::clone(&self.value)
            .downcast::<T>()
            .map(Some)
            .map_err(|_| eyre!("cached value violated its claimed output type"))
    }
}
