use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use std::collections::HashMap;

/// Summarize Azure DevOps user license entitlements by license type.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserSummaryArgs {}

#[derive(Debug, Clone, PartialEq)]
struct LicenseSummaryRow {
    license: AzureDevOpsLicenseType,
    count: usize,
    cost_per_user_cad: f64,
    total_cost_cad: f64,
}

impl AzureDevOpsLicenseEntitlementUserSummaryArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;
        let rows = summarize_licenses(entitlements.iter().map(|entitlement| &entitlement.license));
        let total_users: usize = rows.iter().map(|row| row.count).sum();
        let total_monthly_cost_cad: f64 = rows.iter().map(|row| row.total_cost_cad).sum();

        let license_width = rows
            .iter()
            .map(|row| row.license.to_string().len())
            .max()
            .unwrap_or("License".len())
            .max("License".len())
            .max("TOTAL".len());
        let count_width = rows
            .iter()
            .map(|row| row.count.to_string().len())
            .max()
            .unwrap_or("Count".len())
            .max("Count".len())
            .max(total_users.to_string().len());
        let cost_per_user_width = rows
            .iter()
            .map(|row| format_currency(row.cost_per_user_cad).len())
            .max()
            .unwrap_or("CAD/User".len())
            .max("CAD/User".len());
        let total_cost_width = rows
            .iter()
            .map(|row| format_currency(row.total_cost_cad).len())
            .max()
            .unwrap_or("CAD/Month".len())
            .max("CAD/Month".len())
            .max(format_currency(total_monthly_cost_cad).len());

        println!("{}", "Azure DevOps License Summary".magenta().bold());
        println!(
            "{} {}",
            "Organization:".cyan().bold(),
            org_url.to_string().bright_blue()
        );
        println!(
            "{} {}",
            "Users:".cyan().bold(),
            total_users.to_string().yellow().bold()
        );
        println!();

        let separator = format!(
            "{:-<license_width$}  {:-<count_width$}  {:-<cost_per_user_width$}  {:-<total_cost_width$}",
            "", "", "", "",
        );

        println!("{}", separator.dimmed());
        println!(
            "{}  {}  {}  {}",
            format!("{:<license_width$}", "License").cyan().bold(),
            format!("{:>count_width$}", "Count").cyan().bold(),
            format!("{:>cost_per_user_width$}", "CAD/User")
                .cyan()
                .bold(),
            format!("{:>total_cost_width$}", "CAD/Month").cyan().bold(),
        );
        println!("{}", separator.dimmed());

        for row in &rows {
            let license = pad_right_colored(
                &row.license.to_string(),
                license_width,
                paint_license(&row.license, row.license.to_string()),
            );
            let count = pad_left_colored(
                &row.count.to_string(),
                count_width,
                paint_count(row.count, row.count.to_string()),
            );
            let cost_per_user_value = format_currency(row.cost_per_user_cad);
            let cost_per_user = pad_left_colored(
                &cost_per_user_value,
                cost_per_user_width,
                paint_per_user_cost(row.cost_per_user_cad, &cost_per_user_value),
            );
            let total_cost_value = format_currency(row.total_cost_cad);
            let total_cost = pad_left_colored(
                &total_cost_value,
                total_cost_width,
                paint_total_cost(row.total_cost_cad, &total_cost_value),
            );

            println!("{}  {}  {}  {}", license, count, cost_per_user, total_cost,);
        }

        println!("{}", separator.dimmed());
        println!(
            "{}  {}  {}  {}",
            format!("{:<license_width$}", "TOTAL")
                .bright_magenta()
                .bold(),
            format!("{:>count_width$}", total_users)
                .bright_magenta()
                .bold(),
            format!("{:>cost_per_user_width$}", "-")
                .bright_magenta()
                .bold(),
            format!(
                "{:>total_cost_width$}",
                format_currency(total_monthly_cost_cad)
            )
            .bright_magenta()
            .bold(),
        );

        Ok(())
    }
}

fn summarize_licenses<'a>(
    licenses: impl IntoIterator<Item = &'a AzureDevOpsLicenseType>,
) -> Vec<LicenseSummaryRow> {
    let mut counts: HashMap<AzureDevOpsLicenseType, usize> = HashMap::new();
    for license in licenses {
        *counts.entry(license.clone()).or_insert(0) += 1;
    }

    let mut rows = counts
        .into_iter()
        .map(|(license, count)| {
            let cost_per_user_cad = license.cost_per_month_cad();
            LicenseSummaryRow {
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

fn format_currency(amount: f64) -> String {
    format!("${amount:.2}")
}

fn pad_right_colored(raw: &str, width: usize, colored: String) -> String {
    let padding = width.saturating_sub(raw.len());
    format!("{colored}{:padding$}", "")
}

fn pad_left_colored(raw: &str, width: usize, colored: String) -> String {
    let padding = width.saturating_sub(raw.len());
    format!("{:padding$}{colored}", "")
}

fn paint_license(license: &AzureDevOpsLicenseType, value: String) -> String {
    match license {
        AzureDevOpsLicenseType::AccountAdvanced => value.bright_red().bold().to_string(),
        AzureDevOpsLicenseType::AccountExpress => value.bright_yellow().bold().to_string(),
        AzureDevOpsLicenseType::AccountStakeholder => value.bright_green().bold().to_string(),
        AzureDevOpsLicenseType::MsdnEligible
        | AzureDevOpsLicenseType::MsdnEnterprise
        | AzureDevOpsLicenseType::MsdnProfessional => value.bright_blue().bold().to_string(),
        AzureDevOpsLicenseType::None => value.dimmed().to_string(),
        AzureDevOpsLicenseType::Other(_) => value.magenta().bold().to_string(),
    }
}

fn paint_count(count: usize, value: String) -> String {
    match count {
        0 => value.dimmed().to_string(),
        1..=4 => value.green().bold().to_string(),
        5..=19 => value.yellow().bold().to_string(),
        _ => value.bright_magenta().bold().to_string(),
    }
}

fn paint_per_user_cost(cost: f64, value: &str) -> String {
    match cost {
        0.0 => value.green().to_string(),
        ..=10.0 => value.yellow().bold().to_string(),
        ..=50.0 => value.bright_yellow().bold().to_string(),
        _ => value.bright_red().bold().to_string(),
    }
}

fn paint_total_cost(cost: f64, value: &str) -> String {
    match cost {
        0.0 => value.green().bold().to_string(),
        ..=25.0 => value.yellow().bold().to_string(),
        ..=100.0 => value.bright_yellow().bold().to_string(),
        _ => value.bright_red().bold().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::summarize_licenses;
    use cloud_terrastodon_azure_devops::prelude::AzureDevOpsLicenseType;

    #[test]
    fn it_groups_and_sorts_license_rows_by_monthly_cost() {
        let licenses = vec![
            AzureDevOpsLicenseType::AccountStakeholder,
            AzureDevOpsLicenseType::AccountAdvanced,
            AzureDevOpsLicenseType::AccountExpress,
            AzureDevOpsLicenseType::AccountAdvanced,
            AzureDevOpsLicenseType::AccountExpress,
            AzureDevOpsLicenseType::AccountExpress,
        ];

        let rows = summarize_licenses(licenses.iter());

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].license, AzureDevOpsLicenseType::AccountAdvanced);
        assert_eq!(rows[0].count, 2);
        assert_eq!(rows[1].license, AzureDevOpsLicenseType::AccountExpress);
        assert_eq!(rows[1].count, 3);
        assert_eq!(rows[2].license, AzureDevOpsLicenseType::AccountStakeholder);
        assert_eq!(rows[2].count, 1);
    }
}
