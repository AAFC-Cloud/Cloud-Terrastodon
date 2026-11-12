use crate::AzureBearerAuthenticationSelection;
use crate::AzureDevOpsAuthSelection;
use crate::CloudTerrastodonAccountId;
use arbitrary::Arbitrary;
use facet::Facet;

/// Non-secret authentication intent admitted by an execution provider.
///
/// This is request data rather than acquired authorization or a credential-cache
/// identity. Anonymous execution requests no credential discovery. Bearer
/// selections always carry a resolved source and an explicit tenant.
/// See [Microsoft identity scopes and resource audiences](https://learn.microsoft.com/en-us/entra/identity-platform/scopes-oidc).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[repr(u8)]
pub enum AuthenticationSelection {
    Anonymous,
    AzureBearer(AzureBearerAuthenticationSelection),
    AzureDevOpsPersonalAccessToken {
        account_id: CloudTerrastodonAccountId,
    },
    Gitea {
        account_id: CloudTerrastodonAccountId,
    },
}

impl From<AzureDevOpsAuthSelection> for AuthenticationSelection {
    fn from(selection: AzureDevOpsAuthSelection) -> Self {
        match selection {
            AzureDevOpsAuthSelection::Bearer(selection) => Self::AzureBearer(selection),
            AzureDevOpsAuthSelection::PersonalAccessToken { account_id } => {
                Self::AzureDevOpsPersonalAccessToken { account_id }
            }
        }
    }
}

impl AuthenticationSelection {
    pub const fn account_id(self) -> Option<CloudTerrastodonAccountId> {
        match self {
            Self::Anonymous => None,
            Self::AzureBearer(selection) => Some(selection.account_id),
            Self::AzureDevOpsPersonalAccessToken { account_id } | Self::Gitea { account_id } => {
                Some(account_id)
            }
        }
    }
}

cloud_terrastodon_registry::register_thing!(AuthenticationSelection);
cloud_terrastodon_registry::register_arbitrary!(AuthenticationSelection);
cloud_terrastodon_registry::register_from!(AzureDevOpsAuthSelection => AuthenticationSelection);
