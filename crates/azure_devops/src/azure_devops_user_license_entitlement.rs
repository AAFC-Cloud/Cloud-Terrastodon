use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::async_trait;
use eyre::bail;
use std::pin::Pin;

use crate::fetch_azure_devops_user_license_entitlements;

pub struct AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub user: AzureDevOpsUserArgument<'a>,
    pub invalidate_cache: bool,
}

pub fn fetch_azure_devops_user_license_entitlement<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    user: impl Into<AzureDevOpsUserArgument<'a>>,
) -> AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    AzureDevOpsUserLicenseEntitlementShowRequest {
        org_url,
        user: user.into(),
        invalidate_cache: false,
    }
}

#[async_trait]
impl<'a> CacheInvalidatable for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    async fn invalidate(&self) -> eyre::Result<()> {
        fetch_azure_devops_user_license_entitlements(&self.org_url).invalidate().await
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    type WithInvalidation = Self;
    fn with_invalidation(mut self, invalidate_cache: bool) -> Self {
        self.invalidate_cache = invalidate_cache;
        self
    }
}

impl<'a> IntoFuture for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    type Output = eyre::Result<AzureDevOpsUserLicenseEntitlement>;

    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let entitlements = fetch_azure_devops_user_license_entitlements(&self.org_url)
                .with_invalidation(self.invalidate_cache)
                .await?;

            match entitlements
                .into_iter()
                .find(|license_entitlement| self.user.matches(&license_entitlement.user))
            {
                Some(found) => Ok(found),
                None => bail!(
                    "No license entitlement found for user {:?} in organization {:?}",
                    self.user,
                    self.org_url
                ),
            }
        })
    }
}
