use cloud_terrastodon_command::CommandBuilder;

pub const AZURE_DEVOPS_RESOURCE_ID: &str = "499b84ac-1321-427f-aa17-267ca6975798";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AzureRestResource {
    MicrosoftGraph,
    AzureResourceManager,
    AzureDevOps,
}

impl AzureRestResource {
    pub(crate) fn apply_access_token_args(self, cmd: &mut CommandBuilder) {
        match self {
            AzureRestResource::MicrosoftGraph => {
                cmd.args(["--resource-type", "ms-graph"]);
            }
            AzureRestResource::AzureResourceManager => {}
            AzureRestResource::AzureDevOps => {
                cmd.args(["--resource", AZURE_DEVOPS_RESOURCE_ID]);
            }
        }
    }
}
