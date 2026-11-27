use crate::BmcHelixAccountSession;
use crate::CloudTerrastodonAccountDefault;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use std::collections::HashSet;

/// Account registrations and explicit defaults, persisted without credentials.
#[derive(Debug, Clone, Default, Facet)]
pub struct CloudTerrastodonAccountCatalogue {
    pub accounts: Vec<CloudTerrastodonAccountProfile>,
    pub defaults: Vec<CloudTerrastodonAccountDefault>,
    #[facet(default)]
    pub bmc_helix_sessions: Vec<BmcHelixAccountSession>,
}
impl CloudTerrastodonAccountCatalogue {
    pub fn validate(&self) -> Result<()> {
        let mut ids = HashSet::new();
        let mut aliases = HashSet::new();
        for account in &self.accounts {
            account.validate()?;
            ensure!(
                ids.insert(account.id),
                "account catalogue contains a duplicate account ID"
            );
            ensure!(
                aliases.insert((account.namespace.clone(), &account.alias)),
                "account catalogue contains a duplicate alias in one namespace"
            );
            if let CloudTerrastodonAccountNamespace::BmcHelix { .. } = account.namespace {
                ensure!(
                    account
                        .registration(CloudTerrastodonCredentialSource::BmcHelix)?
                        .session
                        == CloudTerrastodonAccountSession::Unprovisioned,
                    "Helix sessions must use a server-specific credential record"
                );
            }
        }
        let mut defaults = HashSet::new();
        for default in &self.defaults {
            ensure!(
                defaults.insert(&default.namespace),
                "account catalogue contains duplicate defaults"
            );
            let account = self.by_id(default.account_id)?;
            let compatible = match &default.namespace {
                CloudTerrastodonAccountNamespace::BmcHelix { server } => {
                    server.is_some()
                        && matches!(
                            account.namespace,
                            CloudTerrastodonAccountNamespace::BmcHelix { .. }
                        )
                }
                namespace => account.namespace == *namespace,
            };
            ensure!(
                compatible,
                "configured account default has the wrong namespace"
            );
        }
        let mut sessions = HashSet::new();
        let mut generations = HashSet::new();
        for session in &self.bmc_helix_sessions {
            self.bmc_helix_account(session.account_id)?;
            ensure!(
                sessions.insert((session.account_id, &session.server)),
                "account catalogue contains duplicate sessions for one Helix server"
            );
            let generation = match session.session {
                CloudTerrastodonAccountSession::Provisioning { generation }
                | CloudTerrastodonAccountSession::Active { generation }
                | CloudTerrastodonAccountSession::Retired { generation } => generation,
                CloudTerrastodonAccountSession::Unprovisioned => {
                    bail!("a saved Helix session must have a credential generation");
                }
            };
            ensure!(
                generations.insert((session.account_id, generation)),
                "the same Helix credential generation cannot be used on different servers"
            );
        }
        Ok(())
    }

    /// Adopt the old server-bound credential generation without changing its
    /// account ID, alias namespace, server default, or secret-store key.
    pub(crate) fn migrate_bmc_helix_sessions(&mut self) {
        for profile in &mut self.accounts {
            for slot in &mut profile.credentials {
                if let CloudTerrastodonAccountCredential::BmcHelix {
                    server: Some(server),
                    ..
                } = &slot.credential
                    && slot.session != CloudTerrastodonAccountSession::Unprovisioned
                {
                    self.bmc_helix_sessions.push(BmcHelixAccountSession {
                        account_id: profile.id,
                        server: server.clone(),
                        session: std::mem::replace(
                            &mut slot.session,
                            CloudTerrastodonAccountSession::Unprovisioned,
                        ),
                    });
                }
            }
        }
    }

    pub(crate) fn bmc_helix_account(
        &self,
        account_id: CloudTerrastodonAccountId,
    ) -> Result<&CloudTerrastodonAccountProfile> {
        let profile = self.by_id(account_id)?;
        ensure!(
            matches!(
                profile.namespace,
                CloudTerrastodonAccountNamespace::BmcHelix { .. }
            ),
            "selected account is not a BMC Helix account"
        );
        Ok(profile)
    }

    /// Look up the saved session for exactly this account and server.
    pub fn bmc_helix_session(
        &self,
        account_id: CloudTerrastodonAccountId,
        server: &BmcHelixServerUrl,
    ) -> CloudTerrastodonAccountSession {
        self.bmc_helix_sessions
            .iter()
            .find(|record| record.account_id == account_id && record.server == *server)
            .map_or(CloudTerrastodonAccountSession::Unprovisioned, |record| {
                record.session
            })
    }

    /// An explicit account is eligible on any Helix server. Without an
    /// explicit selector, use only the account configured for this server.
    pub fn resolve_bmc_helix(
        &self,
        server: &BmcHelixServerUrl,
        account: Option<&CloudTerrastodonAccountArgument>,
    ) -> Result<&CloudTerrastodonAccountProfile> {
        if let Some(account) = account {
            return self.resolve_in_namespaces(
                account,
                |namespace| matches!(namespace, CloudTerrastodonAccountNamespace::BmcHelix { .. }),
                " among BMC Helix accounts",
            );
        }
        let namespace = CloudTerrastodonAccountNamespace::BmcHelix {
            server: Some(server.clone()),
        };
        let Some(default) = self
            .defaults
            .iter()
            .find(|default| default.namespace == namespace)
        else {
            bail!("no default account is configured for the Helix server {server}");
        };
        self.bmc_helix_account(default.account_id)
    }

    pub fn by_id(&self, id: CloudTerrastodonAccountId) -> Result<&CloudTerrastodonAccountProfile> {
        self.accounts
            .iter()
            .find(|account| account.id == id)
            .ok_or_else(|| eyre::eyre!("account {id} is not registered"))
    }

    /// Resolve an account ID or alias, optionally limiting it to a namespace.
    /// Aliases must be unique in the searched registrations. Omitting the
    /// account uses the configured default for the supplied namespace.
    pub fn resolve(
        &self,
        account: Option<&CloudTerrastodonAccountArgument>,
        namespace: Option<&CloudTerrastodonAccountNamespace>,
    ) -> Result<&CloudTerrastodonAccountProfile> {
        if let Some(CloudTerrastodonAccountNamespace::BmcHelix {
            server: Some(server),
        }) = namespace
        {
            return self.resolve_bmc_helix(server, account);
        }
        if let Some(account) = account {
            return self.resolve_in_namespaces(
                account,
                |candidate| namespace.is_none_or(|namespace| candidate == namespace),
                namespace.map_or("", |_| " in the requested namespace"),
            );
        }
        let Some(namespace) = namespace else {
            bail!("an account ID or alias is required without a namespace");
        };
        let Some(default) = self
            .defaults
            .iter()
            .find(|default| default.namespace == *namespace)
        else {
            bail!("no account was specified and this namespace has no configured account default");
        };
        let account = self.by_id(default.account_id)?;
        ensure!(
            account.namespace == *namespace,
            "selected account belongs to another tenant or service"
        );
        Ok(account)
    }

    /// Resolve a named account among the namespaces eligible for an operation.
    pub(crate) fn resolve_in_namespaces(
        &self,
        account: &CloudTerrastodonAccountArgument,
        eligible: impl Fn(&CloudTerrastodonAccountNamespace) -> bool,
        qualifier: &str,
    ) -> Result<&CloudTerrastodonAccountProfile> {
        let alias = match account {
            CloudTerrastodonAccountArgument::Id(id) => {
                let account = self.by_id(*id)?;
                ensure!(
                    eligible(&account.namespace),
                    "selected account belongs to another tenant or service"
                );
                return Ok(account);
            }
            CloudTerrastodonAccountArgument::Alias(alias) => alias.as_str(),
        };
        let mut matches = self
            .accounts
            .iter()
            .filter(|account| account.alias.as_str() == alias && eligible(&account.namespace));
        let first = matches
            .next()
            .ok_or_else(|| eyre::eyre!("account alias {alias:?} is not registered{qualifier}"))?;
        if let Some(second) = matches.next() {
            let candidates = [first, second]
                .into_iter()
                .chain(matches)
                .map(|account| {
                    let namespace = match &account.namespace {
                        CloudTerrastodonAccountNamespace::Entra { tenant_id } => {
                            format!("Entra tenant {tenant_id}")
                        }
                        CloudTerrastodonAccountNamespace::AzureDevOps { organization } => {
                            format!("Azure DevOps organization {organization}")
                        }
                        CloudTerrastodonAccountNamespace::Gitea { instance } => {
                            format!("Gitea instance {instance}")
                        }
                        CloudTerrastodonAccountNamespace::BmcHelix {
                            server: Some(server),
                        } => {
                            format!("BMC Helix server {server}")
                        }
                        CloudTerrastodonAccountNamespace::BmcHelix { server: None } => {
                            "BMC Helix".to_owned()
                        }
                    };
                    format!("  {} ({namespace})", account.id)
                })
                .collect::<Vec<_>>()
                .join("\n");
            bail!(
                "account alias {alias:?} matches multiple registrations. Use one of these account IDs.\n{candidates}"
            );
        }
        Ok(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use uuid::Uuid;

    #[test]
    fn scoped_aliases_and_explicit_defaults_never_choose_the_first_account() -> Result<()> {
        let tenants = [
            AzureTenantId::new(Uuid::from_u128(1)),
            AzureTenantId::new(Uuid::from_u128(2)),
        ];
        let mut catalogue = CloudTerrastodonAccountCatalogue::default();
        for (index, tenant_id) in tenants.into_iter().enumerate() {
            catalogue.accounts.push(CloudTerrastodonAccountProfile::new(
                CloudTerrastodonAccountId::new(Uuid::from_u128(index as u128 + 10)),
                "admin".parse()?,
                CloudTerrastodonAccountCredential::AzureCli { tenant_id },
            ));
        }
        catalogue.validate()?;
        let namespace = CloudTerrastodonAccountNamespace::Entra {
            tenant_id: tenants[0],
        };
        let admin: CloudTerrastodonAccountArgument = "admin".parse()?;
        let ambiguous = catalogue
            .resolve(Some(&admin), None)
            .unwrap_err()
            .to_string();
        assert!(ambiguous.contains("matches multiple registrations"));
        for account in &catalogue.accounts {
            assert!(ambiguous.contains(&account.id.to_string()));
        }
        assert!(catalogue.resolve(Some(&"Admin".parse()?), None).is_err());
        assert_eq!(
            catalogue.resolve(Some(&admin), Some(&namespace))?.id,
            catalogue.accounts[0].id
        );
        assert!(catalogue.resolve(None, Some(&namespace)).is_err());
        let other_account = CloudTerrastodonAccountArgument::Id(catalogue.accounts[1].id);
        assert_eq!(
            catalogue.resolve(Some(&other_account), None)?.id,
            catalogue.accounts[1].id
        );
        assert!(
            catalogue
                .resolve(Some(&other_account), Some(&namespace))
                .is_err()
        );
        catalogue.defaults.push(CloudTerrastodonAccountDefault {
            namespace: namespace.clone(),
            account_id: catalogue.accounts[0].id,
        });
        assert_eq!(
            catalogue.resolve(None, Some(&namespace))?.id,
            catalogue.accounts[0].id
        );
        assert!(catalogue.resolve(None, None).is_err());
        catalogue.accounts.remove(0);
        assert_eq!(
            catalogue.resolve(Some(&admin), None)?.id,
            catalogue.accounts[0].id
        );
        assert!(catalogue.resolve(Some(&admin), Some(&namespace)).is_err());
        assert!(catalogue.resolve(None, Some(&namespace)).is_err());
        Ok(())
    }
}
