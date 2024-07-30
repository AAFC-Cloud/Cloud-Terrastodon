use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::bail;
use anyhow::Result;
use azure::prelude::fetch_all_resource_groups;
use azure::prelude::fetch_all_role_assignments_v2;
use azure::prelude::fetch_all_role_definitions;
use azure::prelude::fetch_all_security_groups;
use azure::prelude::fetch_all_subscriptions;
use azure::prelude::fetch_all_users;
use azure::prelude::Scope;
use fzf::pick;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use pathing::AppDir;
use tofu::prelude::Sanitizable;
use tofu::prelude::TofuImportBlock;
use tofu::prelude::TofuProviderKind;
use tofu::prelude::TofuProviderReference;
use tofu::prelude::TofuWriter;
use tokio::fs::remove_dir_all;
use tokio::join;
use tracing::info;

pub async fn resource_group_import_wizard_menu() -> Result<()> {
    info!("Confirming remove existing imports");
    match pick(FzfArgs {
        choices: vec!["start from scratch", "keep existing imports"],
        prompt: None,
        header: Some("This will wipe any existing imports from the Cloud Terrastodon work directory. Proceed?".to_string()),
    })? {
        "Yes" => {
            info!("Removing existing imports");
            let _ = remove_dir_all(AppDir::Imports.as_path_buf()).await;
        }
        "No" => {
            bail!("User said no");
        }
        _ => unreachable!(),
    }

    info!("Fetching subscriptions, resource groups, role assignments, role definitions, security groups, users");
    let (
        subscriptions,
        resource_groups,
        role_assignments,
        role_definitions,
        security_groups,
        users,
    ) = join!(
        fetch_all_subscriptions(),
        fetch_all_resource_groups(),
        fetch_all_role_assignments_v2(),
        fetch_all_role_definitions(),
        fetch_all_security_groups(),
        fetch_all_users()
    );
    let subscriptions = subscriptions?
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();
    let resource_groups = resource_groups?;
    let role_assignments = role_assignments?;
    let role_definitions = role_definitions?;
    let security_groups = security_groups?;
    let users = users?;

    info!("Building pick list");
    let mut resource_group_choices = Vec::new();
    for rg in resource_groups {
        let Some(sub) = subscriptions.get(&rg.subscription_id) else {
            bail!(
                "Failed to find subscription {} for resource group {}",
                rg.subscription_id,
                rg.name
            );
        };
        let choice = Choice {
            key: format!("{:16} {}", sub.name, rg.name),
            value: (rg, sub),
        };
        resource_group_choices.push(choice);
    }

    info!("Picking resource groups");
    let resource_groups = pick_many(FzfArgs {
        choices: resource_group_choices,
        prompt: Some("Pick which to import".to_string()),
        header: None,
    })?;
    info!("You chose {} resource groups", resource_groups.len());

    info!("Writing resource group imports");
    TofuWriter::new(AppDir::Imports.join("resource_group_imports.tf"))
        .overwrite(
            resource_groups
                .iter()
                .map(|x| &x.value)
                .map(|(rg, sub)| {
                    let mut block: TofuImportBlock = rg.clone().into();
                    block.provider = TofuProviderReference::Alias {
                        kind: TofuProviderKind::AzureRM,
                        name: sub.name.sanitize(),
                    };
                    block
                })
                .collect_vec(),
        )
        .await?
        .format()
        .await?;

    info!("Identifying relevant role assignments");
    // Convert to lower because I don't trust Azure IDs to match case everywhere
    let resource_group_ids = resource_groups
        .iter()
        .map(|rg| rg.value.0.id.expanded_form().to_lowercase())
        .collect::<HashSet<_>>();
    let role_assignments = role_assignments
        .into_iter()
        .filter(|ra| resource_group_ids.contains(&ra.scope.to_lowercase()))
        .collect_vec();

    // info!("Writing role assignment imports");
    // TofuWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
    //     .overwrite(
    //         role_assignments
    //             .iter()
    //             .map(|ra| {
    //                 let mut block: TofuImportBlock = (*ra.clone()).into();
    //                 let sub = subscriptions.get(ra.id)
    //                 block.provider = TofuProviderReference::Alias {
    //                     kind: TofuProviderKind::AzureRM,
    //                     name: sub.name.sanitize(),
    //                 };
    //                 block
    //             })
    //             .collect_vec(),
    //     )
    //     .await?
    //     .format()
    //     .await?;

    Ok(())
}
