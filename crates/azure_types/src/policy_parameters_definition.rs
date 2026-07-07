use crate::ArbitraryJson;
use arbitrary::Arbitrary;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq, Default, Arbitrary, facet::Facet)]
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

#[derive(Debug, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePolicyDefinitionParametersDefinitionValue {
    pub allowed_values: Option<ArbitraryJson>,
    pub default_value: Option<ArbitraryJson>,
    pub metadata: Option<ArbitraryJson>,
    pub schema: Option<ArbitraryJson>,
    #[facet(rename = "type")]
    pub r#type: String,
}
