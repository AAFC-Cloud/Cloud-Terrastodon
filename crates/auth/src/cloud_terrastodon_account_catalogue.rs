use crate::CloudTerrastodonAccountDefault;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
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
        }
        let mut defaults = HashSet::new();
        for default in &self.defaults {
            ensure!(
                defaults.insert(&default.namespace),
                "account catalogue contains duplicate defaults"
            );
            let account = self.by_id(default.account_id)?;
            ensure!(
                account.namespace == default.namespace,
                "configured account default has the wrong namespace"
            );
        }
        Ok(())
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
        let account = match account {
            Some(CloudTerrastodonAccountArgument::Id(id)) => self.by_id(*id)?,
            Some(CloudTerrastodonAccountArgument::Alias(alias)) => {
                let alias = alias.as_str();
                let mut matches = self.accounts.iter().filter(|account| {
                    account.alias.as_str() == alias
                        && namespace.is_none_or(|namespace| account.namespace == *namespace)
                });
                let first = matches.next().ok_or_else(|| {
                    let qualifier = namespace.map_or("", |_| " in the requested namespace");
                    eyre::eyre!("account alias {alias:?} is not registered{qualifier}")
                })?;
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
                            };
                            format!("  {} ({namespace})", account.id)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    bail!(
                        "account alias {alias:?} matches multiple registrations. Use one of these account IDs.\n{candidates}"
                    );
                }
                first
            }
            None => {
                let namespace = namespace.ok_or_else(|| {
                    eyre::eyre!("an account ID or alias is required without a namespace")
                })?;
                let default = self
                    .defaults
                    .iter()
                    .find(|default| default.namespace == *namespace)
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "no account was specified and this namespace has no configured account default"
                        )
                    })?;
                self.by_id(default.account_id)?
            }
        };
        ensure!(
            namespace.is_none_or(|namespace| account.namespace == *namespace),
            "selected account belongs to another tenant or service"
        );
        Ok(account)
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
