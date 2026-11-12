use crate::cloud_terrastodon_account_admission::CloudTerrastodonAccountAdmission;
use crate::cloud_terrastodon_account_authentication::CloudTerrastodonAccountAuthentication;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use std::sync::Arc;

/// Captured credentials share admission state with every provider made from the
/// same invocation context. Keeping credentials outside that state avoids a
/// reference cycle while leases retain the captured identity through completion.
pub(crate) struct CloudTerrastodonAccountBinding {
    pub(crate) authentication: CloudTerrastodonAccountAuthentication,
    pub(crate) admission: Arc<CloudTerrastodonAccountAdmission>,
    /// Non-secret metadata captured when loading a named credential generation.
    /// New admission validates its selected slot; existing leases finish without
    /// consulting mutable catalogue state. Explicit injected sessions have none.
    pub(crate) profile: Option<CloudTerrastodonAccountProfile>,
}
