use cloud_terrastodon_core_azure::prelude::fetch_all_resources;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use tracing::info;

pub async fn find_resource_owners_menu() -> anyhow::Result<()> {
    info!("Fetching resources");
    let resources = fetch_all_resources().await?;

    let resources = pick_many(FzfArgs {
        choices: resources
            .into_iter()
            .map(|resource| Choice {
                key: format!("{}", resource.id.expanded_form()),
                value: resource,
            })
            .collect_vec(),
        prompt: None,
        header: Some("Pick the resources to find the owners for".to_string()),
    })?;

    info!("You chose:");
    for r in resources.iter() {
        info!("- {}", r.id.expanded_form());
    }


    Ok(())
}
