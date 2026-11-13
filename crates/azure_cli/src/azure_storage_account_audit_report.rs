use crate::AzureAuditFinding;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CancellationScope;
use facet::Facet;
use std::io::Write;

/// Storage anonymous-access findings from the accounts visible to the selected identity.
/// Inconclusive observations remain findings instead of being reported as private.
#[derive(Debug, Arbitrary, Facet)]
pub struct AzureStorageAccountAuditReport {
    pub accounts_observed: usize,
    pub findings: Vec<AzureAuditFinding>,
}

impl AzureStorageAccountAuditReport {
    pub fn write_text(
        &self,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        writeln!(
            writer,
            "Observed {} storage accounts",
            self.accounts_observed
        )?;
        for finding in &self.findings {
            cancellation.bail_if_cancelled()?;
            finding.write_text(writer, cancellation)?;
        }
        if self.findings.is_empty() {
            writeln!(
                writer,
                "No unused account-level anonymous blob access found"
            )?;
        } else {
            writeln!(
                writer,
                "{} storage audit findings (including inconclusive observations)",
                self.findings.len()
            )?;
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountAuditReport);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountAuditReport);
