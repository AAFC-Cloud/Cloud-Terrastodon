use super::azure_devops_license_update_outcome::AzureDevOpsLicenseUpdateOutcome;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::io::Write;

/// Desired license and per-user observations after verification.
#[derive(Facet)]
pub struct AzureDevOpsLicenseUpdateReport {
    pub desired_license: AzureDevOpsLicenseType,
    pub outcomes: Vec<AzureDevOpsLicenseUpdateOutcome>,
}

impl AzureDevOpsLicenseUpdateReport {
    pub fn write_text(
        &self,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        for outcome in &self.outcomes {
            cancellation.bail_if_cancelled()?;
            let (user, was, now) = match outcome {
                AzureDevOpsLicenseUpdateOutcome::NoChangeNeeded(user) => (user, Some(&self.desired_license), Some(&self.desired_license)),
                AzureDevOpsLicenseUpdateOutcome::SkippedDueToLicenseMatchFailureDueToMissingLicense(user) => (user, None, None),
                AzureDevOpsLicenseUpdateOutcome::SkippedDueToLicenseMatchFailure { user, was } => (user, Some(&was.license), Some(&was.license)),
                AzureDevOpsLicenseUpdateOutcome::Updated { user, was } => (user, was.as_ref().map(|value| &value.license), Some(&self.desired_license)),
                AzureDevOpsLicenseUpdateOutcome::UpdateDidntTakeEffect { user, was, now } => (user, was.as_ref().map(|value| &value.license), Some(&now.license)),
            };
            write!(writer, "{user:64} was ")?;
            match was {
                Some(value) if value == &self.desired_license => {
                    write!(writer, "{value} (as desired)")?
                }
                Some(value) => write!(writer, "{value}")?,
                None => write!(writer, "None")?,
            }
            write!(writer, " now ")?;
            match now {
                Some(value) if value == &self.desired_license => {
                    writeln!(writer, "{value} (as desired)")?
                }
                Some(value) => writeln!(writer, "{value}")?,
                None => writeln!(writer, "None")?,
            }
        }
        Ok(())
    }
}
