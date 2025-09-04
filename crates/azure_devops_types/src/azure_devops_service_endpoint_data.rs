use crate::prelude::ServiceEndpointAzureRMData;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum AzureDevOpsServiceEndpointData {
    AzureRM(ServiceEndpointAzureRMData),
    Other(Value),
}

impl Serialize for AzureDevOpsServiceEndpointData {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AzureDevOpsServiceEndpointData::AzureRM(x) => x.serialize(ser),
            AzureDevOpsServiceEndpointData::Other(v) => v.serialize(ser),
        }
    }
}
