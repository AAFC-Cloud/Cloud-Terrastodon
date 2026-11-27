use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseEntitlementUpdateResponse;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_command::app_work::Loadable;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

pub(crate) type AzureDevOpsUserEntitlements = Vec<Arc<AzureDevOpsUserLicenseEntitlement>>;

type InventoryKey = (AzureDevOpsOrganizationUrl, AzureDevOpsAuthSelection);

/// User entitlements and license update feedback for the credentials of one tab.
#[derive(Default)]
pub(crate) struct AzureDevOpsUserInventory {
    users: HashMap<InventoryKey, Loadable<AzureDevOpsUserEntitlements>>,
    updates: HashMap<InventoryKey, Loadable<String>>,
}

impl AzureDevOpsUserInventory {
    pub fn users(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
    ) -> &Loadable<AzureDevOpsUserEntitlements> {
        self.users
            .get(&(organization.clone(), authentication))
            .unwrap_or(&Loadable::NotLoaded)
    }

    pub fn update(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
    ) -> Option<&Loadable<String>> {
        self.updates.get(&(organization.clone(), authentication))
    }

    pub fn is_loading(&self) -> bool {
        self.users
            .values()
            .any(|users| matches!(users, Loadable::Loading { .. }))
    }

    pub fn publish_users(
        &mut self,
        organization: AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        result: Loadable<AzureDevOpsUserEntitlements>,
    ) {
        let key = (organization, authentication);
        match &result {
            Loadable::Loading { .. } => {}
            Loadable::Loaded { started_at, .. } | Loadable::Failed { started_at, .. } if matches!(self.users.get(&key), Some(Loadable::Loading { started_at: active }) if active == started_at) =>
                {}
            _ => return,
        }
        // A read that began before a license change may complete after the cache
        // was invalidated. Only the current load can populate the inventory.
        self.users.insert(key, result);
    }

    pub fn fail_to_enqueue_users(
        &mut self,
        organization: AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        started_at: Instant,
        error: eyre::Report,
    ) {
        self.users.insert(
            (organization, authentication),
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }

    pub fn publish_update(
        &mut self,
        organization: AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        result: Loadable<String>,
    ) {
        self.updates.insert((organization, authentication), result);
    }

    pub fn reconcile_license_update(
        &mut self,
        organization: &AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        response: Option<&AzureDevOpsLicenseEntitlementUpdateResponse>,
    ) {
        self.users.retain(|(scope, account), users| {
            if scope != organization {
                return true;
            }
            if *account == authentication
                && let Some(response) = response
                && let Some(users) = users.as_loaded_mut()
                && let Some(user) = users.iter_mut().find(|user| user.id == response.id)
            {
                Arc::make_mut(user).access_level = response.access_level.clone();
                return true;
            }
            false
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidation_discards_reads_started_before_a_license_change() -> eyre::Result<()> {
        let organization: AzureDevOpsOrganizationUrl = "example-org".parse()?;
        let other: AzureDevOpsOrganizationUrl = "different-org".parse()?;
        let authentication = AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: "00000000-0000-0000-0000-000000000001".parse()?,
        };
        let mut inventory = AzureDevOpsUserInventory::default();
        let started_at = Instant::now();
        for scope in [&organization, &other] {
            inventory.publish_users(
                scope.clone(),
                authentication,
                Loadable::Loading { started_at },
            );
        }
        inventory.reconcile_license_update(&organization, authentication, None);
        for scope in [&organization, &other] {
            inventory.publish_users(
                scope.clone(),
                authentication,
                Loadable::Loaded {
                    value: Vec::new(),
                    started_at,
                    finished_at: Instant::now(),
                },
            );
        }
        assert!(matches!(
            inventory.users(&organization, authentication),
            Loadable::NotLoaded
        ));
        assert!(
            inventory
                .users(&other, authentication)
                .as_loaded()
                .is_some()
        );
        Ok(())
    }
}
