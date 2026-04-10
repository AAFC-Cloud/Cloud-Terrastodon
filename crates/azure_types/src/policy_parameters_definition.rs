use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct AzurePolicyDefinitionParametersDefinition(
    pub HashMap<String, AzurePolicyDefinitionParametersDefinitionValue>,
);

impl Deref for AzurePolicyDefinitionParametersDefinition {
    type Target = HashMap<String, AzurePolicyDefinitionParametersDefinitionValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AzurePolicyDefinitionParametersDefinitionValue {
    pub allowed_values: Option<serde_json::Value>, // todo: harden this
    pub default_value: Option<serde_json::Value>,  // todo: harden this
    pub metadata: Option<serde_json::Value>,       // todo: harden this
    pub schema: Option<serde_json::Value>,         // todo: harden this
    pub r#type: String,
}
