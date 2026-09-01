use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::ContainerRegistry;
use cloud_terrastodon_azure_types::ContainerRegistryId;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryName;
use cloud_terrastodon_azure_types::ContainerRegistryRepositoryTag;
use cloud_terrastodon_azure_types::HasSlug;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct ContainerRegistryListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_container_registries<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> ContainerRegistryListRequest<'a> {
    ContainerRegistryListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for ContainerRegistryListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for ContainerRegistryListRequest<'a> {
    type Output = Vec<ContainerRegistry>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "container_registries",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut query = ResourceGraphHelper::new(
            r#"
Resources
| where type =~ "Microsoft.ContainerRegistry/registries"
        "#,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        query.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ContainerRegistryListRequest<'a>, 'a);

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
        registry_id.resource_group_id.name(),
        "--subscription",
        &registry_id.resource_group_id.subscription_id.short_form(),
        "--output",
        "json",
    ]);
    cmd.cache(CacheKey::new(PathBuf::from_iter([
        "container_registry_repositories",
        &registry_id.container_registry_name,
    ])));
    cmd.run().await
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
        repository_name,
        "--subscription",
        &registry_id.resource_group_id.subscription_id.short_form(),
        "--output",
        "json",
    ]);
    cmd.cache(CacheKey::new(PathBuf::from_iter([
        "container_registry_repository_tags",
        &registry_id.container_registry_name,
        repository_name,
    ])));
    cmd.run().await
}

#[cfg(test)]
mod test {
    use crate::fetch_all_container_registries;
    use crate::fetch_container_registry_repository_names;
    use crate::fetch_container_registry_repository_tags;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::Slug;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_container_registries(
            &AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?,
        )
        .await?;
        assert!(!found.is_empty());
        for registry in found.into_iter() {
            registry.name.validate_slug()?;
        }
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    pub async fn it_works2() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let mut pass = false;
        let auth_context = AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
        let found = fetch_all_container_registries(&auth_context).await?;
        let found_count = found.len();
        for (i, container_registry) in found.into_iter().enumerate() {
            let repository_names =
                fetch_container_registry_repository_names(&container_registry.id).await?;
            assert!(i < found_count);

            let found = repository_names.iter();
            let found_count = found.len();
            for (i, repository) in found.enumerate() {
                assert!(i < found_count);
                let tags =
                    fetch_container_registry_repository_tags(&container_registry.id, repository)
                        .await?;
                for tag in tags.iter() {
                    assert!(!tag.name.is_empty());
                    pass = true;
                }
                // comment this out to display all tags
                // if !tags.is_empty() {
                //     return Ok(());
                // }
            }
        }
        if !pass {
            eyre::bail!("No container registries with tags found.");
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ContainerRegistryListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ContainerRegistryListRequest<'static> => Vec<ContainerRegistry>);
