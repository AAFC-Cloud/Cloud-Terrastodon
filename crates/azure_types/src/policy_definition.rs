use crate::ArbitraryJson;
use crate::AzurePolicyDefinitionParametersDefinition;
use crate::AzurePolicyDefinitionParametersSupplied;
use crate::PolicyDefinitionId;
use crate::PolicyDefinitionName;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use facet::Facet;

#[derive(Debug, PartialEq, Arbitrary, facet::Facet)]
pub struct PolicyDefinition {
    pub id: PolicyDefinitionId,
    pub name: PolicyDefinitionName,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub mode: String,
    #[facet(default)]
    pub parameters: Option<AzurePolicyDefinitionParametersDefinition>,

    // todo: strong type this!
    // todo: strong type this!
    // todo: strong type this!
    // todo: strong type this!
    // todo: strong type this!
    pub policy_rule: ArbitraryJson,

    pub policy_type: String,
    pub version: String,
}

impl AsScope for PolicyDefinition {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &PolicyDefinition {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for PolicyDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("policy definition ")?;
        f.write_str(&self.name)?;
        if let Some(display_name) = self.display_name.as_deref() {
            f.write_str(" (")?;
            f.write_str(display_name)?;
            f.write_str(")")?;
        }
        Ok(())
    }
}
impl From<PolicyDefinition> for HclImportBlock {
    fn from(policy_definition: PolicyDefinition) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: policy_definition.id.expanded_form().to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::PolicyDefinition,
                name: policy_definition.id.expanded_form().sanitize(),
            },
        }
    }
}

impl PolicyDefinition {
    pub fn evaluate_compliance(
        &self,
        _parameters: &AzurePolicyDefinitionParametersSupplied,
        resource: &impl Facet<'static>,
    ) -> eyre::Result<()> {
        // Ensure all parameters are present
        // Convert the resource to JSON for eventual policy evaluation.
        let _json = facet_json::to_string(resource)?;

        todo!();
    }
}

#[cfg(test)]
mod test {
    use crate::PolicyDefinition;
    use crate::PolicyDefinitionId;
    use crate::PolicyDefinitionName;
    use crate::UnscopedPolicyDefinitionId;
    use crate::slug::HasSlug;
    use itertools::Itertools;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    pub fn deserialize_performance() -> eyre::Result<()> {
        let count = 1000;
        let policies = (0..count)
            .map(|i| {
                let id = PolicyDefinitionId::Unscoped(UnscopedPolicyDefinitionId {
                    name: PolicyDefinitionName::new(i.to_string().into()),
                });
                PolicyDefinition {
                    name: id.name().clone(),
                    id,
                    display_name: Some(format!("Policy Definition {}", i)),
                    description: Some(format!("This is policy definition number {}", i)),
                    mode: "All".to_string(),
                    parameters: Some(Default::default()),
                    policy_rule: facet_json::RawJson::from_owned("{}".to_string()).into(),
                    policy_type: "Custom".to_string(),
                    version: "1.0".to_string(),
                }
            })
            .collect_vec();
        let json = facet_json::to_string_pretty(&policies)?;
        let start = Instant::now();
        let deserialized: Vec<PolicyDefinition> = facet_json::from_str(&json)?;
        let duration = Instant::now().duration_since(start);
        assert_eq!(deserialized, policies);
        assert!(duration < Duration::from_secs(2));
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(PolicyDefinition);
cloud_terrastodon_registry::register_arbitrary!(PolicyDefinition);
cloud_terrastodon_registry::register_arbitrary!(Vec<PolicyDefinition>);
