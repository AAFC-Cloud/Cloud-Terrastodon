use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::AzureRoleAssignmentCreateRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionArgument;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::RoleAssignmentId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::uuid::Uuid;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use itertools::Itertools;
use std::borrow::Cow;
use tracing::info;

/// Create Azure role assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleAssignmentCreateArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

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

#[derive(Facet)]
struct RoleAssignmentCreateResult {
    scope: String,
    role: String,
    principal: String,
    assignment_id: cloud_terrastodon_azure::RoleAssignmentId,
}

impl AzureRoleAssignmentCreateArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        info!("Preparing to create role assignment");
        let authentication = self.authentication.resolve(executor).await?;

        // Resolve role definitions
        let role_defs = if let Some(role_arg) = self.role_definition {
            // If caller provided an argument, fetch list to resolve names if needed
            let all = AzureRoleDefinitionListRequest { authentication }
                .execute(executor)
                .await?;
            let matched: Vec<_> = all
                .iter()
                .filter(|r| role_arg.matches(r))
                .cloned()
                .collect();
            if matched.is_empty() {
                eyre::bail!("No role definitions matched '{role_arg}'");
            }
            matched
        } else {
            info!("Fetching role definitions for interactive pick");
            let all = AzureRoleDefinitionListRequest { authentication }
                .execute(executor)
                .await?;
            PickerTui::<_>::new()
                .set_header("Roles to assign")
                .pick_many(
                    all.iter().cloned().map(|r| Choice {
                        key: r.display_name.clone(),
                        value: r,
                    }),
                    executor.cancellation(),
                )
                .await?
        };

        // Resolve principals
        let principals = if let Some(principal_arg) = self.principal {
            let fetched = EntraPrincipalListRequest { authentication }
                .execute(executor)
                .await?;
            let matched: Vec<_> = fetched
                .values()
                .filter(|p| principal_arg.matches(*p))
                .cloned()
                .collect();
            if matched.is_empty() {
                eyre::bail!("No principals matched '{principal_arg}'");
            }
            matched
        } else {
            info!("Fetching principals for interactive pick");
            let fetched = EntraPrincipalListRequest { authentication }
                .execute(executor)
                .await?;
            PickerTui::<_>::new()
                .set_header("Principals to assign")
                .pick_many(
                    fetched.values().map(|u| Choice {
                        key: format!("{} {:64} {}", u.id(), u.display_name(), u.name()),
                        value: u.clone(),
                    }),
                    executor.cancellation(),
                )
                .await?
        };

        // Resolve scopes
        let scopes = if let Some(scope) = self.scope {
            vec![scope]
        } else {
            info!("Fetching resources for interactive pick");
            let resources = AzureResourceListRequest { authentication }
                .execute(executor)
                .await?;
            PickerTui::<_>::new()
                .set_header(format!(
                    "Assigning {} to {}",
                    role_defs.iter().map(|r| &r.display_name).join(", "),
                    principals.iter().map(|p| p.display_name()).join(", ")
                ))
                .pick_many(
                    resources.into_iter().map(|resource| Choice {
                        key: resource.id.to_string(),
                        value: resource.id.clone(),
                    }),
                    executor.cancellation(),
                )
                .await?
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
                    let assignment_id: RoleAssignmentId = format!(
                        "{}/providers/Microsoft.Authorization/roleAssignments/{}",
                        scope.expanded_form().trim_end_matches('/'),
                        Uuid::new_v4(),
                    )
                    .parse()?;
                    let id = AzureRoleAssignmentCreateRequest {
                        authentication,
                        id: Cow::Owned(assignment_id),
                        role_definition_id: Cow::Borrowed(&role.id),
                        principal_id: principal.id(),
                    }
                    .execute(executor)
                    .await?;
                    results.push(RoleAssignmentCreateResult {
                        scope: scope.expanded_form(),
                        role: role.display_name.clone(),
                        principal: principal.display_name().to_owned(),
                        assignment_id: id,
                    });
                }
            }
        }

        info!("Successfully created {} role assignments", results.len());
        Ok(CliOutput::facet(results))
    }
}
