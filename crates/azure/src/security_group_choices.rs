use crate::fetch_all_security_groups;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_user_input::Choice;
use itertools::Itertools;

pub async fn get_security_group_choices(
    auth_context: &AzureTenantAuthContext,
) -> eyre::Result<Vec<Choice<EntraGroup>>> {
    let security_groups = fetch_all_security_groups(auth_context).await?;
    let choices = security_groups
        .into_iter()
        .sorted_by(|x, y| x.display_name.cmp(&y.display_name))
        .map(|u| Choice {
            key: format!("{} {}", u.id, u.display_name),
            value: u,
        })
        .collect_vec();
    Ok(choices)
}
