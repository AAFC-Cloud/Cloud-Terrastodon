use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureRoleDefinitionArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::create_role_assignment;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_azure::fetch_all_role_definitions;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use itertools::Itertools;
use std::io::Write;
use tracing::info;

/// Create Azure role assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleAssignmentCreateArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Principal (object id or userPrincipalName)
    #[facet(figue::named)]
    pub principal: Option<AzurePrincipalArgument<'static>>,

    /// Role definition (display name, GUID, or full role definition id)
    #[facet(figue::named)]
    pub role_definition: Option<AzureRoleDefinitionArgument<'static>>,

    /// Scope (resource id, subscription, resource group, etc.)
    #[facet(figue::named)]
    pub scope: Option<ScopeImpl>,
}

#[derive(facet::Facet)]
struct RoleAssignmentCreateResult {
    scope: String,
    role: String,
    principal: String,
    assignment_id: cloud_terrastodon_azure::RoleAssignmentId,
}

impl AzureRoleAssignmentCreateArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Preparing to create role assignment");
        let tenant_id = self.tenant.resolve().await?;

        // Resolve role definitions
        let role_defs = if let Some(role_arg) = self.role_definition {
            // If caller provided an argument, fetch list to resolve names if needed
            let all = fetch_all_role_definitions(tenant_id).await?;
            let matched: Vec<_> = all.into_iter().filter(|r| role_arg.matches(r)).collect();
            if matched.is_empty() {
                eyre::bail!("No role definitions matched '{role_arg}'");
            }
            matched
        } else {
            info!("Fetching role definitions for interactive pick");
            let all = fetch_all_role_definitions(tenant_id).await?;
            PickerTui::<_>::new()
                .set_header("Roles to assign")
                .pick_many(all.into_iter().map(|r| Choice {
                    key: r.display_name.clone(),
                    value: r,
                })).await?
        };

        // Resolve principals
        let principals = if let Some(principal_arg) = self.principal {
            let fetched = fetch_all_principals(tenant_id).await?;
            let matched: Vec<_> = fetched
                .0
                .into_values()
                .filter(|p| principal_arg.matches(p))
                .collect();
            if matched.is_empty() {
                eyre::bail!("No principals matched '{principal_arg}'");
            }
            matched
        } else {
            info!("Fetching principals for interactive pick");
            let fetched = fetch_all_principals(tenant_id).await?;
            PickerTui::<_>::new()
                .set_header("Principals to assign")
                .pick_many(fetched.values().map(|u| Choice {
                    key: format!("{} {:64} {}", u.id(), u.display_name(), u.name()),
                    value: u.clone(),
                })).await?
        };

        // Resolve scopes
        let scopes = if let Some(scope) = self.scope {
            vec![scope]
        } else {
            info!("Fetching resources for interactive pick");
            let resources = fetch_all_resources(tenant_id).await?;
            PickerTui::<_>::new()
                .set_header(format!(
                    "Assigning {} to {}",
                    role_defs.iter().map(|r| &r.display_name).join(", "),
                    principals.iter().map(|p| p.display_name()).join(", ")
                ))
                .pick_many(resources.into_iter().map(|resource| Choice {
                    key: resource.id.to_string(),
                    value: resource.id,
                })).await?
        };

        // Create assignments for each combination
        let mut results = Vec::new();
        for scope in scopes {
            for role in &role_defs {
                for principal in &principals {
                    info!(
                        "Assigning {} to {} on {}",
                        role.display_name,
                        principal.name(),
                        scope.expanded_form()
                    );
                    let id = create_role_assignment(&scope, &role.id, principal.as_ref()).await?;
                    results.push(RoleAssignmentCreateResult {
                        scope: scope.expanded_form(),
                        role: role.display_name.clone(),
                        principal: principal.display_name().to_owned(),
                        assignment_id: id,
                    });
                }
            }
        }

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &results)?;
        handle.write_all(b"\n")?;

        info!("Successfully created {} role assignments", results.len());
        Ok(())
    }
}
