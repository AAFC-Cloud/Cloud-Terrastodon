use crate::AuthContext;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::BmcHelixAuthorization;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountService;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
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
    BmcHelix {
        server: BmcHelixServerUrl,
        authorization: BmcHelixAuthorization,
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
            (
                Self::BmcHelix {
                    server, generation, ..
                },
                selection,
            ) => {
                let Some((account_id, server_id)) = selection.bmc_helix_session() else {
                    eyre::bail!("requested credential source does not match this account");
                };
                eyre::ensure!(
                    server.id() == server_id,
                    "requested Helix session does not match this account binding"
                );
                Self::bmc_helix_partition(account_id, server_id, *generation)
            }
            _ => {
                eyre::bail!("requested credential source does not match this account");
            }
        }
    }

    pub(crate) fn bmc_helix_partition(
        account_id: CloudTerrastodonAccountId,
        server_id: BmcHelixServerId,
        generation: Uuid,
    ) -> Result<AuthPartition> {
        AuthPartition::new(format!(
            "account/{account_id}/helix-server/{server_id}/session/{generation}"
        ))
    }
}
