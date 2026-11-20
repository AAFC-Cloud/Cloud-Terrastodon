use facet::Facet;

/// The `user.type` discriminator in the [Azure CLI account profile](https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/_profile.py).
#[derive(Debug, Clone, Eq, PartialEq, Facet)]
#[repr(C)]
pub enum AzureCliAccountUserType {
    #[facet(rename = "user")]
    User,
    #[facet(rename = "servicePrincipal")]
    ServicePrincipal,
    #[facet(other)]
    Other(String),
}
