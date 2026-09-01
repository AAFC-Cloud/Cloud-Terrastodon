use cloud_terrastodon_command::CommandBuilder;

pub const AZURE_DEVOPS_RESOURCE_ID: &str = "499b84ac-1321-427f-aa17-267ca6975798";
pub const AZURE_DEVOPS_DELEGATED_SCOPE: &str =
    "499b84ac-1321-427f-aa17-267ca6975798/user_impersonation";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AzureRestResource {
    MicrosoftGraph,
    AzureResourceManager,
    AzureDevOps,
}

impl AzureRestResource {
    pub fn scope(self) -> &'static str {
        crate::workload_identity::resource_scope(self)
    }

    /// The delegated permission requested by the interactive browser flow.
    ///
    /// Workload identity uses `/.default` and application permissions. A
    /// delegated browser token must request a concrete scope instead, so this
    /// intentionally remains separate from [`Self::scope`].
    pub fn delegated_scope(self) -> &'static str {
        match self {
            AzureRestResource::MicrosoftGraph => "https://graph.microsoft.com/User.Read",
            AzureRestResource::AzureResourceManager => {
                "https://management.azure.com/user_impersonation"
            }
            AzureRestResource::AzureDevOps => AZURE_DEVOPS_DELEGATED_SCOPE,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::AzureRestResource;

    #[test]
    fn delegated_scopes_are_resource_specific() {
        assert_eq!(
            AzureRestResource::MicrosoftGraph.delegated_scope(),
            "https://graph.microsoft.com/User.Read"
        );
        assert_eq!(
            AzureRestResource::AzureResourceManager.delegated_scope(),
            "https://management.azure.com/user_impersonation"
        );
        assert_eq!(
            AzureRestResource::AzureDevOps.delegated_scope(),
            super::AZURE_DEVOPS_DELEGATED_SCOPE
        );
    }
}
