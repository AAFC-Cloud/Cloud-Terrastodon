use cloud_terrastodon_azure::prelude::RolePermissionAction;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_key_vaults;
use cloud_terrastodon_azure::prelude::fetch_all_role_assignments;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_azure::prelude::fetch_current_user;
use cloud_terrastodon_azure::prelude::fetch_governance_role_assignments_for_principal;
use cloud_terrastodon_azure::prelude::uuid::Uuid;
use std::collections::HashMap;
use tokio::try_join;

#[tokio::test]
pub async fn predict_would_secret_list_succeed() -> eyre::Result<()> {
    let current_user = fetch_current_user().await?;
    let (key_vaults, role_assignments, role_definitions, pim_assignments) = try_join!(
        fetch_all_key_vaults(),
        fetch_all_role_assignments(),
        fetch_all_role_definitions(),
        fetch_governance_role_assignments_for_principal(&current_user.id)
    )?;

    let role_definitions = role_definitions
        .into_iter()
        .map(|role_definition| (role_definition.id.clone(), role_definition))
        .collect::<HashMap<_, _>>();

    let user_role_assignments = role_assignments
        .iter()
        .filter(|ra| ra.principal_id == current_user.id)
        .map(|ra| {
            eyre::Ok((
                ra,
                role_definitions
                    .get(&ra.role_definition_id)
                    .ok_or_else(|| {
                        eyre::eyre!("Role definition not found for role assignment: {:?}", ra)
                    })?,
            ))
        })
        .collect::<eyre::Result<Vec<_>>>()?;

    let list_secrets_permissions: &[RolePermissionAction] = &[RolePermissionAction::new(
        "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
    )];
    for kv in key_vaults {
        let mut can_list_secrets = false;
        let kv_uses_rbac = kv.properties.enable_rbac_authorization.unwrap_or_default();
        if kv_uses_rbac {
            for (ra, role_definition) in &user_role_assignments {
                if ra.scope.expanded_form().starts_with(&kv.id.expanded_form()) {
                    for perm in &role_definition.permissions {
                        if perm.satisfies(&list_secrets_permissions, &[]) {
                            can_list_secrets = true;
                            break;
                        }
                    }
                }
            }
        }
        println!("Key Vault: {} - {}", kv.name, can_list_secrets);
    }

    Ok(())
}
