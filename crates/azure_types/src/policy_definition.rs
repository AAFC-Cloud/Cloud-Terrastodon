use crate::prelude::PolicyDefinitionId;
use crate::prelude::PolicyDefinitionName;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyDefinition {
    pub id: PolicyDefinitionId,
    pub name: PolicyDefinitionName,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub mode: String,
    pub parameters: Option<HashMap<String, Value>>,
    pub policy_rule: serde_json::Value,
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
impl From<PolicyDefinition> for HCLImportBlock {
    fn from(policy_definition: PolicyDefinition) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: policy_definition.id.expanded_form().to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::PolicyDefinition,
                name: policy_definition.id.expanded_form().sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::PolicyDefinition;
    use crate::prelude::PolicyDefinitionId;
    use crate::prelude::PolicyDefinitionName;
    use crate::prelude::UnscopedPolicyDefinitionId;
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
                    parameters: None,
                    policy_rule: serde_json::json!({}),
                    policy_type: "Custom".to_string(),
                    version: "1.0".to_string(),
                }
            })
            .collect_vec();
        let json = serde_json::to_string_pretty(&policies)?;
        let start = Instant::now();
        let deserialized: Vec<PolicyDefinition> = serde_json::from_str(&json)?;
        let duration = Instant::now().duration_since(start);
        println!(
            "Deserialized {} policy definitions in {:?}",
            count,
            humantime::format_duration(duration)
        );
        assert_eq!(deserialized, policies);
        assert!(
            duration < Duration::from_secs(1),
            "Deserialization took too long"
        );
        Ok(())
    }
}
