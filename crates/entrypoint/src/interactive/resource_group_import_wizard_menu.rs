use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::ResourceGroupScoped;
use cloud_terrastodon_azure::RoleAssignmentId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::SubscriptionScoped;
use cloud_terrastodon_azure::fetch_all_role_assignments;
use cloud_terrastodon_azure::fetch_all_security_groups;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_azure::get_resource_group_choices;
use cloud_terrastodon_azure::uuid::Uuid;
use cloud_terrastodon_hcl::HclImportBlock;
use cloud_terrastodon_hcl::HclProviderBlock;
use cloud_terrastodon_hcl::HclProviderReference;
use cloud_terrastodon_hcl::HclWriter;
use cloud_terrastodon_hcl::ProviderKind;
use cloud_terrastodon_hcl::Sanitizable;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use std::collections::HashSet;
use tokio::fs::remove_dir_all;
use tokio::join;
use tracing::info;

pub async fn resource_group_import_wizard_menu(tenant_id: AzureTenantId) -> Result<()> {
    info!("Confirming remove existing imports");
    let start_from_scratch = "start from scratch";
    let keep_existing_imports = "keep existing imports";
    match PickerTui::new()
        .set_header("This will wipe any existing imports from the Cloud Terrastodon work directory. Proceed?")
        .pick_one(vec![start_from_scratch, keep_existing_imports])? {
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

    info!("Fetching a bunch of data");
    let (
        subscriptions,
        // resource_groups,
        role_assignments,
        // role_definitions,
        security_groups,
        // users,
        resource_group_choices,
    ) = join!(
        fetch_all_subscriptions(tenant_id),
        // fetch_all_resource_groups(tenant_id),
        fetch_all_role_assignments(tenant_id),
        // fetch_all_role_definitions(),
        fetch_all_security_groups(tenant_id),
        // fetch_all_users(),
        get_resource_group_choices(tenant_id),
    );
    let subscriptions = subscriptions?
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();
    let role_assignments = role_assignments?;
    // let role_definitions = role_definitions?;
    let security_groups = security_groups?;
    // let users = users?;

    info!("Picking resource groups");
    let chosen_resource_groups = PickerTui::new()
        .set_header("Pick which to import")
        .pick_many(resource_group_choices?)?;
    info!("You chose {} resource groups", chosen_resource_groups.len());

    let mut used_resource_groups = HashSet::new();
    let mut used_subscriptions = HashSet::new();

    info!("Building resource group imports");
    let mut rg_imports = Vec::new();
    for rg in chosen_resource_groups {
        // Track the RG id for filtering role assignments later
        used_resource_groups.insert(rg.id.to_owned());

        // Track the subscription for writing the provider blocks later
        used_subscriptions.insert((rg.id.subscription_id, rg.subscription_name.to_owned()));

        // Create the import block
        let provider_name = rg.subscription_name.sanitize();
        let mut import_block: HclImportBlock = rg.into();

        // Update the provider to use the subscription alias
        import_block.provider = HclProviderReference::Alias {
            kind: ProviderKind::AzureRM,
            name: provider_name,
        };

        // Add to results
        rg_imports.push(import_block);
    }

    info!("Writing resource group imports");
    HclWriter::new(AppDir::Imports.join("resource_group_imports.tf"))
        .overwrite(rg_imports)
        .await?
        .format_file()
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
        if !used_resource_groups.contains(ra_id.resource_group_id()) {
            continue;
        }

        // Identify subscription
        let Some(sub) = subscriptions.get(ra_id.subscription_id()) else {
            bail!(
                "Could not find subscription for role assignment {}",
                ra_id.expanded_form()
            );
        };

        // Track the principal
        used_principals.insert(*ra.principal_id);

        // Create the import block
        let mut import_block: HclImportBlock = ra.into();

        // Update the provider to use the subscription alias
        import_block.provider = HclProviderReference::Alias {
            kind: ProviderKind::AzureRM,
            name: sub.name.sanitize(),
        };

        // Add to results
        ra_imports.push(import_block);
    }

    info!("Writing role assignment imports");
    HclWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(ra_imports)
        .await?
        .format_file()
        .await?;

    info!("Building security group imports");
    let mut sg_imports = Vec::new();
    for sg in security_groups {
        // Only import security groups which have role assignments
        if !used_principals.contains(sg.id.as_ref()) {
            continue;
        }

        // Create the import block
        let import_block: HclImportBlock = sg.into();

        // Add to results
        sg_imports.push(import_block);
    }

    info!("Writing security group imports");
    HclWriter::new(AppDir::Imports.join("security_groups.tf"))
        .overwrite(sg_imports)
        .await?
        .format_file()
        .await?;

    info!("Building provider blocks");
    let mut providers = Vec::new();
    for (subscription_id, subscription_name) in used_subscriptions {
        let provider = HclProviderBlock::AzureRM {
            alias: Some(subscription_name.sanitize()),
            subscription_id: Some(subscription_id.short_form()),
        };
        providers.push(provider);
    }

    info!("Writing provider blocks");
    HclWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format_file()
        .await?;

    Ok(())
}
