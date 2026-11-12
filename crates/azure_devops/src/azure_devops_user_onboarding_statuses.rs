use crate::AzureDevOpsUserLicenseEntitlementListRequest;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use std::borrow::Cow;
use std::collections::HashSet;

pub enum AzureDevOpsUserOnboardingStatus {
    NotOnboarded,
    Onboarded,
}

pub async fn get_azure_devops_user_onboarding_statuses<T: AsRef<str>>(
    org_url: &AzureDevOpsOrganizationUrl,
    user_emails: impl IntoIterator<Item = T>,
    auth_selection: AzureDevOpsAuthSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Vec<(T, AzureDevOpsUserOnboardingStatus)>> {
    let entitlements = AzureDevOpsUserLicenseEntitlementListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_selection,
    }
    .execute(executor)
    .await?;
    let existing_users = entitlements
        .iter()
        .map(|user| user.user.unique_name.as_str())
        .collect::<HashSet<_>>();

    let mut rtn = Vec::new();
    for user in user_emails {
        executor.cancellation().bail_if_cancelled()?;
        let status = match existing_users.contains(user.as_ref()) {
            true => AzureDevOpsUserOnboardingStatus::Onboarded,
            false => AzureDevOpsUserOnboardingStatus::NotOnboarded,
        };
        rtn.push((user, status));
    }
    Ok(rtn)
}
