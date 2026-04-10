use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_azure::fetch_all_role_definitions_and_assignments;
use itertools::Itertools;
use std::collections::HashMap;
use tokio::try_join;
use tracing::info;
use tracing::warn;

#[allow(unused_mut)]
#[allow(unused)]
pub async fn audit_azure(tenant_id: AzureTenantId) -> eyre::Result<()> {
    // TODO: audit admin accounts without corresponding user accounts should be disabled
    // TODO: audit admin accounts without corresponding user accounts should be deleted
    info!("Fetching information...");
    let start = std::time::Instant::now();
    let mut total_problems = 0;
    let mut total_cost_waste_cad = 0.00;
    let mut message_counts: HashMap<&'static str, usize> = HashMap::new();
    let (rbac, principals, resources) = try_join!(
        fetch_all_role_definitions_and_assignments(tenant_id),
        fetch_all_principals(tenant_id),
        fetch_all_resources(tenant_id)
    )?;
    let elapsed = start.elapsed();
    info!(
        elapsed_ms = elapsed.as_millis(),
        "Finished fetching information in {}, starting audit...",
        humantime::format_duration(elapsed)
    );

    // Identify role assignments for which the principal is unknown
    for (role_assignment, role_definition) in rbac.iter_role_assignments() {
        let principal_id = &role_assignment.principal_id;
        if !principals.contains_key(principal_id) {
            total_problems += 1;
            let msg = "Found role assignment with unknown principal";
            warn!(
                principal_id = ?principal_id,
                role_definition_name = %role_definition.display_name,
                role_assignment_id = %role_assignment.id.expanded_form(),
                scope = %role_assignment.scope.expanded_form(),
                "{}", msg,
            );
            *message_counts.entry(msg).or_insert(0) += 1;
        }
    }

    // Identify service principals with expiring or expired credentials
    let now = chrono::Utc::now();
    for principal in principals
        .values()
        .filter_map(|principal| principal.as_service_principal())
    {
        let has_any_valid_password_cred_that_is_good_for_more_than_30_days = principal
            .password_credentials
            .iter()
            .any(|password_cred| (password_cred.end_date_time - now).num_days() > 30);

        for password_cred in principal.password_credentials.iter() {
            let end_date = password_cred.end_date_time;
            let days_until_expiry = (end_date - now).num_days();

            // Possible states:
            // - Expired with no new one <- warn
            // - Soon expiring with no new one <- warn
            // - Expiring with a new one <- okaydoke
            // - Expired with a new one <- warn, should be cleanup
            if days_until_expiry < 0
                && !has_any_valid_password_cred_that_is_good_for_more_than_30_days
            {
                total_problems += 1;
                let msg =
                    "Service principal has an expired password credential and no valid credentials";
                warn!(
                    principal_id = %principal.id,
                    principal_name = %principal.display_name,
                    password_cred_end_date = %end_date,
                    days_until_expiry,
                    "{}", msg,
                );
                *message_counts.entry(msg).or_insert(0) += 1;
            } else if days_until_expiry < 30
                && !has_any_valid_password_cred_that_is_good_for_more_than_30_days
            {
                total_problems += 1;
                let msg = "Service principal has a soon-to-expire password credential and no valid credentials";
                warn!(
                    principal_id = %principal.id,
                    principal_name = %principal.display_name,
                    password_cred_end_date = %end_date,
                    days_until_expiry,
                    "{}", msg,
                );
                *message_counts.entry(msg).or_insert(0) += 1;
            } else if days_until_expiry < 0
                && has_any_valid_password_cred_that_is_good_for_more_than_30_days
            {
                total_problems += 1;
                let msg = "Service principal has an expired password credential but also has valid credentials; consider cleaning up old credentials";
                warn!(
                    principal_id = %principal.id,
                    principal_name = %principal.display_name,
                    password_cred_end_date = %end_date,
                    days_until_expiry,
                    "{}", msg,
                );
                *message_counts.entry(msg).or_insert(0) += 1;
            }
        }
    }

    // Audit resources which have tag keys that the parent have but where the values do not match the parent
    let resource_tags = resources
        .iter()
        .map(|resource| (resource.id.expanded_form(), &resource.tags))
        .collect::<HashMap<_, _>>();
    for resource in resource_tags.keys() {
        let mut chunky: &str = resource;
        while let Some((parent, _)) = chunky.rsplit_once("/") {
            if let Some(parent_tags) = resource_tags.get(parent) {
                if let Some(resource_tags) = resource_tags.get(resource) {
                    for (tag_key, parent_tag_value) in parent_tags.iter() {
                        if let Some(resource_tag_value) = resource_tags.get(tag_key)
                            && resource_tag_value != parent_tag_value
                        {
                            total_problems += 1;
                            let msg = "Resource tag value does not match parent resource tag value";
                            warn!(
                                resource = %resource,
                                parent = %parent,
                                tag_key = %tag_key,
                                resource_tag_value = %resource_tag_value,
                                parent_tag_value = %parent_tag_value,
                                "{}", msg,
                            );
                            *message_counts.entry(msg).or_insert(0) += 1;
                        }
                    }
                }
                break;
            }
            chunky = parent;
        }
    }

    // Audit admin user accounts that do not have a corresponding user account according to the other_mails property
    {
        let users_by_email = principals
            .values()
            .filter_map(|p| p.as_user())
            .map(|user| (user.user_principal_name.to_lowercase(), user))
            .collect::<HashMap<_, _>>();
        for principal in principals
            .values()
            .filter_map(|p| p.as_user())
            .filter(|user| {
                user.user_principal_name
                    .to_lowercase()
                    .starts_with("admin.")
            })
        {
            let mut non_admin_account = None;
            for other_mail in &principal.other_mails {
                if let Some(user) = users_by_email.get(&other_mail.to_lowercase()) {
                    non_admin_account = Some(user);
                    break;
                }
            }
            if non_admin_account.is_none() {
                total_problems += 1;
                let msg = "Admin user account does not have a corresponding non-admin account according to other_mails property";
                warn!(
                    principal_id = %principal.id,
                    principal_name = principal.display_name,
                    principal_other_mails = principal.other_mails.iter().join("; "),
                    "{}", msg,
                );
                *message_counts.entry(msg).or_insert(0) += 1;
            }
        }
    }

    // Audit resource groups that are missing mandatory tags (TODO)

    // Emit summary
    if total_problems > 0 {
        warn!(
            total_problems,
            message_counts = ?message_counts,
            "Found potential problems in Azure"
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
        info!("No potential problems found in Azure");
    }
    let elapsed = start.elapsed();
    info!(
        elapsed_ms = elapsed.as_millis(),
        "Finished audit in {}",
        humantime::format_duration(elapsed)
    );
    Ok(())
}
