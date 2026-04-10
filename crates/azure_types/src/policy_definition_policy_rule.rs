use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AzurePolicyDefinitionPolicyRule {
    pub r#if: AzurePolicyDefinitionPolicyRuleIfBlock,
    pub then: AzurePolicyDefinitionPolicyRuleEffectBlock,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AzurePolicyDefinitionPolicyRuleEffectBlock {
    pub effect: AzurePolicyDefinitionPolicyRuleEffect,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AzurePolicyDefinitionPolicyRuleEffect {
    Audit,
    Deny,
}

#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum AzurePolicyDefinitionPolicyRuleIfBlock {
    AllOf(AzurePolicyDefinitionPolicyRuleIfBlockAllOf),
    AnyOf(AzurePolicyDefinitionPolicyRuleIfBlockAnyOf),
    Equals(AzurePolicyDefinitionPolicyRuleIfBlockEquals),
    FieldIn(AzurePolicyDefinitionPolicyRuleIfBlockFieldIn),
}
impl Serialize for AzurePolicyDefinitionPolicyRuleIfBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AzurePolicyDefinitionPolicyRuleIfBlock::AllOf(all_of) => all_of.serialize(serializer),
            AzurePolicyDefinitionPolicyRuleIfBlock::AnyOf(any_of) => any_of.serialize(serializer),
            AzurePolicyDefinitionPolicyRuleIfBlock::Equals(equals_block) => {
                equals_block.serialize(serializer)
            }
            AzurePolicyDefinitionPolicyRuleIfBlock::FieldIn(field_in) => {
                field_in.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for AzurePolicyDefinitionPolicyRuleIfBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Value::Object(obj) = serde_json::Value::deserialize(deserializer)? else {
            return Err(serde::de::Error::custom("Expected an object"));
        };

        if let Some(all_of) = obj.get("allOf") {
            let all_of = AzurePolicyDefinitionPolicyRuleIfBlockAllOf::deserialize(all_of.clone())
                .map_err(serde::de::Error::custom)?;
            Ok(Self::AllOf(all_of))
        } else if let Some(any_of) = obj.get("anyOf") {
            let any_of = AzurePolicyDefinitionPolicyRuleIfBlockAnyOf::deserialize(any_of.clone())
                .map_err(serde::de::Error::custom)?;
            Ok(Self::AnyOf(any_of))
        } else if obj.contains_key("field") {
            if obj.contains_key("equals") {
                let equals_block =
                    AzurePolicyDefinitionPolicyRuleIfBlockEquals::deserialize(Value::Object(obj))
                        .map_err(serde::de::Error::custom)?;
                Ok(Self::Equals(equals_block))
            } else if obj.contains_key("in") {
                let field_in =
                    AzurePolicyDefinitionPolicyRuleIfBlockFieldIn::deserialize(Value::Object(obj))
                        .map_err(serde::de::Error::custom)?;
                Ok(Self::FieldIn(field_in))
            } else {
                Err(serde::de::Error::custom(
                    "Expected an object with 'field' to also have either 'equals' or 'in'",
                ))
            }
        } else {
            Err(serde::de::Error::custom(
                "Expected an object with either 'allOf', 'anyOf', or 'field'",
            ))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(transparent)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockAllOf(
    pub BTreeSet<AzurePolicyDefinitionPolicyRuleIfBlock>,
);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(transparent)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockAnyOf(
    pub BTreeSet<AzurePolicyDefinitionPolicyRuleIfBlock>,
);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockEquals {
    pub equals: String,
    pub field: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockFieldIn {
    pub r#in: BTreeSet<String>,
    pub field: String,
}

#[cfg(test)]
mod test {
    use crate::AzurePolicyDefinitionPolicyRuleIfBlock;
    use crate::AzurePolicyDefinitionPolicyRuleIfBlockAllOf;
    use crate::AzurePolicyDefinitionPolicyRuleIfBlockEquals;
    use serde_json::json;

    #[test]
    pub fn it_works1() -> eyre::Result<()> {
        let json = json!({
            "equals": "Microsoft.DesktopVirtualization/workspaces",
            "field": "type"
        });
        let deserialized: AzurePolicyDefinitionPolicyRuleIfBlock =
            serde_json::from_value(json.clone())?;
        assert!(matches!(
            &deserialized,
            AzurePolicyDefinitionPolicyRuleIfBlock::Equals(AzurePolicyDefinitionPolicyRuleIfBlockEquals {
                equals,
                field
            }) if equals == "Microsoft.DesktopVirtualization/workspaces" && field == "type"
        ));
        assert_eq!(serde_json::to_value(&deserialized)?, json,);
        Ok(())
    }

    #[test]
    pub fn it_works2() -> eyre::Result<()> {
        let json = json!({
          "allOf": [
            {
              "field": "type",
              "in": [
                "Microsoft.Resources/subscriptions",
                "Microsoft.Resources/subscriptions/resourceGroups"
              ]
            },
            {
              "equals": "true",
              "value": "false"
            }
          ]
        });
        let deserialized: AzurePolicyDefinitionPolicyRuleIfBlock =
            serde_json::from_value(json.clone())?;
        assert!(matches!(
            &deserialized,
            AzurePolicyDefinitionPolicyRuleIfBlock::AllOf(AzurePolicyDefinitionPolicyRuleIfBlockAllOf(all_of)) if all_of.len() == 2
        ));
        assert_eq!(serde_json::to_value(&deserialized)?, json,);
        Ok(())
    }
}
