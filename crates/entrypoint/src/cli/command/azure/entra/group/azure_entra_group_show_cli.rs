use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraDirectoryObject;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignments;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_azure::fetch_group_members;
use cloud_terrastodon_azure::fetch_group_owners;
use cloud_terrastodon_azure::fetch_groups_by_id;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_credentials::AuthContext;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::OptionExt;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::Write;
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
    members: Vec<EntraDirectoryObject>,
    owners: Vec<EntraDirectoryObject>,
    role_assignments: Vec<(RoleAssignment, RoleDefinition)>,
}

/// Transparent to preserve the existing JSON array of group reports.
#[derive(Debug, facet::Facet)]
#[facet(transparent)]
struct GroupShowOutput(Vec<GroupData>);

impl AzureEntraGroupShowArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<CliOutput> {
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

        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        info!(count = ids.len(), tenant_id = %tenant_auth_context.tenant_id, "Fetching Entra groups");
        let groups = fetch_groups_by_id(ids.clone(), &tenant_auth_context, cancellation).await?;

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
                members: Vec<EntraDirectoryObject>,
            },
            Owners {
                group_id: EntraGroupId,
                owners: Vec<EntraDirectoryObject>,
            },
            Rbac(RoleDefinitionsAndAssignments),
        }
        let mut work =
            ParallelFallibleWorkQueue::new("group members, owners, and role assignments", 8);
        for group in &chosen_groups {
            let group_id = group.id;
            let members_auth_context = tenant_auth_context.clone();
            let members_cancellation = cancellation.clone();
            work.enqueue(async move {
                let members =
                    fetch_group_members(group_id, &members_auth_context, &members_cancellation)
                        .await?;
                eyre::Ok(Resp::Members {
                    group_id,
                    members,
                })
            });
            let owners_auth_context = tenant_auth_context.clone();
            let owners_cancellation = cancellation.clone();
            work.enqueue(async move {
                let owners =
                    fetch_group_owners(group_id, &owners_auth_context, &owners_cancellation)
                        .await?;
                eyre::Ok(Resp::Owners {
                    group_id,
                    owners,
                })
            });
        }
        let auth_context = tenant_auth_context;
        let rbac_cancellation = cancellation.clone();
        work.enqueue(async move {
            let rbac =
                fetch_all_role_definitions_and_assignments(&auth_context, &rbac_cancellation)
                    .await?;
            eyre::Ok(Resp::Rbac(rbac))
        });
        let work_results = work.join().await?;
        let mut rbac = None;
        for result in work_results {
            match result {
                Resp::Members {
                    group_id,
                    members,
                } => {
                    chosen_group_members.insert(group_id, members);
                }
                Resp::Owners {
                    group_id,
                    owners,
                } => {
                    chosen_group_owners.insert(group_id, owners);
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
            GroupShowOutput::write_text,
        ))
    }
}

impl GroupShowOutput {
    fn write_text(&self, writer: &mut dyn Write, cancellation: &CancellationScope) -> Result<()> {
        let header = Style::new().cyan().bold();
        let dimmed = Style::new().dimmed();
        let mail = Style::new().blue().underline();
        let magenta = Style::new().magenta();
        let green = Style::new().green().bold();
        let red = Style::new().red().bold();
        let blue = Style::new().blue().bold();
        let section = Style::new().yellow().bold();
        for group_data in &self.0 {
            cancellation.bail_if_cancelled()?;
            let group = &group_data.group;
            let members = &group_data.members;
            let owners = &group_data.owners;
            let role_assignments = &group_data.role_assignments;

            writeln!(
                writer,
                "{}",
                "────────────────────────────────────────"
                    .if_supports_color(Stream::Stdout, |text| text.style(dimmed))
            )?;
            writeln!(
                writer,
                "{} {}",
                "Group ID:".if_supports_color(Stream::Stdout, |text| text.style(header)),
                group.id
            )?;
            writeln!(
                writer,
                "{} {}",
                "Display Name:".if_supports_color(Stream::Stdout, |text| text.style(header)),
                group
                    .display_name
                    .if_supports_color(Stream::Stdout, |text| text.style(header))
            )?;
            if let Some(desc) = &group.description {
                writeln!(
                    writer,
                    "{} {}",
                    "Description:".if_supports_color(Stream::Stdout, |text| text.style(header)),
                    desc.if_supports_color(Stream::Stdout, |text| text.style(dimmed))
                )?;
            }
            if let Some(created_date) = &group.created_date_time {
                writeln!(
                    writer,
                    "{} {}",
                    "Created DateTime:"
                        .if_supports_color(Stream::Stdout, |text| text.style(header)),
                    created_date.if_supports_color(Stream::Stdout, |date| date.style(dimmed))
                )?;
            }
            if let Some(address) = &group.mail {
                writeln!(
                    writer,
                    "{} {}",
                    "Mail:".if_supports_color(Stream::Stdout, |text| text.style(header)),
                    address.if_supports_color(Stream::Stdout, |text| text.style(mail))
                )?;
            }

            write!(
                writer,
                "{} ",
                "Group Types:".if_supports_color(Stream::Stdout, |text| text.style(header))
            )?;
            if group.group_types.is_empty() {
                writeln!(writer, "None")?;
            } else {
                for (i, group_type) in group.group_types.iter().enumerate() {
                    cancellation.bail_if_cancelled()?;
                    if i > 0 {
                        write!(writer, ", ")?;
                    }
                    write!(
                        writer,
                        "{}",
                        group_type.if_supports_color(Stream::Stdout, |text| text.style(magenta))
                    )?;
                }
                writeln!(writer)?;
            }

            let (security, security_style) = if group.security_enabled {
                ("true", green)
            } else {
                ("false", red)
            };
            writeln!(
                writer,
                "{} {}",
                "Is Security Group:".if_supports_color(Stream::Stdout, |text| text.style(header)),
                security.if_supports_color(Stream::Stdout, |text| text.style(security_style))
            )?;

            writeln!(
                writer,
                "{}",
                format!("Owners ({}):", owners.len())
                    .if_supports_color(Stream::Stdout, |text| text.style(section))
            )?;
            for owner in owners {
                cancellation.bail_if_cancelled()?;
                writeln!(
                    writer,
                    "  - {} ({})",
                    owner
                        .display_name()
                        .unwrap_or("Name unavailable")
                        .if_supports_color(Stream::Stdout, |text| text.style(green)),
                    owner
                        .id()
                        .if_supports_color(Stream::Stdout, |id| id.style(dimmed))
                )?;
            }
            writeln!(
                writer,
                "{}",
                format!("Members ({}):", members.len())
                    .if_supports_color(Stream::Stdout, |text| text.style(section))
            )?;
            for member in members {
                cancellation.bail_if_cancelled()?;
                writeln!(
                    writer,
                    "  - {} ({})",
                    member
                        .display_name()
                        .unwrap_or("Name unavailable")
                        .if_supports_color(Stream::Stdout, |text| text.style(blue)),
                    member
                        .id()
                        .if_supports_color(Stream::Stdout, |id| id.style(dimmed))
                )?;
            }
            writeln!(
                writer,
                "{}",
                format!("Role Assignments ({}):", role_assignments.len())
                    .if_supports_color(Stream::Stdout, |text| text.style(section))
            )?;
            for (role_assignment, role_definition) in role_assignments {
                cancellation.bail_if_cancelled()?;
                writeln!(
                    writer,
                    "  - Role: {}",
                    role_definition
                        .display_name
                        .if_supports_color(Stream::Stdout, |text| text.style(header))
                )?;
                writeln!(
                    writer,
                    "    Scope: {}",
                    role_assignment
                        .scope
                        .expanded_form()
                        .if_supports_color(Stream::Stdout, |scope| scope.style(dimmed))
                )?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_azure::EntraDirectoryObjectUser;
    use cloud_terrastodon_command::CancellationToken;

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
        let member = EntraDirectoryObject::User(EntraDirectoryObjectUser {
            id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            display_name: Some("Synthetic Test User".to_owned()),
            user_principal_name: Some("member@example.invalid".parse().unwrap()),
            mail: None,
        });
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
            members: vec![member.clone()],
            owners: vec![member],
            role_assignments: vec![(role_assignment, role_definition)],
        }])
    }

    #[test]
    fn group_text_preserves_sections_and_fields() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let output =
                    CliOutput::facet_with_text(example_report(), GroupShowOutput::write_text);
                let plain = output
                    .render(Some(OutputFormat::Text), false, &cancellation)
                    .unwrap()
                    .unwrap();
                for expected in [
                    "Display Name:",
                    "Synthetic Test Group",
                    "Description:",
                    "Offline fixture",
                    "Mail:",
                    "group@example.invalid",
                    "Group Types:",
                    "Unified",
                    "DynamicMembership",
                    "Is Security Group:",
                    "true",
                    "Owners (1):",
                    "Members (1):",
                    "member@example.invalid",
                    "Role Assignments (1):",
                    "Role:",
                    "Synthetic Reader",
                    "Scope:",
                    "/subscriptions/33333333-3333-3333-3333-333333333333",
                ] {
                    assert!(
                        plain.contains(expected),
                        "missing fixture field: {expected}"
                    );
                }
                Ok::<_, eyre::Report>(())
            })
            .unwrap();
    }

    #[test]
    fn limited_information_members_keep_their_identity_without_inventing_names() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let mut report = example_report();
                report.0[0].members = vec![EntraDirectoryObject::User(EntraDirectoryObjectUser {
                    id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
                    display_name: None,
                    user_principal_name: None,
                    mail: None,
                })];
                let output = CliOutput::facet_with_text(report, GroupShowOutput::write_text);
                let text = output
                    .render(Some(OutputFormat::Text), false, &cancellation)?
                    .unwrap();
                assert!(text.contains("Name unavailable"));
                assert!(text.contains("22222222-2222-2222-2222-222222222222"));
                let json = output
                    .render(Some(OutputFormat::Json), false, &cancellation)?
                    .unwrap();
                let json: serde_json::Value = serde_json::from_str(&json)?;
                assert!(json[0]["members"][0]["displayName"].is_null());
                assert!(json[0]["members"][0]["userPrincipalName"].is_null());
                Ok::<_, eyre::Report>(())
            })
            .unwrap();
    }

    #[test]
    fn group_json_preserves_array_schema_principals_and_owned_role_pairs() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let output =
                    CliOutput::facet_with_text(example_report(), GroupShowOutput::write_text);
                let json = output
                    .render(Some(OutputFormat::Json), true, &cancellation)
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
                Ok::<_, eyre::Report>(())
            })
            .unwrap();
    }
}
