use crate::cloud_terrastodon_account_admission_state::CloudTerrastodonAccountAdmissionState;
use cloud_terrastodon_command::CancellationToken;
use std::collections::HashSet;
use std::sync::Mutex;
use tokio::sync::Notify;

/// Coordinates closing new account work and draining existing credential leases.
#[derive(Debug)]
pub(crate) struct CloudTerrastodonAccountAdmission {
    pub(crate) state: Mutex<CloudTerrastodonAccountAdmissionState>,
    pub(crate) drained: Notify,
    pub(crate) cancellation: CancellationToken,
}
impl CloudTerrastodonAccountAdmission {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(CloudTerrastodonAccountAdmissionState {
                open: true,
                active: 0,
                partitions: HashSet::new(),
            }),
            drained: Notify::new(),
            cancellation: CancellationToken::new(),
        }
    }
}
