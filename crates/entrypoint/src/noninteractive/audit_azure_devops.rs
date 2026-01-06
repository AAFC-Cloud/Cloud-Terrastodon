use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseEntitlementLicense;
use cloud_terrastodon_azure_devops::prelude::LastAccessedDate;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use std::collections::HashMap;
use tracing::info;
use tracing::warn;

pub async fn audit_azure_devops() -> eyre::Result<()> {
    info!("Fetching a buncha information...");

    let mut total_problems = 0;
    let mut total_cost_waste_cad = 0.00;
    let mut message_counts: HashMap<&'static str, usize> = HashMap::new();

    let org_url = get_default_organization_url().await?;
    let entitlements = fetch_azure_devops_license_entitlements(&org_url).await?;
    let users_by_principal_name = fetch_all_users()
        .await?
        .into_iter()
        .map(|user| (user.user_principal_name.to_lowercase(), user))
        .collect::<HashMap<_, _>>();

    // Emit a warning for anyone who has never accessed azure devops and have greater than stakeholder license
    for entitlement in entitlements
        .iter()
        .filter(|e| e.last_accessed_date == LastAccessedDate::Never)
        .filter(|e| e.license != AzureDevOpsLicenseEntitlementLicense::AccountStakeholder)
    {
        let msg = "User has never accessed Azure DevOps but has a paid license; consider downgrading to stakeholder license or removing access";
        warn!(
            user_display_name = %entitlement.user.display_name,
            user_unique_name = %entitlement.user.unique_name,
            license = ?entitlement.license,
            status = ?entitlement.status,
            cost_per_month_cad = %entitlement.license.cost_per_month_cad(),
            "{}", msg
        );
        total_problems += 1;
        total_cost_waste_cad += entitlement.license.cost_per_month_cad();
        *message_counts.entry(msg).or_insert(0) += 1;
    }

    // Emit a warning for entitlements for users who do not exist in entra
    for entitlement in entitlements
        .iter()
        .filter(|e| matches!(e.user.descriptor, AzureDevOpsDescriptor::EntraUser(_)))
    {
        if !users_by_principal_name.contains_key(&entitlement.user.unique_name.to_lowercase()) {
            let msg = "Azure DevOps entitlement exists for user that no longer exists in Entra ID; consider removing this orphaned entitlement to save costs";
            warn!(
                user_display_name = %entitlement.user.display_name,
                user_unique_name = %entitlement.user.unique_name,
                user_descriptor = ?entitlement.user.descriptor,
                license = ?entitlement.license,
                status = ?entitlement.status,
                cost_per_month_cad = %entitlement.license.cost_per_month_cad(),
                "{}", msg
            );
            total_problems += 1;
            total_cost_waste_cad += entitlement.license.cost_per_month_cad();
            *message_counts.entry(msg).or_insert(0) += 1;
        }
    }

    // Emit a warning for licenses assigned to admin accounts for which no user account exists
    // blocker TODO: create entra helper to cleanly map between user and admin accounts

    // Emit a warning for users with a AccountAdvanced license who have not used a test plan in the past 30 days
    // TODO: enumerate test plans using new test plan functions to find edited date and stuff


    // Emit summary
    if total_problems > 0 {
        warn!(
            total_problems,
            total_cost_waste_cad,
            message_counts = ?message_counts,
            "Found potential problems in Azure DevOps"
        );
        warn!(
            total_cost_waste_cad,
            "Potential monthly cost waste: ${:.2} CAD", total_cost_waste_cad
        );

        // Emit message type summary
        for (msg, count) in &message_counts {
            warn!(count = %count, "{}", msg);
        }
    } else {
        info!("No potential problems found in Azure DevOps");
    }
    Ok(())
}
