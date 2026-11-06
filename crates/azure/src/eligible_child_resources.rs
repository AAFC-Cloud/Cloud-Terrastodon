use crate::EligibleChildResourcesAtScopeRequest;
use crate::FetchChildrenBehaviour;
use crate::ManagementGroupListRequest;
use crate::ResourceGroupListRequest;
use crate::management_groups::fetch_root_management_group;
use crate::resource_group_list_request::fetch_all_resource_groups;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AsScope;
use cloud_terrastodon_azure_types::EligibleChildResource;
use cloud_terrastodon_azure_types::EligibleChildResourceKind;
use cloud_terrastodon_azure_types::ManagementGroupId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;

#[track_caller]
pub fn fetch_eligible_child_resources<'a>(
    scope: &impl Scope,
    behaviour: FetchChildrenBehaviour,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EligibleChildResourcesAtScopeRequest<'a>> {
    executor.execute(EligibleChildResourcesAtScopeRequest {
        scope: scope.as_scope_impl(),
        behaviour,
        auth_context: Cow::Borrowed(auth_context),
    })
}

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct EligibleChildResourceListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_eligible_resource_containers<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EligibleChildResourceListRequest<'a>> {
    executor.execute(EligibleChildResourceListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for EligibleChildResourceListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl CacheInvalidationKeys for EligibleChildResourceListRequest<'_> {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        // The root-group lookup and both inventory reads contribute to this composition.
        Ok(vec![
            ManagementGroupListRequest {
                auth_context: self.auth_context.clone(),
            }
            .cache_key()?,
            EligibleChildResourcesAtScopeRequest {
                scope: ManagementGroupId::from_name(&self.auth_context.tenant_id.to_string())
                    .as_scope_impl(),
                behaviour: FetchChildrenBehaviour::GetAllChildren,
                auth_context: self.auth_context.clone(),
            }
            .cache_key()?,
            ResourceGroupListRequest {
                auth_context: self.auth_context.clone(),
            }
            .cache_key()?,
        ])
    }
}

impl<'a> ExecutableRequest for EligibleChildResourceListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EligibleChildResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let root_mg =
                fetch_root_management_group(self.auth_context.as_ref(), &executor).await?;
            let scope = root_mg.as_scope();
            let mut resource_containers = fetch_eligible_child_resources(
                scope,
                FetchChildrenBehaviour::GetAllChildren,
                self.auth_context.as_ref(),
                &executor,
            )
            .await?;
            // this contains management groups and subscriptions

            let rgs = fetch_all_resource_groups(self.auth_context.as_ref(), &executor)
                .await?
                .into_iter()
                .map(|x| {
                    Arc::new(EligibleChildResource {
                        name: x.name.to_string(),
                        kind: EligibleChildResourceKind::ResourceGroup,
                        id: x.as_scope().as_scope_impl(),
                    })
                });
            resource_containers.extend(rgs);
            // extend to include resource groups

            Ok(resource_containers)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use crate::management_groups::fetch_root_management_group;
    use crate::subscriptions::fetch_all_subscriptions;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::AsScope;
    use cloud_terrastodon_azure_types::Scope;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_user_input::Choice;
    use cloud_terrastodon_user_input::PickerTui;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let tenant_auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let mg = fetch_root_management_group(&tenant_auth_context, &executor).await?;
                let scope = mg.as_scope();
                let Some(found) = expect_aad_premium_p2_license(
                    fetch_eligible_child_resources(
                        scope,
                        FetchChildrenBehaviour::GetAllChildren,
                        &tenant_auth_context,
                        &executor,
                    )
                    .await,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_auth_context.tenant_id,
                    &executor,
                )
                .await?
                else {
                    return Ok(());
                };
                assert!(!found.is_empty());
                Ok(())
            })
            .await
    }

    #[test_log::test(tokio::test)]
    async fn it_works2() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli()
                    .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let Some(found) = expect_aad_premium_p2_license(
                    fetch_all_eligible_resource_containers(&auth_context, &executor).await,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    auth_context.tenant_id,
                    &executor,
                )
                .await?
                else {
                    return Ok(());
                };
                assert!(!found.is_empty());
                Ok(())
            })
            .await
    }

    #[test_log::test(tokio::test)]
    async fn it_works3() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(), &cancellation,
                )?;
            let tenant_id = get_test_tenant_id(&cancellation).await?;
            let auth_context = AuthContext::explicit_azure_cli();
            let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
            let rg = fetch_all_resource_groups(&tenant_auth_context, &executor)
                .await?
                .into_iter()
                .next()
                .unwrap();
            let result = expect_aad_premium_p2_license(
                fetch_eligible_child_resources(
                    rg.as_scope(),
                    FetchChildrenBehaviour::GetAllChildren,
                    &tenant_auth_context,
                    &executor,
                )
                .await, cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli, tenant_id, &executor)
            .await;
            match result {
                Ok(_) => {
                    eyre::bail!(
                        "Expected error, but got success; GetAllChildren is only supported for management groups?"
                    );
                }
                Err(_) => Ok(()),
            }
            })
            .await
    }

    #[test_log::test(tokio::test)]
    async fn it_works4() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(), &cancellation,
                )?;
            let tenant_id = get_test_tenant_id(&cancellation).await?;
            let auth_context = AuthContext::explicit_azure_cli();
            let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
            let subs = fetch_all_subscriptions(&tenant_auth_context, &executor).await?;
            let sub = subs.first().unwrap();
            let result = expect_aad_premium_p2_license(
                fetch_eligible_child_resources(
                    sub.as_scope(),
                    FetchChildrenBehaviour::GetAllChildren,
                    &tenant_auth_context,
                    &executor,
                )
                .await, cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli, tenant_id, &executor)
            .await;
            match result {
                Ok(_) => {
                    eyre::bail!(
                        "Expected error, but got success; GetAllChildren is only supported for management groups?"
                    );
                }
                Err(_) => Ok(()),
            }
            })
            .await
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works_interactive() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let tenant_auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let mg = fetch_root_management_group(&tenant_auth_context, &executor).await?;
                let mut scope = mg.as_scope().as_scope_impl().to_owned();
                loop {
                    let Some(resources) = expect_aad_premium_p2_license(
                        fetch_eligible_child_resources(
                            &scope,
                            FetchChildrenBehaviour::default(),
                            &tenant_auth_context,
                            &executor,
                        )
                        .await,
                        cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                        tenant_auth_context.tenant_id,
                        &executor,
                    )
                    .await?
                    else {
                        return Ok(());
                    };
                    let next_scope: Arc<EligibleChildResource> = PickerTui::<_>::new()
                        .set_header("Choose a scope")
                        .pick_one(resources.into_iter().map(|x| Choice {
                            key: x.name.to_owned(),
                            value: x,
                        }))
                        .await?;
                    scope = next_scope.id.clone();
                }
            })
            .await
    }
    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works_interactive2() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli()
                    .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let Some(resources) = expect_aad_premium_p2_license(
                    fetch_all_eligible_resource_containers(&auth_context, &executor).await,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    auth_context.tenant_id,
                    &executor,
                )
                .await?
                else {
                    return Ok(());
                };
                let chosen: Vec<Arc<EligibleChildResource>> = PickerTui::<_>::new()
                    .pick_many(resources.into_iter().map(|x| Choice {
                        key: x.to_string(),
                        value: x,
                    }))
                    .await?;
                assert!(!chosen.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(EligibleChildResourceListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EligibleChildResourceListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(EligibleChildResourceListRequest<'static> => Vec<Arc<EligibleChildResource>>, effects = [Read]);
