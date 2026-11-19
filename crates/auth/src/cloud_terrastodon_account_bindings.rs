use crate::cloud_terrastodon_account_binding::CloudTerrastodonAccountBinding;
use crate::cloud_terrastodon_credential_identity::CloudTerrastodonCredentialIdentity;
use std::collections::HashMap;
use std::sync::Arc;

/// The bindings and admission state shared by one authentication provider.
#[derive(Default)]
pub(crate) struct CloudTerrastodonAccountBindings {
    pub(crate) closed: bool,
    pub(crate) entries:
        HashMap<CloudTerrastodonCredentialIdentity, Arc<CloudTerrastodonAccountBinding>>,
}
