use crate::EntraSecurityGroupListRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use itertools::Itertools;
use std::sync::Arc;

pub async fn get_security_group_choices(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Vec<Choice<Arc<EntraGroup>>>> {
    let security_groups = EntraSecurityGroupListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let choices = security_groups
        .iter()
        .cloned()
        .sorted_by(|x, y| x.display_name.cmp(&y.display_name))
        .map(|u| Choice {
            key: format!("{} {}", u.id, u.display_name),
            value: u,
        })
        .collect_vec();
    Ok(choices)
}
