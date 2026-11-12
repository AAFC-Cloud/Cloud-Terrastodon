use cloud_terrastodon_auth_types::CloudTerrastodonAccountService;
use facet::Facet;

/// The non-secret fields used to validate an explicitly supplied tea snapshot.
/// Other tea display fields do not participate in credential selection.
#[derive(Facet)]
pub(crate) struct GiteaAccountLoginMetadata {
    pub(crate) name: String,
    pub(crate) url: CloudTerrastodonAccountService,
}
