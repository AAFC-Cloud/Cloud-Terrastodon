use crate::CloudTerrastodonAccountDefault;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use eyre::Result;
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

    pub fn resolve(
        &self,
        namespace: &CloudTerrastodonAccountNamespace,
        selector: Option<&str>,
    ) -> Result<&CloudTerrastodonAccountProfile> {
        let account = if let Some(selector) = selector {
            if let Ok(id) = selector.parse::<CloudTerrastodonAccountId>() {
                self.by_id(id)?
            } else {
                self.accounts.iter().find(|account| account.namespace == *namespace && account.alias.as_str() == selector)
                    .ok_or_else(|| eyre::eyre!("account alias {selector:?} is not registered in the requested namespace"))?
            }
        } else {
            let id = self.defaults.iter().find(|default| default.namespace == *namespace)
                .map(|default| default.account_id)
                .ok_or_else(|| eyre::eyre!("no account was specified and this namespace has no configured account default"))?;
            self.by_id(id)?
        };
        ensure!(
            account.namespace == *namespace,
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
        assert!(catalogue.resolve(&namespace, None).is_err());
        assert_eq!(
            catalogue.resolve(&namespace, Some("admin"))?.id,
            catalogue.accounts[0].id
        );
        assert!(
            catalogue
                .resolve(&namespace, Some(&catalogue.accounts[1].id.to_string()))
                .is_err()
        );
        catalogue.defaults.push(CloudTerrastodonAccountDefault {
            namespace: namespace.clone(),
            account_id: catalogue.accounts[0].id,
        });
        assert_eq!(
            catalogue.resolve(&namespace, None)?.id,
            catalogue.accounts[0].id
        );
        catalogue.accounts.remove(0);
        assert!(catalogue.resolve(&namespace, None).is_err());
        Ok(())
    }
}
