use super::azure_devops_license_summary::AzureDevOpsLicenseSummary;
use super::azure_devops_license_summary_row::AzureDevOpsLicenseSummaryRow;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashMap;

/// Summarize Azure DevOps user license entitlements by license type.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserSummaryArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsLicenseEntitlementUserSummaryArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let entitlements = AzureDevOpsUserLicenseEntitlementListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;
        let rows = summarize_licenses(entitlements.iter().map(|entitlement| &entitlement.license));
        let total_users: usize = rows.iter().map(|row| row.count).sum();
        let total_monthly_cost_cad: f64 = rows.iter().map(|row| row.total_cost_cad).sum();

        Ok(CliOutput::facet_with_text(
            AzureDevOpsLicenseSummary {
                org: self.org,
                rows,
                total_users,
                total_monthly_cost_cad,
            },
            AzureDevOpsLicenseSummary::write_text,
        ))
    }
}
fn summarize_licenses<'a>(
    licenses: impl IntoIterator<Item = &'a AzureDevOpsLicenseType>,
) -> Vec<AzureDevOpsLicenseSummaryRow> {
    let mut counts: HashMap<AzureDevOpsLicenseType, usize> = HashMap::new();
    for license in licenses {
        *counts.entry(license.clone()).or_insert(0) += 1;
    }

    let mut rows = counts
        .into_iter()
        .map(|(license, count)| {
            let cost_per_user_cad = license.cost_per_month_cad();
            AzureDevOpsLicenseSummaryRow {
                total_cost_cad: cost_per_user_cad * count as f64,
                cost_per_user_cad,
                license,
                count,
            }
        })
        .collect::<Vec<_>>();

    rows.sort_by(|left, right| {
        right
            .total_cost_cad
            .total_cmp(&left.total_cost_cad)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.license.to_string().cmp(&right.license.to_string()))
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::summarize_licenses;
    use cloud_terrastodon_azure_devops::AzureDevOpsAccountLicenseType;
    use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
    use cloud_terrastodon_azure_devops::AzureDevOpsMsdnLicenseType;

    #[test]
    fn it_groups_and_sorts_license_rows_by_monthly_cost() {
        let licenses = [
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Stakeholder),
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Advanced),
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Express),
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Advanced),
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Express),
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Express),
        ];

        let rows = summarize_licenses(licenses.iter());

        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[0].license,
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Advanced)
        );
        assert_eq!(rows[0].count, 2);
        assert_eq!(
            rows[1].license,
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Express)
        );
        assert_eq!(rows[1].count, 3);
        assert_eq!(
            rows[2].license,
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Stakeholder)
        );
        assert_eq!(rows[2].count, 1);
    }
    #[test]
    fn it_keeps_matching_license_names_in_distinct_families() {
        let account = AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Professional);
        let msdn = AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Professional);
        let licenses = [account.clone(), msdn.clone(), account.clone()];
        let rows = summarize_licenses(licenses.iter());

        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.iter()
                .find(|row| row.license == account)
                .unwrap()
                .count,
            2
        );
        assert_eq!(
            rows.iter().find(|row| row.license == msdn).unwrap().count,
            1
        );
    }
}
