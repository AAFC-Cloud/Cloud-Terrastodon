use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::UnifiedRoleDefinition;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::borrow::Cow;
use std::path::PathBuf;

/// Fetch an individual Entra role assignment
#[derive(facet::Facet)]
pub struct UnifiedRoleDefinitionRequest<'a> {
    pub role_definition_id: UnifiedRoleDefinitionId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for UnifiedRoleDefinitionRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            role_definition_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_unified_role_definition<'a>(
    role_definition_id: UnifiedRoleDefinitionId,
    auth_context: &'a AzureTenantAuthContext,
) -> UnifiedRoleDefinitionRequest<'a> {
    UnifiedRoleDefinitionRequest {
        role_definition_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for UnifiedRoleDefinitionRequest<'_> {
    type Output = UnifiedRoleDefinition;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_definition",
            self.auth_context.tenant_id.to_string().as_ref(),
            self.role_definition_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let role_definition_id = self.role_definition_id.as_ref();
        let url = format!(
            "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions/{role_definition_id}?$expand=inheritsPermissionsFrom"
        );
        let query = MicrosoftGraphHelper::new(
            url,
            Some(CacheKey::new(PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "unified_role_definition",
                self.auth_context.tenant_id.to_string().as_ref(),
                role_definition_id.to_string().as_ref(),
            ]))),
            self.auth_context.as_ref(),
        );

        let found = query.fetch_one().await?;
        Ok(found)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(UnifiedRoleDefinitionRequest<'a>, 'a);

/// Unravels the [`UnifiedRoleDefinition::inherits_permissions_from`] chain
/// into the top-level [`UnifiedRoleDefinition::role_permissions`]
pub async fn fetch_unified_role_definition_deep(
    role_definition_id: UnifiedRoleDefinitionId,
    auth_context: &AzureTenantAuthContext,
) -> eyre::Result<UnifiedRoleDefinition> {
    let mut this = fetch_unified_role_definition(role_definition_id, auth_context).await?;
    let mut next = std::mem::take(&mut this.inherits_permissions_from);
    while let Some(parent_id) = next.pop() {
        let parent = fetch_unified_role_definition(parent_id.id, auth_context).await?;
        this.inherits_permissions_from.push(parent_id);
        this.role_permissions.extend(parent.role_permissions);
        next.extend(parent.inherits_permissions_from);
    }

    Ok(this)
}

#[cfg(test)]
mod test {
    use crate::fetch_unified_role_definition;
    use crate::fetch_unified_role_definition_deep;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::UnifiedRoleDefinitionId;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works_single() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let application_developer_role_id: UnifiedRoleDefinitionId =
            "cf1c38e5-3621-4004-a7cb-879624dced7c".parse()?;
        let directory_readers_role_id: UnifiedRoleDefinitionId =
            "88d8e3e3-8f55-4a1e-953a-9b9898b8876b".parse()?;
        let auth_context = AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
        let found =
            fetch_unified_role_definition(application_developer_role_id, &auth_context).await?;
        assert!(
            matches!(found.inherits_permissions_from.as_slice(), [x] if x.id == directory_readers_role_id)
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works_single_deep() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let application_developer_role_id: UnifiedRoleDefinitionId =
            "cf1c38e5-3621-4004-a7cb-879624dced7c".parse()?;
        let auth_context = AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
        let found =
            fetch_unified_role_definition_deep(application_developer_role_id, &auth_context)
                .await?;
        assert!(!found.role_permissions.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(UnifiedRoleDefinitionRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleDefinitionRequest<'static>);
cloud_terrastodon_registry::register_into_future!(UnifiedRoleDefinitionRequest<'static> => UnifiedRoleDefinition);
