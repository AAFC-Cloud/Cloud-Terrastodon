// https://learn.microsoft.com/en-us/graph/api/resources/oauth2permissiongrant?view=graph-rest-1.0
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraServicePrincipalId;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_azure_types::OAuth2PermissionGrant;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;

pub static FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from_iter(["ms", "graph", "GET", "oauth2PermissionGrants"]));

pub(crate) async fn bust_oauth2_permission_grants_cache(
    tenant_id: AzureTenantId,
) -> eyre::Result<()> {
    let mut cache = CommandBuilder::default();
    cache.cache(CacheKey::new(
        FETCH_OAUTH2_PERMISSION_GRANTS_CACHE_DIR.join(tenant_id.to_string()),
    ));
    cache.bust_cache().await?;
    Ok(())
}

pub fn split_oauth2_permission_grant_scope(scope: &str) -> Vec<String> {
    scope
        .split_ascii_whitespace()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn join_oauth2_permission_grant_scopes<'a>(
    scopes: impl IntoIterator<Item = &'a str>,
) -> String {
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();
    for scope in scopes {
        let scope = scope.trim();
        if scope.is_empty() || !seen.insert(scope.to_string()) {
            continue;
        }
        ordered.push(scope.to_string());
    }
    ordered.join(" ")
}

pub fn merge_oauth2_permission_grant_scopes<'a>(
    existing: &str,
    add_scopes: impl IntoIterator<Item = &'a str>,
    remove_scopes: impl IntoIterator<Item = &'a str>,
) -> String {
    let remove = remove_scopes
        .into_iter()
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();

    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for scope in split_oauth2_permission_grant_scope(existing) {
        if remove.contains(&scope) || !seen.insert(scope.clone()) {
            continue;
        }
        merged.push(scope);
    }

    for scope in add_scopes {
        let scope = scope.trim();
        if scope.is_empty() || remove.contains(scope) || !seen.insert(scope.to_string()) {
            continue;
        }
        merged.push(scope.to_string());
    }

    merged.join(" ")
}

pub fn find_matching_oauth2_permission_grant(
    grants: &mut [OAuth2PermissionGrant],
    resource_id: EntraServicePrincipalId,
    client_id: EntraServicePrincipalId,
    principal_id: EntraUserId,
) -> Option<&mut OAuth2PermissionGrant> {
    grants.iter_mut().find(|grant| {
        grant.resource_id == resource_id
            && grant.client_id == client_id
            && grant.principal_id == Some(principal_id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_oauth2_permission_grants;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let found = fetch_oauth2_permission_grants(get_test_tenant_id().await?).await?;
        assert!(!found.is_empty());
        Ok(())
    }

    #[test]
    fn merge_scopes_preserves_existing_order_and_appends_new_values() {
        let merged = merge_oauth2_permission_grant_scopes(
            "Calendars.Read openid profile",
            ["RoleManagement.ReadWrite.Directory", "profile"],
            std::iter::empty(),
        );
        assert_eq!(
            merged,
            "Calendars.Read openid profile RoleManagement.ReadWrite.Directory"
        );
    }

    #[test]
    fn merge_scopes_removes_values_before_appending() {
        let merged = merge_oauth2_permission_grant_scopes(
            "Calendars.Read openid profile",
            ["RoleManagement.ReadWrite.Directory"],
            ["openid"],
        );
        assert_eq!(
            merged,
            "Calendars.Read profile RoleManagement.ReadWrite.Directory"
        );
    }
}
