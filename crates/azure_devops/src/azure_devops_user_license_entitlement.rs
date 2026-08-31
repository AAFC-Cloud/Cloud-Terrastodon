use crate::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::bail;
use std::borrow::Cow;
use std::pin::Pin;

pub struct AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub user: AzureDevOpsUserArgument<'a>,
    pub invalidate_cache: bool,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_azure_devops_user_license_entitlement<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    user: impl Into<AzureDevOpsUserArgument<'a>>,
    auth_context: &'a AuthContext,
) -> AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    AzureDevOpsUserLicenseEntitlementShowRequest {
        org_url,
        user: user.into(),
        invalidate_cache: false,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheInvalidatable for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    async fn invalidate(&self) -> eyre::Result<()> {
        fetch_azure_devops_user_license_entitlements(self.org_url, self.auth_context.as_ref())
            .invalidate()
            .await
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
            let entitlements = fetch_azure_devops_user_license_entitlements(
                self.org_url,
                self.auth_context.as_ref(),
            )
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
