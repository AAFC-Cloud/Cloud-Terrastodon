use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AzureDevOpsServiceEndpointOwner {
    Library,
}
