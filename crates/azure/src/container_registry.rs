use cloud_terrastodon_azure_types::prelude::ContainerRegistry;
use cloud_terrastodon_azure_types::prelude::ContainerRegistryId;
use cloud_terrastodon_azure_types::prelude::ContainerRegistryRepositoryName;
use cloud_terrastodon_azure_types::prelude::ContainerRegistryRepositoryTag;
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

pub async fn fetch_container_registry_repository_names(
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
        path: PathBuf::from_iter([
            "container_registry_repositories",
            &registry_id.container_registry_name,
        ]),
        valid_for: Duration::from_hours(8),
    });
    Ok(cmd.run().await?)
}

pub async fn fetch_container_registry_repository_tags(
    registry_id: &ContainerRegistryId,
    repository_name: &ContainerRegistryRepositoryName,
) -> Result<Vec<ContainerRegistryRepositoryTag>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "acr",
        "repository",
        "show-tags",
        "--detail",
        "--name",
        &registry_id.container_registry_name,
        "--repository",
        &repository_name,
        "--subscription",
        &registry_id.resource_group_id.subscription_id.short_form(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "container_registry_repository_tags",
            &registry_id.container_registry_name,
            &repository_name,
        ]),
        valid_for: Duration::from_hours(8),
    });
    Ok(cmd.run().await?)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_container_registries;
    use crate::prelude::fetch_container_registry_repository_names;
    use crate::prelude::fetch_container_registry_repository_tags;
    use cloud_terrastodon_azure_types::prelude::Scope;
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
        println!("Fetching container registries...");
        let mut pass = false;
        let found = fetch_all_container_registries().await?;
        let found_count = found.len();
        for (i, container_registry) in found.into_iter().enumerate() {
            let repository_names =
                fetch_container_registry_repository_names(&container_registry.id).await?;
            println!(
                "[{}/{found_count}] Found {} repositories for {}",
                i + 1,
                repository_names.len(),
                container_registry.id.short_form()
            );

            let found = repository_names.iter();
            let found_count = found.len();
            for (i, repository) in found.enumerate() {
                println!("  - [{}/{found_count}] {}", i + 1, repository);
                repository.validate()?;
                let tags =
                    fetch_container_registry_repository_tags(&container_registry.id, repository)
                        .await?;
                println!("    Found {} tags", tags.len());
                for tag in tags.iter() {
                    println!(
                        "      - {} updated at {}",
                        tag.name,
                        tag.last_update_time.naive_local()
                    );
                    pass = true;
                }
                // comment this out to display all tags
                if !tags.is_empty() {
                    return Ok(());
                }
            }
        }
        if !pass {
            eyre::bail!("No container registries with tags found.");
        }
        Ok(())
    }
}
