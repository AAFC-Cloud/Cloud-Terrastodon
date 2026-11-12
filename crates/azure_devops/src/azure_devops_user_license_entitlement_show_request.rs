use crate::AzureDevOpsUserLicenseEntitlementListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;

/// Selects one user's license entitlement from the organization inventory.
/// See [User Entitlements](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/list?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub user: AzureDevOpsUserArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementShowRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            user: AzureDevOpsUserArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}

impl<'a> CacheInvalidationKeys for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            crate::AzureDevOpsUserLicenseEntitlementListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .cache_key()?,
        ])
    }
}

impl<'a> ExecutableRequest for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = Arc<AzureDevOpsUserLicenseEntitlement>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let entitlements = AzureDevOpsUserLicenseEntitlementListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .execute(&executor)
            .await?;

            match entitlements
                .into_iter()
                .find(|license_entitlement| self.user.matches(&license_entitlement.user))
            {
                Some(found) => Ok(found),
                None => {
                    bail!(
                        "No license entitlement found for user {:?} in organization {:?}",
                        self.user,
                        self.org_url
                    );
                }
            }
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsUserLicenseEntitlementShowRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsUserLicenseEntitlementShowRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsUserLicenseEntitlementShowRequest<'static> => Arc<AzureDevOpsUserLicenseEntitlement>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsUserLicenseEntitlementShowRequest<'static>,
    name = "azure-devops-user-license-entitlement-show",
    cached
);
