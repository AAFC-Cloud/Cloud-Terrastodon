use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerId;

/// The independently initialized and revocable mechanism selected by a request.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct CloudTerrastodonCredentialIdentity {
    pub(crate) account_id: CloudTerrastodonAccountId,
    pub(crate) source: CloudTerrastodonCredentialSource,
    /// A Helix session is admitted for a particular server. For retirement
    /// records, absence identifies all sessions for the selected source.
    pub(crate) server_id: Option<BmcHelixServerId>,
}

impl CloudTerrastodonCredentialIdentity {
    pub(crate) fn for_selection(selection: &AuthenticationSelection) -> Option<Self> {
        let (account_id, source) = match *selection {
            AuthenticationSelection::Anonymous => return None,
            AuthenticationSelection::AzureBearer(value) => (
                value.account_id,
                match value.auth_source {
                    AzureBearerAuthSource::AzureCli => CloudTerrastodonCredentialSource::AzureCli,
                    AzureBearerAuthSource::Browser => CloudTerrastodonCredentialSource::Browser,
                    AzureBearerAuthSource::WorkloadIdentity => {
                        CloudTerrastodonCredentialSource::WorkloadIdentity
                    }
                },
            ),
            AuthenticationSelection::AzureDevOpsPersonalAccessToken { account_id } => (
                account_id,
                CloudTerrastodonCredentialSource::PersonalAccessToken,
            ),
            AuthenticationSelection::Gitea { account_id } => {
                (account_id, CloudTerrastodonCredentialSource::Gitea)
            }
            AuthenticationSelection::BmcHelix { .. }
            | AuthenticationSelection::BmcHelixInnovationSuite { .. } => {
                let (account_id, server_id) = selection.bmc_helix_session()?;
                return Some(Self {
                    account_id,
                    source: CloudTerrastodonCredentialSource::BmcHelix,
                    server_id: Some(server_id),
                });
            }
        };
        Some(Self {
            account_id,
            source,
            server_id: None,
        })
    }

    pub(crate) fn for_context(
        account_id: CloudTerrastodonAccountId,
        source: AuthSource,
    ) -> eyre::Result<Self> {
        Ok(Self {
            account_id,
            source: source.try_into()?,
            server_id: None,
        })
    }
}
