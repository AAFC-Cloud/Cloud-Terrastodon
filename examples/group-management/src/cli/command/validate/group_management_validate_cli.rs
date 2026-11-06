use crate::ValidationReport;
#[cfg(test)]
use arbitrary::Arbitrary;
use cloud_terrastodon::app::CliOutput;
use cloud_terrastodon::app::Result;
use cloud_terrastodon::app::facet;
use cloud_terrastodon::app::figue;
use facet::Facet;
use std::path::PathBuf;

/// Check the local desired-state schema and reconciliation policy.
#[derive(Facet)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct GroupManagementValidateArgs {
    /// JSON array describing existing groups, their tenants, and managed relationships.
    #[facet(figue::named, figue::label = "FILE")]
    pub desired: PathBuf,
}

impl GroupManagementValidateArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        Ok(CliOutput::facet(ValidationReport::try_from_path(
            &self.desired,
        )?))
    }
}
