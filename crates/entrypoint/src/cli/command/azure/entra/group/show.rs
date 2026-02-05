use clap::Args;
use clap::ValueEnum;
use cloud_terrastodon_azure::prelude::EntraGroup;
use cloud_terrastodon_azure::prelude::EntraGroupId;
use cloud_terrastodon_azure::prelude::Principal;
use cloud_terrastodon_azure::prelude::RoleAssignment;
use cloud_terrastodon_azure::prelude::RoleDefinition;
use cloud_terrastodon_azure::prelude::RoleDefinitionsAndAssignments;
use cloud_terrastodon_azure::prelude::RoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_groups;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_azure::prelude::fetch_group_members;
use cloud_terrastodon_azure::prelude::fetch_group_owners;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use color_eyre::owo_colors::OwoColorize;
use eyre::OptionExt;
use eyre::Result;
use eyre::bail;
use serde::Serialize;
use serde_json::to_writer_pretty;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::IsTerminal;
use std::io::stdin;
use std::io::stdout;
use tracing::info;

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Auto,
    Text,
    Json,
}

/// Show one or more Entra (Azure AD) groups by id.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupShowArgs {
    /// Group identifier(s) (UUID). Use '-' to read IDs from stdin (one per line).
    #[arg(long = "group-id")]
    pub group_id: Vec<String>,

    /// Output format
    #[arg(long = "output", default_value = "auto")]
    pub output_format: OutputFormat,
}

impl AzureEntraGroupShowArgs {
    pub async fn invoke(mut self) -> Result<()> {
        let is_terminal = std::io::stdout().is_terminal();
        if matches!(self.output_format, OutputFormat::Auto) {
            self.output_format = if is_terminal {
                OutputFormat::Text
            } else {
                OutputFormat::Json
            };
        }

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

        info!(count = ids.len(), "Fetching Entra groups");
        let groups = fetch_all_groups().await?;

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
            work.enqueue(async move {
                let members = fetch_group_members(group_id).await?;
                eyre::Ok(Resp::Members {
                    group_id,
                    principals: members,
                })
            });
            work.enqueue(async move {
                let owners = fetch_group_owners(group_id).await?;
                eyre::Ok(Resp::Owners {
                    group_id,
                    principals: owners,
                })
            });
        }
        work.enqueue(async move {
            let rbac = fetch_all_role_definitions_and_assignments().await?;
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

        #[derive(Serialize)]
        struct GroupData<'a> {
            group: EntraGroup,
            members: Vec<Principal>,
            owners: Vec<Principal>,
            role_assignments: Vec<(&'a RoleAssignment, &'a RoleDefinition)>,
        }

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
                group_rbac.push((role_assignment, role_definition));
            }
            rtn.push(GroupData {
                group,
                members,
                owners,
                role_assignments: group_rbac,
            });
        }

        match self.output_format {
            OutputFormat::Auto => unreachable!(),
            OutputFormat::Json => {
                to_writer_pretty(stdout(), &rtn)?;
                println!();
            }
            OutputFormat::Text => {
                for group_data in rtn {
                    let group = group_data.group;
                    let members = group_data.members;
                    let owners = group_data.owners;
                    let role_assignments = group_data.role_assignments;

                    // Group header
                    println!("{}", "────────────────────────────────────────".dimmed());

                    println!("{} {}", "Group ID:".cyan().bold(), group.id);
                    println!(
                        "{} {}",
                        "Display Name:".cyan().bold(),
                        group.display_name.cyan().bold()
                    );
                    if let Some(desc) = &group.description {
                        println!("{} {}", "Description:".cyan().bold(), desc.dimmed());
                    }
                    if let Some(created_date) = group.created_date_time {
                        println!(
                            "{} {}",
                            "Created DateTime:".cyan().bold(),
                            created_date.to_string().dimmed()
                        );
                    }
                    if let Some(mail) = &group.mail {
                        println!("{} {}", "Mail:".cyan().bold(), mail.blue().underline());
                    }

                    print!("{} ", "Group Types:".cyan().bold());
                    if group.group_types.is_empty() {
                        println!("None");
                    } else {
                        for (i, t) in group.group_types.iter().enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{}", t.magenta());
                        }
                        println!();
                    }

                    let sec = if group.security_enabled {
                        "true".green().bold().to_string()
                    } else {
                        "false".red().bold().to_string()
                    };
                    println!("{} {}", "Is Security Group:".cyan().bold(), sec);

                    println!("{}", format!("Owners ({}):", owners.len()).yellow().bold());
                    for owner in owners {
                        println!(
                            "  - {} ({})",
                            owner.name().green().bold(),
                            owner.id().to_string().dimmed()
                        );
                    }

                    println!(
                        "{}",
                        format!("Members ({}):", members.len()).yellow().bold()
                    );
                    for member in members {
                        println!(
                            "  - {} ({})",
                            member.name().blue().bold(),
                            member.id().to_string().dimmed()
                        );
                    }

                    println!(
                        "{}",
                        format!("Role Assignments ({}):", role_assignments.len())
                            .yellow()
                            .bold()
                    );
                    for (role_assignment, role_definition) in role_assignments {
                        println!("  - Role: {}", role_definition.display_name.cyan().bold());
                        println!(
                            "    Scope: {}",
                            role_assignment.scope.expanded_form().dimmed()
                        );
                    }

                    println!();
                }
            }
        }

        Ok(())
    }
}
