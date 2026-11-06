use crate::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::bail;
use std::borrow::Cow;
use std::sync::Arc;

pub struct AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub user: AzureDevOpsUserArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_user_license_entitlement<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    user: impl Into<AzureDevOpsUserArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsUserLicenseEntitlementShowRequest<'a>> {
    executor.execute(AzureDevOpsUserLicenseEntitlementShowRequest {
        org_url,
        user: user.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}
impl<'a> CacheInvalidationKeys for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![
            crate::AzureDevOpsUserLicenseEntitlementListRequest {
                org_url: Cow::Borrowed(self.org_url),
                auth_context: Cow::Borrowed(self.auth_context.as_ref()),
            }
            .cache_key()?,
        ])
    }
}

impl<'a> ExecutableRequest for AzureDevOpsUserLicenseEntitlementShowRequest<'a> {
    type Error = eyre::Report;
    type Output = Arc<AzureDevOpsUserLicenseEntitlement>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let entitlements = fetch_azure_devops_user_license_entitlements(
                self.org_url,
                self.auth_context.as_ref(),
                &executor,
            )
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
