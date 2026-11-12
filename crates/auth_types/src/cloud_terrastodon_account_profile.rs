use crate::AuthSource;
use crate::CloudTerrastodonAccountAlias;
use crate::CloudTerrastodonAccountCredential;
use crate::CloudTerrastodonAccountId;
use crate::CloudTerrastodonAccountNamespace;
use crate::CloudTerrastodonAccountPrincipal;
use crate::CloudTerrastodonAccountSession;
use crate::CloudTerrastodonCredentialRegistration;
use crate::CloudTerrastodonCredentialSource;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::collections::HashSet;

/// Inspectable account identity and credential registrations. Contains no tokens
/// or credential handles. Catalogue boundaries validate the relationships before
/// persisting records or resolving execution selections.
#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct CloudTerrastodonAccountProfile {
    pub id: CloudTerrastodonAccountId,
    pub alias: CloudTerrastodonAccountAlias,
    pub namespace: CloudTerrastodonAccountNamespace,
    pub preferred_source: CloudTerrastodonCredentialSource,
    pub principal: Option<CloudTerrastodonAccountPrincipal>,
    pub credentials: Vec<CloudTerrastodonCredentialRegistration>,
}

impl CloudTerrastodonAccountProfile {
    pub fn new(
        id: CloudTerrastodonAccountId,
        alias: CloudTerrastodonAccountAlias,
        credential: CloudTerrastodonAccountCredential,
    ) -> Self {
        Self {
            id,
            alias,
            namespace: credential.namespace(),
            preferred_source: credential.source(),
            principal: None,
            credentials: vec![CloudTerrastodonCredentialRegistration {
                credential,
                session: CloudTerrastodonAccountSession::Unprovisioned,
            }],
        }
    }

    pub fn registration(
        &self,
        source: CloudTerrastodonCredentialSource,
    ) -> Result<&CloudTerrastodonCredentialRegistration> {
        self.credentials
            .iter()
            .find(|slot| slot.credential.source() == source)
            .ok_or_else(|| {
                eyre::eyre!("the selected account has no {source} credential registration")
            })
    }

    pub fn registration_mut(
        &mut self,
        source: CloudTerrastodonCredentialSource,
    ) -> Result<&mut CloudTerrastodonCredentialRegistration> {
        self.credentials
            .iter_mut()
            .find(|slot| slot.credential.source() == source)
            .ok_or_else(|| {
                eyre::eyre!("the selected account has no {source} credential registration")
            })
    }

    /// Resolve a CLI preference without acquiring credentials or choosing a
    /// different mechanism after an unavailable selection.
    pub fn select_source(&self, source: AuthSource) -> Result<CloudTerrastodonCredentialSource> {
        ensure!(
            source != AuthSource::None,
            "authentication is disabled by auth source none"
        );
        let source = if source == AuthSource::Auto {
            self.preferred_source
        } else {
            source.try_into()?
        };
        self.registration(source)?;
        Ok(source)
    }

    /// First provisioning establishes identity. Subsequent mechanisms and
    /// replacement sessions must authenticate exactly that same principal.
    pub fn verify_principal(&mut self, principal: CloudTerrastodonAccountPrincipal) -> Result<()> {
        ensure!(
            self.principal.is_none_or(|expected| expected == principal),
            "credential authenticates a different principal; register a separate account"
        );
        for slot in &self.credentials {
            ensure!(
                match (&slot.credential, principal) {
                    (
                        CloudTerrastodonAccountCredential::AzureCli { .. }
                        | CloudTerrastodonAccountCredential::Browser { .. },
                        CloudTerrastodonAccountPrincipal::EntraUser(_),
                    ) => true,
                    (
                        CloudTerrastodonAccountCredential::WorkloadIdentity { client_id, .. },
                        CloudTerrastodonAccountPrincipal::EntraApplication(id),
                    ) => *client_id == id,
                    _ => false,
                },
                "verified identity is incompatible with the account's credential registrations"
            );
        }
        self.principal = Some(principal);
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.credentials.is_empty(),
            "account has no credential registrations"
        );
        let mut sources = HashSet::new();
        for slot in &self.credentials {
            ensure!(
                slot.credential.namespace() == self.namespace,
                "credential belongs to a different account namespace"
            );
            ensure!(
                sources.insert(slot.credential.source()),
                "account contains duplicate credential sources"
            );
            if matches!(slot.session, CloudTerrastodonAccountSession::Active { .. })
                && matches!(
                    self.namespace,
                    CloudTerrastodonAccountNamespace::Entra { .. }
                )
            {
                ensure!(
                    self.principal.is_some(),
                    "active Entra account has no verified principal"
                );
            }
            if let CloudTerrastodonAccountCredential::Gitea { login, .. } = &slot.credential {
                ensure!(
                    !login.trim().is_empty() && !login.chars().any(char::is_control),
                    "tea login profile must be nonempty and contain no control characters"
                );
            }
        }
        ensure!(
            sources.contains(&self.preferred_source),
            "preferred credential source is not registered on the account"
        );
        ensure!(
            self.credentials.len() == 1
                || sources.iter().all(|source| matches!(
                    source,
                    CloudTerrastodonCredentialSource::AzureCli
                        | CloudTerrastodonCredentialSource::Browser
                )),
            "only delegated Azure CLI and browser mechanisms may share one account"
        );
        if let Some(principal) = self.principal {
            let mut checked = self.clone();
            checked.verify_principal(principal)?;
        }
        Ok(())
    }
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountProfile);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_azure_types::EntraApplicationClientId;
    use cloud_terrastodon_azure_types::EntraUserId;
    use cloud_terrastodon_azure_types::uuid::Uuid;

    #[test]
    fn additional_mechanisms_preserve_principal_and_source_preference() -> Result<()> {
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        let mut profile = CloudTerrastodonAccountProfile::new(
            CloudTerrastodonAccountId::new(Uuid::from_u128(2)),
            "admin".parse()?,
            CloudTerrastodonAccountCredential::AzureCli { tenant_id },
        );
        let principal =
            CloudTerrastodonAccountPrincipal::EntraUser(EntraUserId::new(Uuid::from_u128(3)));
        profile.verify_principal(principal)?;
        profile
            .credentials
            .push(CloudTerrastodonCredentialRegistration {
                credential: CloudTerrastodonAccountCredential::Browser {
                    tenant_id,
                    client_id: EntraApplicationClientId::new(Uuid::from_u128(4)),
                },
                session: CloudTerrastodonAccountSession::Unprovisioned,
            });
        profile.validate()?;
        assert_eq!(
            profile.select_source(AuthSource::Auto)?,
            CloudTerrastodonCredentialSource::AzureCli
        );
        assert_eq!(
            profile.select_source(AuthSource::Browser)?,
            CloudTerrastodonCredentialSource::Browser
        );
        profile.verify_principal(principal)?;
        assert!(
            profile
                .verify_principal(CloudTerrastodonAccountPrincipal::EntraUser(
                    EntraUserId::new(Uuid::from_u128(5))
                ))
                .is_err()
        );
        assert_eq!(profile.principal, Some(principal));
        assert!(profile.select_source(AuthSource::WorkloadIdentity).is_err());
        assert!(profile.select_source(AuthSource::None).is_err());
        profile.preferred_source = CloudTerrastodonCredentialSource::Browser;
        assert_eq!(
            profile.select_source(AuthSource::Auto)?,
            CloudTerrastodonCredentialSource::Browser
        );
        assert_eq!(
            profile.select_source(AuthSource::AzureCli)?,
            CloudTerrastodonCredentialSource::AzureCli
        );
        profile.credentials[1].credential = CloudTerrastodonAccountCredential::Browser {
            tenant_id: AzureTenantId::new(Uuid::from_u128(6)),
            client_id: EntraApplicationClientId::new(Uuid::from_u128(4)),
        };
        assert!(profile.validate().is_err());
        Ok(())
    }
}
