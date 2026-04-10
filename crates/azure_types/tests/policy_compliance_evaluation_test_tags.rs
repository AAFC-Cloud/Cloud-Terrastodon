use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_azure_types::AzurePolicyDefinitionParametersSupplied;
use cloud_terrastodon_azure_types::PolicyDefinition;
use cloud_terrastodon_azure_types::ResourceGroup;

const REQUIRE_A_TAG_ON_RESOURCE_GROUPS_POLICY_DEFINITION_JSON_STR: &str = r#"
{
  "properties": {
    "displayName": "Require a tag on resource groups",
    "policyType": "BuiltIn",
    "mode": "All",
    "description": "Enforces existence of a tag on resource groups.",
    "metadata": {
      "version": "1.0.0",
      "category": "Tags"
    },
    "version": "1.0.0",
    "parameters": {
      "tagName": {
        "type": "String",
        "metadata": {
          "displayName": "Tag Name",
          "description": "Name of the tag, such as 'environment'"
        }
      }
    },
    "policyRule": {
      "if": {
        "allOf": [
          {
            "field": "type",
            "equals": "Microsoft.Resources/subscriptions/resourceGroups"
          },
          {
            "field": "[concat('tags[', parameters('tagName'), ']')]",
            "exists": "false"
          }
        ]
      },
      "then": {
        "effect": "deny"
      }
    }
  },
  "id": "/providers/Microsoft.Authorization/policyDefinitions/96670d01-0a4d-4649-9c89-2d3abc0a5025/versions/1.0.0",
  "type": "Microsoft.Authorization/policyDefinitions/versions",
  "name": "1.0.0"
}
"#;

#[test]
#[ignore] // todo: finish this
fn tag_policy_compliance_evaluation() -> eyre::Result<()> {
    // Load the policy
    let require_a_tag_on_resource_groups_policy_definition: PolicyDefinition =
        serde_json::from_str(REQUIRE_A_TAG_ON_RESOURCE_GROUPS_POLICY_DEFINITION_JSON_STR)?;

    // Create a resource group without tags
    let bytes = [0u8; 100];
    let mut unstructured = Unstructured::new(&bytes);
    let mut resource_group = ResourceGroup::arbitrary(&mut unstructured)?;
    resource_group.tags.clear();

    // Describe policy parameters
    let parameters: AzurePolicyDefinitionParametersSupplied = [("tagName", "environment")].into();
    let compliance_result = require_a_tag_on_resource_groups_policy_definition
        .evaluate_compliance(&parameters, &resource_group);

    assert!(
        compliance_result.is_err(),
        "Expected non-compliance due to missing tag"
    );

    Ok(())
}
