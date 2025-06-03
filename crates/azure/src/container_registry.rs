use cloud_terrastodon_azure_types::prelude::ContainerRegistry;
use cloud_terrastodon_azure_types::prelude::ContainerRegistryId;
use cloud_terrastodon_azure_types::prelude::ContainerRegistryRepositoryName;
use cloud_terrastodon_azure_types::prelude::HasSlug;
use cloud_terrastodon_azure_types::prelude::Scope;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

use crate::prelude::ResourceGraphHelper;

pub async fn fetch_all_container_registries() -> Result<Vec<ContainerRegistry>> {
    let mut query = ResourceGraphHelper::new(
        r#"
Resources
| where type =~ "Microsoft.ContainerRegistry/registries"
        "#,
        CacheBehaviour::Some {
            path: PathBuf::from("container_registries"),
            valid_for: Duration::from_hours(8),
        },
    );
    query.collect_all().await
}

pub async fn fetch_all_repositories_for_container_registry(
    registry_id: &ContainerRegistryId,
) -> Result<Vec<ContainerRegistryRepositoryName>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "acr",
        "repository",
        "list",
        "--name",
        &registry_id.container_registry_name,
        "--resource-group",
        &registry_id.resource_group_id.name(),
        "--subscription",
        &registry_id.resource_group_id.subscription_id.short_form(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["container_registry_repositories", &registry_id.container_registry_name]),
        valid_for: Duration::from_hours(8),
    });
    Ok(cmd.run().await?)
}

#[cfg(test)]
mod test {
    use crate::prelude::{fetch_all_container_registries, fetch_all_repositories_for_container_registry};
    use cloud_terrastodon_azure_types::prelude::{ContainerRegistryId, Scope};
    use validator::Validate;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_container_registries().await?;
        for registry in found.into_iter() {
            println!("{}", registry.id.expanded_form());
            registry.name.validate()?;
        }
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works2() -> eyre::Result<()> {
        let container_registry_id = ContainerRegistryId::try_from_expanded(
            todo!("just iterate over registries until one has repositories"),
        )?;
        let repositories = fetch_all_repositories_for_container_registry(&container_registry_id).await?;

        println!("{:?}", repositories);
        Ok(())
    }
}
