use crate::cloud_terrastodon_account_admission::CloudTerrastodonAccountAdmission;
use crate::cloud_terrastodon_account_authentication::CloudTerrastodonAccountAuthentication;
use std::sync::Arc;

/// Retains one immutable binding for a bounded SDK execution tree.
pub(crate) struct CloudTerrastodonAccountLease {
    pub(crate) admission: Arc<CloudTerrastodonAccountAdmission>,
    pub(crate) binding: CloudTerrastodonAccountAuthentication,
}
impl Drop for CloudTerrastodonAccountLease {
    fn drop(&mut self) {
        match self.admission.state.lock() {
            Ok(mut state) => {
                state.active -= 1;
                if state.active == 0 {
                    self.admission.drained.notify_waiters();
                }
            }
            Err(error) => {
                tracing::error!(%error, "account admission lock poisoned while releasing a credential lease")
            }
        }
    }
}
