use super::azure_devops_license_summary_row::AzureDevOpsLicenseSummaryRow;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::io::Write;

/// License inventory and its estimated monthly cost in Canadian dollars.
#[derive(Facet)]
pub struct AzureDevOpsLicenseSummary {
    pub org: AzureDevOpsOrganizationUrl,
    pub rows: Vec<AzureDevOpsLicenseSummaryRow>,
    pub total_users: usize,
    pub total_monthly_cost_cad: f64,
}

impl AzureDevOpsLicenseSummary {
    pub fn write_text(
        &self,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        writeln!(writer, "Azure DevOps License Summary")?;
        writeln!(writer, "Organization: {}", self.org)?;
        writeln!(writer, "Users: {}\n", self.total_users)?;
        let license_width = self
            .rows
            .iter()
            .map(|row| row.license.to_string().len())
            .max()
            .unwrap_or(7)
            .max(7);
        writeln!(
            writer,
            "{:<license_width$}  {:>8}  {:>12}  {:>12}",
            "License", "Count", "CAD/User", "CAD/Month"
        )?;
        for row in &self.rows {
            cancellation.bail_if_cancelled()?;
            writeln!(
                writer,
                "{:<license_width$}  {:>8}  {:>12}  {:>12}",
                row.license.to_string(),
                row.count,
                format!("${:.2}", row.cost_per_user_cad),
                format!("${:.2}", row.total_cost_cad)
            )?;
        }
        writeln!(
            writer,
            "{:<license_width$}  {:>8}  {:>12}  {:>12}",
            "TOTAL",
            self.total_users,
            "-",
            format!("${:.2}", self.total_monthly_cost_cad)
        )?;
        Ok(())
    }
}
