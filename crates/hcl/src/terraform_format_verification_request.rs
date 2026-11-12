use crate::AsFormattedHclString;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;

/// Operator verification of the configured Terraform formatter using supplied HCL.
///
/// This resolves local tool configuration and executes `terraform fmt -`; it
/// therefore belongs to explicit runtime verification, not an ambient Cargo test.
#[derive(Debug, Facet)]
pub struct TerraformFormatVerificationRequest {
    pub input: String,
}

impl ExecutableRequest for TerraformFormatVerificationRequest {
    type Output = ();
    type Error = eyre::Report;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()>
    where
        Self: 'a,
    {
        Box::pin(async move {
            self.input
                .as_formatted_hcl_string(executor.cancellation())
                .await?;
            Ok(())
        })
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(TerraformFormatVerificationRequest => (), name = "terraform-format", effects = [Read], uncached);
