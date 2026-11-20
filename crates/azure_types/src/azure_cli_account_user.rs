use crate::AzureCliAccountUserType;
use facet::Facet;

/// The `user` record in the [Azure CLI account profile](https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/_profile.py).
#[derive(Debug, Eq, PartialEq, Facet)]
pub struct AzureCliAccountUser {
    pub name: String,
    #[facet(rename = "type")]
    pub r#type: AzureCliAccountUserType,
}
