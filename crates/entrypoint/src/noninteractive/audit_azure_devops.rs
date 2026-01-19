use chrono::Local;
use chrono::Utc;
use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseKind;
use cloud_terrastodon_azure_devops::prelude::LastAccessedDate;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_groups_for_member;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_test_plans;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_test_suites;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use itertools::Itertools;
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;
use tracing::warn;

pub async fn audit_azure_devops(
    test_license_inactivity_threshold: Duration,
    paid_license_inactivity_threshold: Duration,
) -> eyre::Result<()> {
    let test_license_inactivity_threshold = chrono::Duration::from_std(test_license_inactivity_threshold)?;
    let paid_license_inactivity_threshold = chrono::Duration::from_std(paid_license_inactivity_threshold)?;

    warn!("Use `cloud_terrastodon clean` to wipe the cache if you think results are stale.");
    info!("Fetching a buncha information...");

    let mut total_problems = 0;
    let mut total_cost_waste_cad = 0.00;
    let mut message_counts: HashMap<String, usize> = HashMap::new();

    let org_url = get_default_organization_url().await?;
    let entitlements = fetch_azure_devops_license_entitlements(&org_url).await?;
    let users_by_principal_name = fetch_all_users()
        .await?
        .into_iter()
        .map(|user| (user.user_principal_name.to_lowercase(), user))
        .collect::<HashMap<_, _>>();

    // Emit a warning for anyone who has not recently accessed azure devops and have greater than stakeholder license
    for entitlement in entitlements
        .iter()
        .filter(|e| e.license != AzureDevOpsLicenseKind::AccountStakeholder)
        .filter(|e| {
            e.assignment_date.max(e.date_created).max(e.last_updated)
                < Utc::now() - paid_license_inactivity_threshold
        })
    {
        match entitlement.last_accessed_date {
            LastAccessedDate::Never => {
                let msg = "User has never accessed Azure DevOps but has a paid license; consider downgrading license";
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
                *message_counts.entry(msg.to_string()).or_insert(0) += 1;
            }
            LastAccessedDate::Some(date) if date < Utc::now() - paid_license_inactivity_threshold => {
                let msg = format!(
                    "User has not accessed Azure DevOps in the last {} days but has a paid license; consider downgrading license",
                    paid_license_inactivity_threshold.num_days()
                );
                warn!(
                    user_display_name = %entitlement.user.display_name,
                    user_unique_name = %entitlement.user.unique_name,
                    last_accessed_date = %date.to_rfc3339(),
                    last_accessed_ago = humantime::format_duration({
                        let mut duration = (Utc::now() - date)
                            .to_std()
                            .expect("Time went backwards");
                        duration = Duration::from_hours(duration.as_secs() / 3600);
                        duration
                    })
                    .to_string(),
                    last_accessed_ago_days = ((Utc::now() - date).num_days()),
                    license = ?entitlement.license,
                    status = ?entitlement.status,
                    cost_per_month_cad = %entitlement.license.cost_per_month_cad(),
                    "{}", msg
                );
                total_problems += 1;
                total_cost_waste_cad += entitlement.license.cost_per_month_cad();
                *message_counts.entry(msg).or_insert(0) += 1;
            }
            _ => {}
        }
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
                user_descriptor = %entitlement.user.descriptor,
                license = ?entitlement.license,
                status = ?entitlement.status,
                cost_per_month_cad = %entitlement.license.cost_per_month_cad(),
                "{}", msg
            );
            total_problems += 1;
            total_cost_waste_cad += entitlement.license.cost_per_month_cad();
            *message_counts.entry(msg.to_string()).or_insert(0) += 1;
        }
    }

    // Emit a warning for licenses assigned to admin accounts for which no user account exists
    // blocker TODO: create entra helper to cleanly map between user and admin accounts

    // Identify unused test plan license assignments.
    let test_plan_licenses = entitlements
        .iter()
        .filter(|e| e.license == AzureDevOpsLicenseKind::AccountAdvanced)
        .filter(|e| {
            e.assignment_date.max(e.date_created).max(e.last_updated)
                < Utc::now() - test_license_inactivity_threshold
        })
        .collect_vec();
    info!(
        test_plan_license_entitlement_count = test_plan_licenses.len(),
        "Analyzing test plan license usage",
    );

    let projects = fetch_all_azure_devops_projects(&org_url).await?;
    let project_test_plans = projects
        .iter()
        .map(|project| {
            let org_url = org_url.clone();
            let project_id = project.id.clone();
            async move {
                let plans = fetch_azure_devops_test_plans(&org_url, &project_id).await?;
                Ok((project_id, plans))
            }
        })
        .fold(
            ParallelFallibleWorkQueue::new("fetching azure devops test plans for projects", 4),
            |mut queue, fut| {
                queue.enqueue(fut);
                queue
            },
        )
        .join()
        .await?
        .into_iter()
        .collect::<HashMap<_, _>>();

    let test_suites = project_test_plans
        .iter()
        .flat_map(|(project, plans)| plans.iter().map(|plan| (project.clone(), plan)))
        .map(|(project_id, test_plan)| {
            let org_url = org_url.clone();
            let plan_id = test_plan.id;
            async move {
                let suites =
                    fetch_azure_devops_test_suites(&org_url, &project_id, plan_id.to_string())
                        .await?;
                Ok((project_id, plan_id, suites))
            }
        })
        .fold(
            ParallelFallibleWorkQueue::new(
                "fetching azure devops test suites for project test plans",
                4,
            ),
            |mut queue, fut| {
                queue.enqueue(fut);
                queue
            },
        )
        .join()
        .await?
        .into_iter()
        .map(|(project_id, plan_id, suites)| ((project_id, plan_id), suites))
        .collect::<HashMap<_, _>>();

    let groups_for_test_plan_licensed_users = test_plan_licenses
        .iter()
        .map(|entitlement| {
            let member_id = entitlement.user.descriptor.clone();
            let org_url = org_url.clone();
            async move {
                let groups = fetch_azure_devops_groups_for_member(&org_url, &member_id).await?;
                Ok((member_id, groups))
            }
        })
        .fold(
            ParallelFallibleWorkQueue::new("fetching groups for test plan licensed users", 4),
            |mut queue, fut| {
                queue.enqueue(fut);
                queue
            },
        )
        .join()
        .await?
        .into_iter()
        .collect::<HashMap<_, _>>();

    let groups_for_projects = projects
        .iter()
        .map(|project| {
            let org_url = org_url.clone();
            let project_id = project.id.clone();
            async move {
                let groups = fetch_azure_devops_groups_for_project(&org_url, &project_id).await?;
                Ok((project_id, groups))
            }
        })
        .fold(
            ParallelFallibleWorkQueue::new("fetching groups for projects", 4),
            |mut queue, fut| {
                queue.enqueue(fut);
                queue
            },
        )
        .join()
        .await?
        .into_iter()
        .collect::<HashMap<_, _>>();

    // for each license haver, print their projects and the test plans in those projects (plan name, last date)
    let now = Local::now();
    let test_license_inactivity_threshold_ago = now - test_license_inactivity_threshold;
    let basic_license_inactivity_threshold_ago = now - paid_license_inactivity_threshold;
    info!(
        ?test_license_inactivity_threshold_ago,
        ?basic_license_inactivity_threshold_ago,
        test_license_inactivity_threshold = %humantime::format_duration(test_license_inactivity_threshold.to_std()?),
        basic_license_inactivity_threshold = %humantime::format_duration(paid_license_inactivity_threshold.to_std()?),
        "Using inactivity threshold for license usage audit",
    );
    for test_plan_entitlement in test_plan_licenses {
        // Get the groups for the user
        let Some(user_groups) =
            groups_for_test_plan_licensed_users.get(&test_plan_entitlement.user.descriptor)
        else {
            continue;
        };

        let mut last_used = None;
        let mut project_count = 0;
        let mut test_plan_count = 0;
        let mut test_suite_count = 0;

        for project in &projects {
            // Get the groups for the project
            let Some(project_groups) = groups_for_projects.get(&project.id) else {
                continue;
            };

            // Ensure user is in any of the project groups
            let is_user_in_project_groups = user_groups.iter().any(|user_group| {
                project_groups.iter().any(|project_group| {
                    user_group.container_descriptor == project_group.descriptor
                })
            });
            if !is_user_in_project_groups {
                continue;
            }

            project_count += 1;

            let project_test_plans = project_test_plans.get(&project.id);
            for plan in project_test_plans.into_iter().flatten() {
                test_plan_count += 1;
                last_used =
                    last_used.max(plan.start_date.max(plan.end_date).max(plan.updated_date));
                for suite in test_suites
                    .get(&(project.id.clone(), plan.id))
                    .into_iter()
                    .flatten()
                {
                    test_suite_count += 1;
                    last_used =
                        last_used.max(suite.last_updated_date.max(suite.last_populated_date));
                }
            }
        }

        let license_wasted = last_used
            .filter(|date| date > &test_license_inactivity_threshold_ago)
            .is_none();
        if license_wasted {
            let msg = "User has an Advanced license for Test Plans but has not used any test plans; consider downgrading license";
            warn!(
                user_display_name = %test_plan_entitlement.user.display_name,
                user_unique_name = %test_plan_entitlement.user.unique_name,
                last_used = last_used
                    .map(|date| date.to_string())
                    .as_deref()
                    .unwrap_or("never"),
                last_used_ago = last_used
                    .map(|date| humantime::format_duration({
                        let mut duration = (Utc::now() - date).to_std().expect("Time went backwards");
                        duration = Duration::from_hours(duration.as_secs() / 3600);
                        duration
                    })
                    .to_string())
                    .as_deref()
                    .unwrap_or("N/A"),
                last_used_ago_days = last_used.map(|date| (Utc::now() - date).num_days()),
                license = ?test_plan_entitlement.license,
                status = ?test_plan_entitlement.status,
                cost_per_month_cad = %test_plan_entitlement.license.cost_per_month_cad(),
                project_count,
                test_plan_count,
                test_suite_count,
                "{msg}"
            );
            total_problems += 1;
            total_cost_waste_cad += test_plan_entitlement.license.cost_per_month_cad();
            *message_counts.entry(msg.to_string()).or_insert(0) += 1;
        }
    }

    // Emit summary
    if total_problems > 0 {
        warn!(
            total_problems,
            total_cost_waste_cad,
            "Found potential problems in Azure DevOps; cost waste: ${:.2} CAD",
            total_cost_waste_cad
        );
        // Emit message type summary
        for (msg, count) in &message_counts {
            warn!(count, "{}", msg);
        }
    } else {
        info!("No potential problems found in Azure DevOps");
    }
    Ok(())
}
