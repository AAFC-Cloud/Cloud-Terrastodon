use std::collections::HashMap;
use std::collections::HashSet;

use eyre::bail;
use eyre::Result;
use cloud_terrastodon_core_azure::prelude::ensure_logged_in;
use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments_v2;
use cloud_terrastodon_core_azure::prelude::fetch_all_security_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_azure::prelude::ResourceGroupScoped;
use cloud_terrastodon_core_azure::prelude::RoleAssignmentId;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::SubscriptionScoped;
use cloud_terrastodon_core_azure::prelude::UuidWrapper;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::Sanitizable;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderKind;
use cloud_terrastodon_core_tofu::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use tokio::fs::remove_dir_all;
use tokio::join;
use tracing::info;

pub async fn resource_group_import_wizard_menu() -> Result<()> {
    info!("Confirming remove existing imports");
    let start_from_scratch = "start from scratch";
    let keep_existing_imports = "keep existing imports";
    match pick(FzfArgs {
        choices: vec![start_from_scratch, keep_existing_imports],
        prompt: None,
        header: Some("This will wipe any existing imports from the Cloud Terrastodon work directory. Proceed?".to_string()),
    })? {
        x if x == start_from_scratch => {
            info!("Removing existing imports");
            let _ = remove_dir_all(AppDir::Imports.as_path_buf()).await;
            let _ = remove_dir_all(AppDir::Processed.as_path_buf()).await;
        }
        x if x == keep_existing_imports => {
            info!("Keeping existing imports");
        }
        _ => unreachable!(),
    }

    info!("Ensuring CLI is authenticated");
    ensure_logged_in().await?;

    info!("Fetching a bunch of data");
    let (
        subscriptions,
        resource_groups,
        role_assignments,
        // role_definitions,
        security_groups,
        // users,
    ) = join!(
        fetch_all_subscriptions(),
        fetch_all_resource_groups(),
        fetch_all_role_assignments_v2(),
        // fetch_all_role_definitions(),
        fetch_all_security_groups(),
        // fetch_all_users()
    );
    let subscriptions = subscriptions?
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();
    let resource_groups = resource_groups?;
    let role_assignments = role_assignments?;
    // let role_definitions = role_definitions?;
    let security_groups = security_groups?;
    // let users = users?;

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
        prompt: None,
        header: Some("Pick which to import".to_string()),
    })?;
    info!("You chose {} resource groups", resource_groups.len());

    let mut used_resource_groups = HashSet::new();
    let mut used_subscriptions = HashSet::new();

    info!("Building resource group imports");
    let mut rg_imports = Vec::new();
    for entry in resource_groups {
        let (rg, sub) = entry.value;

        // Track the RG id for filtering role assignments later
        used_resource_groups.insert(rg.id.to_owned());

        // Create the import block
        let mut import_block: TofuImportBlock = rg.into();

        // Update the provider to use the subscription alias
        import_block.provider = TofuProviderReference::Alias {
            kind: TofuProviderKind::AzureRM,
            name: sub.name.sanitize(),
        };

        // Track the subscription for writing the provider blocks later
        used_subscriptions.insert(sub);

        // Add to results
        rg_imports.push(import_block);
    }

    info!("Writing resource group imports");
    TofuWriter::new(AppDir::Imports.join("resource_group_imports.tf"))
        .overwrite(rg_imports)
        .await?
        .format()
        .await?;

    let mut used_principals: HashSet<Uuid> = HashSet::new();

    info!("Building role assignment imports");
    let mut ra_imports = Vec::new();
    for ra in role_assignments {
        // Only import resource group level role assignments
        let RoleAssignmentId::ResourceGroupScoped(ra_id) = &ra.id else {
            continue;
        };

        // Only import role assignments targetting a resource group being imported
        if !used_resource_groups.contains(&ra_id.resource_group_id()) {
            continue;
        }

        // Identify subscription
        let Some(sub) = subscriptions.get(&ra_id.subscription_id()) else {
            bail!(
                "Could not find subscription for role assignment {}",
                ra_id.expanded_form()
            );
        };

        // Track the principal
        used_principals.insert(*ra.principal_id);

        // Create the import block
        let mut import_block: TofuImportBlock = ra.into();

        // Update the provider to use the subscription alias
        import_block.provider = TofuProviderReference::Alias {
            kind: TofuProviderKind::AzureRM,
            name: sub.name.sanitize(),
        };

        // Add to results
        ra_imports.push(import_block);
    }

    info!("Writing role assignment imports");
    TofuWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(ra_imports)
        .await?
        .format()
        .await?;

    info!("Building security group imports");
    let mut sg_imports = Vec::new();
    for sg in security_groups {
        // Only import security groups which have role assignments
        if !used_principals.contains(sg.id.as_ref()) {
            continue;
        }

        // Create the import block
        let import_block: TofuImportBlock = sg.into();

        // Add to results
        sg_imports.push(import_block);
    }

    info!("Writing security group imports");
    TofuWriter::new(AppDir::Imports.join("security_groups.tf"))
        .overwrite(sg_imports)
        .await?
        .format()
        .await?;

    info!("Building provider blocks");
    let mut providers = Vec::new();
    for sub in used_subscriptions {
        let provider = sub.clone().into_provider_block();
        providers.push(provider);
    }

    info!("Writing provider blocks");
    TofuWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format()
        .await?;

    Ok(())
}
