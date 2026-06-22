use facet_json::RawJson;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, facet::Facet)]
pub struct AzurePolicyDefinitionPolicyRule {
    pub r#if: AzurePolicyDefinitionPolicyRuleIfBlock,
    pub then: AzurePolicyDefinitionPolicyRuleEffectBlock,
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
pub struct AzurePolicyDefinitionPolicyRuleEffectBlock {
    pub effect: AzurePolicyDefinitionPolicyRuleEffect,
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum AzurePolicyDefinitionPolicyRuleEffect {
    Audit,
    Deny,
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[facet(opaque, proxy = RawJson<'static>)]
#[repr(C)]
pub enum AzurePolicyDefinitionPolicyRuleIfBlock {
    AllOf(AzurePolicyDefinitionPolicyRuleIfBlockAllOf),
    AnyOf(AzurePolicyDefinitionPolicyRuleIfBlockAnyOf),
    Equals(AzurePolicyDefinitionPolicyRuleIfBlockEquals),
    FieldIn(AzurePolicyDefinitionPolicyRuleIfBlockFieldIn),
    Other(HashMap<String, RawJson<'static>>),
}

impl TryFrom<RawJson<'static>> for AzurePolicyDefinitionPolicyRuleIfBlock {
    type Error = eyre::Error;

    fn try_from(value: RawJson<'static>) -> Result<Self, Self::Error> {
        let obj = facet_json::from_str::<HashMap<String, RawJson<'static>>>(value.as_str())?;

        if let Some(all_of) = obj.get("allOf") {
            let all_of =
                AzurePolicyDefinitionPolicyRuleIfBlockAllOf(facet_json::from_str(all_of.as_str())?);
            Ok(Self::AllOf(all_of))
        } else if let Some(any_of) = obj.get("anyOf") {
            let any_of =
                AzurePolicyDefinitionPolicyRuleIfBlockAnyOf(facet_json::from_str(any_of.as_str())?);
            Ok(Self::AnyOf(any_of))
        } else if obj.contains_key("field") {
            if obj.contains_key("equals") {
                let equals_block = facet_json::from_str(value.as_str())?;
                Ok(Self::Equals(equals_block))
            } else if obj.contains_key("in") {
                let field_in = facet_json::from_str(value.as_str())?;
                Ok(Self::FieldIn(field_in))
            } else {
                Err(eyre::eyre!(
                    "Expected an object with 'field' to also have either 'equals' or 'in'",
                ))
            }
        } else {
            Ok(Self::Other(obj))
        }
    }
}

impl TryFrom<&AzurePolicyDefinitionPolicyRuleIfBlock> for RawJson<'static> {
    type Error = eyre::Error;

    fn try_from(value: &AzurePolicyDefinitionPolicyRuleIfBlock) -> Result<Self, Self::Error> {
        let json = match value {
            AzurePolicyDefinitionPolicyRuleIfBlock::AllOf(all_of) => {
                let mut object = HashMap::new();
                object.insert(
                    "allOf".to_string(),
                    RawJson::from_owned(facet_json::to_string(&all_of.0)?),
                );
                facet_json::to_string(&object)?
            }
            AzurePolicyDefinitionPolicyRuleIfBlock::AnyOf(any_of) => {
                let mut object = HashMap::new();
                object.insert(
                    "anyOf".to_string(),
                    RawJson::from_owned(facet_json::to_string(&any_of.0)?),
                );
                facet_json::to_string(&object)?
            }
            AzurePolicyDefinitionPolicyRuleIfBlock::Equals(equals_block) => {
                facet_json::to_string(equals_block)?
            }
            AzurePolicyDefinitionPolicyRuleIfBlock::FieldIn(field_in) => {
                facet_json::to_string(field_in)?
            }
            AzurePolicyDefinitionPolicyRuleIfBlock::Other(value) => facet_json::to_string(value)?,
        };
        Ok(RawJson::from_owned(json))
    }
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockAllOf(
    pub Vec<AzurePolicyDefinitionPolicyRuleIfBlock>,
);

#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockAnyOf(
    pub Vec<AzurePolicyDefinitionPolicyRuleIfBlock>,
);

#[derive(Debug, PartialEq, Eq, facet::Facet)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockEquals {
    pub equals: String,
    pub field: String,
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
pub struct AzurePolicyDefinitionPolicyRuleIfBlockFieldIn {
    pub r#in: HashSet<String>,
    pub field: String,
}

#[cfg(test)]
mod test {
    use crate::AzurePolicyDefinitionPolicyRuleIfBlock;
    use crate::AzurePolicyDefinitionPolicyRuleIfBlockAllOf;
    use crate::AzurePolicyDefinitionPolicyRuleIfBlockEquals;

    #[test]
    pub fn it_works1() -> eyre::Result<()> {
        let json = r#"{
            "equals": "Microsoft.DesktopVirtualization/workspaces",
            "field": "type"
        }"#;
        let deserialized: AzurePolicyDefinitionPolicyRuleIfBlock = facet_json::from_str(json)?;
        assert!(matches!(
            &deserialized,
            AzurePolicyDefinitionPolicyRuleIfBlock::Equals(AzurePolicyDefinitionPolicyRuleIfBlockEquals {
                equals,
                field
            }) if equals == "Microsoft.DesktopVirtualization/workspaces" && field == "type"
        ));
        let reparsed: AzurePolicyDefinitionPolicyRuleIfBlock =
            facet_json::from_str(&facet_json::to_string(&deserialized)?)?;
        assert_eq!(deserialized, reparsed);
        Ok(())
    }

    #[test]
    pub fn it_works2() -> eyre::Result<()> {
        let json = r#"{
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
        }"#;
        let deserialized: AzurePolicyDefinitionPolicyRuleIfBlock = facet_json::from_str(json)?;
        assert!(matches!(
            &deserialized,
            AzurePolicyDefinitionPolicyRuleIfBlock::AllOf(AzurePolicyDefinitionPolicyRuleIfBlockAllOf(all_of)) if all_of.len() == 2
        ));
        Ok(())
    }
}
