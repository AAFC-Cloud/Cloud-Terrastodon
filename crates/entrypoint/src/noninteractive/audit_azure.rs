use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions_and_assignments;
use std::collections::HashMap;
use tokio::try_join;
use tracing::info;
use tracing::warn;

#[allow(unused_mut)]
#[allow(unused)]
pub async fn audit_azure() -> eyre::Result<()> {

    // TODO: audit admin accounts without corresponding user accounts should be disabled
    // TODO: audit admin accounts without corresponding user accounts should be deleted
    info!("Fetching a buncha information...");

    let mut total_problems = 0;
    let mut total_cost_waste_cad = 0.00;
    let mut message_counts: HashMap<&'static str, usize> = HashMap::new();

    let (rbac, principals) = try_join!(
        fetch_all_role_definitions_and_assignments(),
        fetch_all_principals()
    )?;

    // Identify role assignments for which the principal is unknwon
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
    Ok(())
}
