use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;

/// The independently initialized and revocable mechanism selected by a request.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct CloudTerrastodonCredentialIdentity {
    pub(crate) account_id: CloudTerrastodonAccountId,
    pub(crate) source: CloudTerrastodonCredentialSource,
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
        };
        Some(Self { account_id, source })
    }

    pub(crate) fn for_context(
        account_id: CloudTerrastodonAccountId,
        source: AuthSource,
    ) -> eyre::Result<Self> {
        Ok(Self {
            account_id,
            source: source.try_into()?,
        })
    }
}
