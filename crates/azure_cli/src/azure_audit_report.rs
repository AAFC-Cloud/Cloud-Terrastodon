use crate::AzureAuditFinding;
use arbitrary::Arbitrary;
use cloud_terrastodon_command::CancellationScope;
use facet::Facet;
use std::collections::BTreeMap;
use std::io::Write;

/// Combined findings; all output formats render this same completed audit result.
#[derive(Debug, Arbitrary, Facet)]
pub struct AzureAuditReport {
    pub findings: Vec<AzureAuditFinding>,
}

impl AzureAuditReport {
    pub fn write_text(
        &self,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        let mut counts = BTreeMap::new();
        for finding in &self.findings {
            cancellation.bail_if_cancelled()?;
            finding.write_text(writer, cancellation)?;
            *counts.entry(finding.message()).or_insert(0_usize) += 1;
        }
        if self.findings.is_empty() {
            writeln!(writer, "No potential problems found in Azure")?;
        } else {
            writeln!(
                writer,
                "Found {} potential problems in Azure",
                self.findings.len()
            )?;
            writeln!(writer, "Potential monthly cost waste: $0.00 CAD")?;
            for (message, count) in counts {
                writeln!(writer, "{count}: {message}")?;
            }
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureAuditReport);
cloud_terrastodon_registry::register_arbitrary!(AzureAuditReport);
