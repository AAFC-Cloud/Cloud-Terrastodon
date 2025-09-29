use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions_and_assignments;
use tokio::try_join;
use tracing::info;
use tracing::warn;

#[allow(unused_mut)]
#[allow(unused)]
pub async fn audit_azure() -> eyre::Result<()> {
    info!("Fetching a buncha information...");

    let mut total_problems = 0;
    let mut total_cost_waste_cad = 0.00;

    let (rbac, principals) = try_join!(
        fetch_all_role_definitions_and_assignments(),
        fetch_all_principals()
    )?;

    // Identify role assignments for which the principal is unknwon
    for (role_assignment, role_definition) in rbac.iter_role_assignments() {
        let principal_id = &role_assignment.principal_id;
        if !principals.contains_key(principal_id) {
            total_problems += 1;
            warn!(
                principal_id = ?principal_id,
                role_definition_name = %role_definition.display_name,
                role_assignment_id = %role_assignment.id.expanded_form(),
                scope = %role_assignment.scope.expanded_form(),
                "Found role assignment with unknown principal",
            );
        }
    }

    // Emit summary
    if total_problems > 0 {
        warn!(total_problems, "Found potential problems in Azure");
        warn!(
            total_cost_waste_cad,
            "Potential monthly cost waste: ${:.2} CAD",
            total_cost_waste_cad
        );
    } else {
        info!("No potential problems found in Azure DevOps");
    }
    Ok(())
}
