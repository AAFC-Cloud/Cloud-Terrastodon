use crate::management_groups::fetch_root_management_group;
use crate::resource_groups::fetch_all_resource_groups;
use cloud_terrastodon_azure_types::AsScope;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EligibleChildResource;
use cloud_terrastodon_azure_types::EligibleChildResourceKind;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Eq, PartialEq, Debug, Default)]
pub enum FetchChildrenBehaviour {
    #[default]
    Unspecified,
    GetAllChildren,
}

// https://learn.microsoft.com/en-us/rest/api/authorization/eligible-child-resources/get?view=rest-authorization-2020-10-01&tabs=HTTP
pub async fn fetch_eligible_child_resources(
    scope: &impl Scope,
    behaviour: FetchChildrenBehaviour,
) -> Result<Vec<EligibleChildResource>> {
    let scope = scope.expanded_form();
    let scope = scope.strip_prefix('/').unwrap_or(&scope);
    let mut url = format!(
        "https://management.azure.com/{scope}/providers/Microsoft.Authorization/eligibleChildResources?api-version=2020-10-01"
    );
    if behaviour == FetchChildrenBehaviour::GetAllChildren {
        url.push_str("&getAllChildren=true");
    }
    let mut cache_chunks = PathBuf::from_iter(["az", "rest", "GET", "eligibleChildResources"]);
    scope
        .split("/")
        .filter(|x| !x.is_empty())
        .for_each(|x| cache_chunks.push(x));
    let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.cache(CacheKey::new(cache_chunks));

    #[derive(Deserialize)]
    struct Response {
        value: Vec<EligibleChildResource>,
    }

    let resp: Response = cmd.run().await?;
    Ok(resp.value)
}

#[must_use = "This is a future request, you must .await it"]
pub struct EligibleChildResourceListRequest {
    tenant_id: AzureTenantId,
}

pub fn fetch_all_eligible_resource_containers(
    tenant_id: AzureTenantId,
) -> EligibleChildResourceListRequest {
    EligibleChildResourceListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for EligibleChildResourceListRequest {
    type Output = Vec<EligibleChildResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "management",
            "eligible_child_resources",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let root_mg = fetch_root_management_group(self.tenant_id).await?;
        let scope = root_mg.as_scope();
        let mut resource_containers =
            fetch_eligible_child_resources(scope, FetchChildrenBehaviour::GetAllChildren).await?;
        // this contains management groups and subscriptions

        let rgs = fetch_all_resource_groups(self.tenant_id)
            .await?
            .into_iter()
            .map(|x| EligibleChildResource {
                name: x.name.to_string(),
                kind: EligibleChildResourceKind::ResourceGroup,
                id: x.as_scope().as_scope_impl(),
            });
        resource_containers.extend(rgs);
        // extend to include resource groups

        Ok(resource_containers)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EligibleChildResourceListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use crate::management_groups::fetch_root_management_group;
    use crate::subscriptions::fetch_all_subscriptions;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_azure_types::AsScope;
    use cloud_terrastodon_azure_types::Scope;
    use cloud_terrastodon_user_input::Choice;
    use cloud_terrastodon_user_input::PickerTui;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let mg = fetch_root_management_group(get_test_tenant_id().await?).await?;
        let scope = mg.as_scope();
        let Some(found) = expect_aad_premium_p2_license(
            fetch_eligible_child_resources(scope, FetchChildrenBehaviour::GetAllChildren).await,
        )
        .await?
        else {
            return Ok(());
        };
        assert!(!found.is_empty());
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn it_works2() -> Result<()> {
        let Some(found) = expect_aad_premium_p2_license(
            fetch_all_eligible_resource_containers(get_test_tenant_id().await?).await,
        )
        .await?
        else {
            return Ok(());
        };
        assert!(!found.is_empty());
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn it_works3() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let rg = fetch_all_resource_groups(tenant_id)
            .await?
            .into_iter()
            .next()
            .unwrap();
        let result = expect_aad_premium_p2_license(
            fetch_eligible_child_resources(rg.as_scope(), FetchChildrenBehaviour::GetAllChildren)
                .await,
        )
        .await;
        match result {
            Ok(_) => eyre::bail!(
                "Expected error, but got success; GetAllChildren is only supported for management groups?"
            ),
            Err(_) => return Ok(()),
        }
    }

    #[test_log::test(tokio::test)]
    async fn it_works4() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let subs = fetch_all_subscriptions(tenant_id).await?;
        let sub = subs.first().unwrap();
        let result = expect_aad_premium_p2_license(
            fetch_eligible_child_resources(sub.as_scope(), FetchChildrenBehaviour::GetAllChildren)
                .await,
        )
        .await;
        match result {
            Ok(_) => eyre::bail!(
                "Expected error, but got success; GetAllChildren is only supported for management groups?"
            ),
            Err(_) => return Ok(()),
        }
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works_interactive() -> Result<()> {
        let mg = fetch_root_management_group(get_test_tenant_id().await?).await?;
        let mut scope = mg.as_scope().as_scope_impl().to_owned();
        loop {
            let Some(resources) = expect_aad_premium_p2_license(
                fetch_eligible_child_resources(&scope, FetchChildrenBehaviour::default()).await,
            )
            .await?
            else {
                return Ok(());
            };
            let next_scope: EligibleChildResource =
                PickerTui::new().set_header("Choose a scope").pick_one(
                    resources.into_iter().map(|x| Choice {
                        key: x.name.to_owned(),
                        value: x,
                    }),
                )?;
            scope = next_scope.id;
        }
    }
    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works_interactive2() -> Result<()> {
        let Some(resources) = expect_aad_premium_p2_license(
            fetch_all_eligible_resource_containers(get_test_tenant_id().await?).await,
        )
        .await?
        else {
            return Ok(());
        };
        let chosen: Vec<EligibleChildResource> =
            PickerTui::new().pick_many(resources.into_iter().map(|x| Choice {
                key: x.to_string(),
                value: x,
            }))?;
        assert!(!chosen.is_empty());
        Ok(())
    }
}
