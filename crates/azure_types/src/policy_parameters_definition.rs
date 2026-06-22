use facet_json::RawJson;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq, Default, facet::Facet)]
#[facet(transparent)]
pub struct AzurePolicyDefinitionParametersDefinition(
    pub HashMap<String, AzurePolicyDefinitionParametersDefinitionValue>,
);

impl Deref for AzurePolicyDefinitionParametersDefinition {
    type Target = HashMap<String, AzurePolicyDefinitionParametersDefinitionValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePolicyDefinitionParametersDefinitionValue {
    pub allowed_values: Option<RawJson<'static>>, // todo: harden this
    pub default_value: Option<RawJson<'static>>,  // todo: harden this
    pub metadata: Option<RawJson<'static>>,       // todo: harden this
    pub schema: Option<RawJson<'static>>,         // todo: harden this
    #[facet(rename = "type")]
    pub r#type: String,
}
