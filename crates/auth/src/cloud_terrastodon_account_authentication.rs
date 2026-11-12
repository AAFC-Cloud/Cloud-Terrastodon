use crate::AuthContext;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountService;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_execution::GiteaCommandSession;
use eyre::Result;
use std::sync::Arc;
use uuid::Uuid;

/// Captured credential source. A binding never changes after admission.
#[derive(Clone)]
pub(crate) enum CloudTerrastodonAccountAuthentication {
    Azure(AuthContext),
    Gitea {
        instance: CloudTerrastodonAccountService,
        session: Arc<GiteaCommandSession>,
        generation: Uuid,
    },
}
impl CloudTerrastodonAccountAuthentication {
    pub(crate) fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        match (self, selection) {
            (Self::Azure(context), _) => context.admit(selection),
            (Self::Gitea { generation, .. }, AuthenticationSelection::Gitea { account_id }) => {
                AuthPartition::new(format!("account/{account_id}/tea-session/{generation}"))
            }
            _ => {
                eyre::bail!("requested credential source does not match this account");
            }
        }
    }
}
