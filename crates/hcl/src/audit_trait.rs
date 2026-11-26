use crate::HclProject;
use cloud_terrastodon_relative_location::RelativeLocation;
use std::panic::Location;

/// A finding produced while auditing an HCL project.
pub struct HclAuditProblem {
    pub message: String,
    pub location: RelativeLocation,
}

impl std::fmt::Debug for HclAuditProblem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HclAuditProblem")
            .field("message", &self.message)
            .field("location", &self.location.to_string())
            .finish()
    }
}

impl HclAuditProblem {
    #[track_caller]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: RelativeLocation::from(Location::caller()),
        }
    }
}

#[async_trait::async_trait]
pub trait HclAuditor: Send {
    async fn audit(&mut self, hcl: HclProject) -> eyre::Result<(HclProject, Vec<HclAuditProblem>)>;
}
