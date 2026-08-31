use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RoleDefinition;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

/// Fetches all AzureRM role definitions.
///
/// Not to be confused with Entra role definitions.
#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct RoleDefinitionListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for RoleDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_all_role_definitions<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> RoleDefinitionListRequest<'a> {
    RoleDefinitionListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheableCommand for RoleDefinitionListRequest<'a> {
    type Output = Vec<RoleDefinition>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role-definitions",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching role definitions");
        let helper = ResourceGraphHelper::new(
            self.tenant_id,
            r#"authorizationresources
| where type =~ "microsoft.authorization/roledefinitions"
| project id, properties
| extend
    assignable_scopes = properties.assignableScopes,
    description = properties.description,
    permissions = properties.permissions,
    display_name = properties.roleName,
    ['kind'] = properties.type
| project-away properties"#,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        let mut helper = helper;
        let role_definitions = helper.collect_all().await?;
        debug!("Found {} role definitions", role_definitions.len());
        Ok(role_definitions)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(RoleDefinitionListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::RolePermissionAction;
    use cloud_terrastodon_azure_types::Scope;
    use eyre::ContextCompat;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let auth_context = AuthContext::default();
        let results =
            fetch_all_role_definitions(get_test_tenant_id().await?, &auth_context).await?;
        assert!(!results.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn key_vaults() -> Result<()> {
        let auth_context = AuthContext::default();
        let role_definitions =
            fetch_all_role_definitions(get_test_tenant_id().await?, &auth_context).await?;
        let key_vault_secrets_officer_id = "b86a8fe4-44ce-4948-aee5-eccb2c155cd7";
        let key_vault_secrets_officer = role_definitions
            .iter()
            .find(|rd| rd.id.short_form() == key_vault_secrets_officer_id)
            .wrap_err("Couldn't find Key Vault Secrets Officer role definition")?;
        let permission = "Microsoft.KeyVault/vaults/secrets/readMetadata/action";
        assert!(key_vault_secrets_officer.satisfies(&[], &[RolePermissionAction::new(permission)]));
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(RoleDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(RoleDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(RoleDefinitionListRequest<'static> => Vec<RoleDefinition>);
