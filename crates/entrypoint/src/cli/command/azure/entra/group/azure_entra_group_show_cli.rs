use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::Principal;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignments;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_azure::fetch_group_members;
use cloud_terrastodon_azure::fetch_group_owners;
use cloud_terrastodon_azure::fetch_groups_by_id;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_credentials::AuthContext;
use color_eyre::owo_colors::Style;
use eyre::OptionExt;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use std::fmt::Write;
use std::io::BufRead;
use std::io::stdin;
use tracing::info;

/// Show one or more Entra (Azure AD) groups by id.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Group identifier(s) (UUID). Use '-' to read IDs from stdin (one per line).
    #[facet(figue::named)]
    pub group_id: Vec<String>,
}

#[derive(Debug, facet::Facet)]
struct GroupData {
    group: EntraGroup,
    members: Vec<Principal>,
    owners: Vec<Principal>,
    role_assignments: Vec<(RoleAssignment, RoleDefinition)>,
}

/// Transparent to preserve the existing JSON array of group reports.
#[derive(Debug, facet::Facet)]
#[facet(transparent)]
struct GroupShowOutput(Vec<GroupData>);

impl AzureEntraGroupShowArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        // Determine requested IDs. Support `-` as stdin source when a single `-` is provided.
        let id_strings: Vec<String> = if self.group_id.len() == 1 && self.group_id[0] == "-" {
            let mut v = Vec::new();
            for line in stdin().lock().lines() {
                let line = line?;
                let s = line.trim();
                if s.is_empty() {
                    continue;
                }
                v.push(s.to_string());
            }
            v
        } else {
            self.group_id
        };

        // Parse to EntraGroupId
        let mut ids: Vec<EntraGroupId> = Vec::new();
        for s in id_strings.iter() {
            ids.push(s.parse()?);
        }

        if ids.is_empty() {
            bail!("At least one group ID must be provided.");
        }

        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        info!(count = ids.len(), tenant_id = %tenant_auth_context.tenant_id, "Fetching Entra groups");
        let groups = fetch_groups_by_id(ids.clone(), &tenant_auth_context).await?;

        // Map by id for fast lookup
        let mut map: HashMap<EntraGroupId, EntraGroup> =
            groups.into_iter().map(|g| (g.id, g)).collect();

        // Emit each requested group in the order requested
        let mut chosen_groups = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(group) = map.remove(&id) {
                chosen_groups.push(group);
            } else {
                bail!("No group found matching '{}'.", id);
            }
        }

        let mut chosen_group_members = HashMap::with_capacity(chosen_groups.len());
        let mut chosen_group_owners = HashMap::with_capacity(chosen_groups.len());

        enum Resp {
            Members {
                group_id: EntraGroupId,
                principals: Vec<Principal>,
            },
            Owners {
                group_id: EntraGroupId,
                principals: Vec<Principal>,
            },
            Rbac(RoleDefinitionsAndAssignments),
        }
        let mut work =
            ParallelFallibleWorkQueue::new("group members, owners, and role assignments", 8);
        for group in &chosen_groups {
            let group_id = group.id;
            let members_auth_context = tenant_auth_context.clone();
            work.enqueue(async move {
                let members = fetch_group_members(group_id, &members_auth_context).await?;
                eyre::Ok(Resp::Members {
                    group_id,
                    principals: members,
                })
            });
            let owners_auth_context = tenant_auth_context.clone();
            work.enqueue(async move {
                let owners = fetch_group_owners(group_id, &owners_auth_context).await?;
                eyre::Ok(Resp::Owners {
                    group_id,
                    principals: owners,
                })
            });
        }
        let auth_context = tenant_auth_context;
        work.enqueue(async move {
            let rbac = fetch_all_role_definitions_and_assignments(&auth_context).await?;
            eyre::Ok(Resp::Rbac(rbac))
        });
        let work_results = work.join().await?;
        let mut rbac = None;
        for result in work_results {
            match result {
                Resp::Members {
                    group_id,
                    principals,
                } => {
                    chosen_group_members.insert(group_id, principals);
                }
                Resp::Owners {
                    group_id,
                    principals,
                } => {
                    chosen_group_owners.insert(group_id, principals);
                }
                Resp::Rbac(r) => {
                    assert!(rbac.is_none());
                    rbac = Some(r);
                }
            }
        }
        let Some(rbac) = rbac else {
            bail!("Failed to fetch role definitions and assignments");
        };

        let mut rtn = Vec::with_capacity(chosen_groups.len());
        for group in chosen_groups {
            let members = chosen_group_members
                .remove(&group.id)
                .ok_or_eyre("Missing members for group")?;
            let owners = chosen_group_owners
                .remove(&group.id)
                .ok_or_eyre("Missing owners for group")?;
            let mut group_rbac = Vec::new();
            for (role_assignment, role_definition) in
                rbac.iter_role_assignments().filter_principal(&group.id)
            {
                group_rbac.push((role_assignment.clone(), role_definition.clone()));
            }
            rtn.push(GroupData {
                group,
                members,
                owners,
                role_assignments: group_rbac,
            });
        }

        Ok(CliOutput::facet_with_text(
            GroupShowOutput(rtn),
            GroupShowOutput::render_text,
        ))
    }
}

impl GroupShowOutput {
    fn render_text(&self, stdout_is_terminal: bool) -> Result<String> {
        let mut output = String::new();
        let terminal_style = |style| {
            if stdout_is_terminal {
                style
            } else {
                Style::new()
            }
        };
        let header = terminal_style(Style::new().cyan().bold());
        let dimmed = terminal_style(Style::new().dimmed());
        let mail = terminal_style(Style::new().blue().underline());
        let magenta = terminal_style(Style::new().magenta());
        let green = terminal_style(Style::new().green().bold());
        let red = terminal_style(Style::new().red().bold());
        let blue = terminal_style(Style::new().blue().bold());
        let section = terminal_style(Style::new().yellow().bold());
        for group_data in &self.0 {
            let group = &group_data.group;
            let members = &group_data.members;
            let owners = &group_data.owners;
            let role_assignments = &group_data.role_assignments;

            writeln!(
                output,
                "{}",
                dimmed.style("────────────────────────────────────────")
            )?;
            writeln!(output, "{} {}", header.style("Group ID:"), group.id)?;
            writeln!(
                output,
                "{} {}",
                header.style("Display Name:"),
                header.style(&group.display_name)
            )?;
            if let Some(desc) = &group.description {
                writeln!(
                    output,
                    "{} {}",
                    header.style("Description:"),
                    dimmed.style(desc)
                )?;
            }
            if let Some(created_date) = &group.created_date_time {
                writeln!(
                    output,
                    "{} {}",
                    header.style("Created DateTime:"),
                    dimmed.style(created_date)
                )?;
            }
            if let Some(address) = &group.mail {
                writeln!(output, "{} {}", header.style("Mail:"), mail.style(address))?;
            }

            write!(output, "{} ", header.style("Group Types:"))?;
            if group.group_types.is_empty() {
                writeln!(output, "None")?;
            } else {
                for (i, group_type) in group.group_types.iter().enumerate() {
                    if i > 0 {
                        write!(output, ", ")?;
                    }
                    write!(output, "{}", magenta.style(group_type))?;
                }
                writeln!(output)?;
            }

            let sec = if group.security_enabled {
                green.style("true").to_string()
            } else {
                red.style("false").to_string()
            };
            writeln!(output, "{} {}", header.style("Is Security Group:"), sec)?;

            writeln!(
                output,
                "{}",
                section.style(format!("Owners ({}):", owners.len()))
            )?;
            for owner in owners {
                writeln!(
                    output,
                    "  - {} ({})",
                    green.style(owner.name()),
                    dimmed.style(owner.id())
                )?;
            }
            writeln!(
                output,
                "{}",
                section.style(format!("Members ({}):", members.len()))
            )?;
            for member in members {
                writeln!(
                    output,
                    "  - {} ({})",
                    blue.style(member.name()),
                    dimmed.style(member.id())
                )?;
            }
            writeln!(
                output,
                "{}",
                section.style(format!("Role Assignments ({}):", role_assignments.len()))
            )?;
            for (role_assignment, role_definition) in role_assignments {
                writeln!(
                    output,
                    "  - Role: {}",
                    header.style(&role_definition.display_name)
                )?;
                writeln!(
                    output,
                    "    Scope: {}",
                    dimmed.style(role_assignment.scope.expanded_form())
                )?;
            }
            writeln!(output)?;
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_app::OutputFormat;

    fn example_report() -> GroupShowOutput {
        let group = facet_json::from_str::<EntraGroup>(
            r#"{
            "id": "11111111-1111-1111-1111-111111111111",
            "displayName": "Synthetic Test Group",
            "description": "Offline fixture",
            "createdDateTime": "2026-01-01T00:00:00Z",
            "mail": "group@example.invalid",
            "creationOptions": [],
            "groupTypes": ["Unified", "DynamicMembership"],
            "onPremisesProvisioningErrors": [],
            "proxyAddresses": [],
            "resourceBehaviorOptions": [],
            "resourceProvisioningOptions": [],
            "securityEnabled": true,
            "securityIdentifier": "synthetic-identifier",
            "serviceProvisioningErrors": []
        }"#,
        )
        .unwrap();
        let member = facet_json::from_str::<cloud_terrastodon_azure::EntraUser>(
            r#"{
            "id": "22222222-2222-2222-2222-222222222222",
            "displayName": "Synthetic Test User",
            "userPrincipalName": "member@example.invalid",
            "businessPhones": []
        }"#,
        )
        .unwrap();
        let role_assignment = facet_json::from_str::<RoleAssignment>(r#"{
            "id": "/subscriptions/33333333-3333-3333-3333-333333333333/providers/Microsoft.Authorization/roleAssignments/44444444-4444-4444-4444-444444444444",
            "scope": "/subscriptions/33333333-3333-3333-3333-333333333333",
            "role_definition_id": "/providers/Microsoft.Authorization/roleDefinitions/55555555-5555-5555-5555-555555555555",
            "principal_id": "11111111-1111-1111-1111-111111111111"
        }"#).unwrap();
        let role_definition = facet_json::from_str::<RoleDefinition>(r#"{
            "id": "/providers/Microsoft.Authorization/roleDefinitions/55555555-5555-5555-5555-555555555555",
            "display_name": "Synthetic Reader",
            "description": "Offline fixture",
            "assignable_scopes": [],
            "permissions": [],
            "kind": "BuiltInRole"
        }"#).unwrap();
        GroupShowOutput(vec![GroupData {
            group,
            members: vec![member.clone().into()],
            owners: vec![member.into()],
            role_assignments: vec![(role_assignment, role_definition)],
        }])
    }

    #[test]
    fn group_text_preserves_sections_and_only_colors_terminal_output() {
        let output = CliOutput::facet_with_text(example_report(), GroupShowOutput::render_text);
        let plain = output
            .render(Some(OutputFormat::Text), false)
            .unwrap()
            .unwrap();
        for expected in [
            "Display Name: Synthetic Test Group",
            "Description: Offline fixture",
            "Mail: group@example.invalid",
            "Group Types: Unified, DynamicMembership",
            "Is Security Group: true",
            "Owners (1):",
            "Members (1):",
            "member@example.invalid",
            "Role Assignments (1):",
            "Role: Synthetic Reader",
            "Scope: /subscriptions/33333333-3333-3333-3333-333333333333",
        ] {
            assert!(
                plain.contains(expected),
                "missing fixture field: {expected}"
            );
        }
        assert!(!plain.contains('\x1b'));
        let terminal = output
            .render(Some(OutputFormat::Text), true)
            .unwrap()
            .unwrap();
        assert!(terminal.contains('\x1b'));
        assert!(terminal.contains("Synthetic Test Group"));
    }

    #[test]
    fn group_json_preserves_array_schema_principals_and_owned_role_pairs() {
        let output = CliOutput::facet_with_text(example_report(), GroupShowOutput::render_text);
        let json = output
            .render(Some(OutputFormat::Json), true)
            .unwrap()
            .unwrap();
        assert!(!json.contains('\x1b'));
        let json: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["group"]["displayName"], "Synthetic Test Group");
        assert_eq!(
            json[0]["members"][0]["userPrincipalName"],
            "member@example.invalid"
        );
        assert_eq!(json[0]["owners"][0]["@odata.type"], "#microsoft.graph.user");
        assert_eq!(json[0]["role_assignments"][0].as_array().unwrap().len(), 2);
        assert_eq!(
            json[0]["role_assignments"][0][0]["scope"],
            "/subscriptions/33333333-3333-3333-3333-333333333333"
        );
        assert_eq!(
            json[0]["role_assignments"][0][1]["display_name"],
            "Synthetic Reader"
        );
    }
}
