use crate::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use std::collections::HashSet;

pub enum UserOnboardingStatus {
    NotOnboarded,
    Onboarded,
}

pub async fn get_azure_devops_user_onboarding_statuses<T: AsRef<str>>(
    org_url: &AzureDevOpsOrganizationUrl,
    user_emails: impl IntoIterator<Item = T>,
    auth_context: &AzureDevOpsAuthContext,
    cancellation: &CancellationScope,
) -> eyre::Result<Vec<(T, UserOnboardingStatus)>> {
    let existing_users =
        fetch_azure_devops_user_license_entitlements(org_url, auth_context, &cancellation)
            .await?
            .into_iter()
            .map(|user| user.user.unique_name)
            .collect::<HashSet<_>>();

    let mut rtn = Vec::new();
    for user in user_emails {
        cancellation.bail_if_cancelled()?;
        let status = match existing_users.contains(user.as_ref()) {
            true => UserOnboardingStatus::Onboarded,
            false => UserOnboardingStatus::NotOnboarded,
        };
        rtn.push((user, status));
    }
    Ok(rtn)
}
