use crate::cache_state::CacheState;
use crate::work_certificate::WorkCertificate;
use eyre::Result;
use eyre::eyre;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

/// One shared canonical value with an observation that hits cannot renew.
#[derive(Debug)]
pub(crate) struct CachedWorkValue {
    pub value: Arc<dyn Any + Send + Sync>,
    pub certificate: WorkCertificate,
}

impl CachedWorkValue {
    pub fn is_reusable(&self, state: &CacheState) -> bool {
        self.certificate.is_reusable(state)
    }

    pub fn load<T: Send + Sync + 'static>(
        &self,
        state: &CacheState,
        validity: Duration,
    ) -> Result<Option<(Arc<T>, WorkCertificate)>> {
        let certificate = self.certificate.for_caller(validity);
        if !certificate.is_reusable(state) {
            return Ok(None);
        }
        Arc::clone(&self.value)
            .downcast::<T>()
            .map(|value| Some((value, certificate)))
            .map_err(|_| eyre!("cached value violated its claimed output type"))
    }
}
