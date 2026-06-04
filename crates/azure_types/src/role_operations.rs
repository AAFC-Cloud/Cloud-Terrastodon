use crate::RolePermissionAction;
use crate::serde_helpers::deserialize_optional_bool_from_bool_or_string;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationsMetadata {
    pub display_name: Option<String>,
    pub id: String,
    pub name: String,
    pub operations: Vec<AzureProviderOperation>,
    pub resource_types: Vec<AzureProviderOperationResourceType>,
    #[serde(rename = "type")]
    pub kind: String,
}

impl AzureProviderOperationsMetadata {
    #[must_use]
    pub fn flatten_operations(&self) -> Vec<AzureRoleOperation> {
        let mut flattened = self
            .operations
            .iter()
            .cloned()
            .map(|operation| AzureRoleOperation {
                provider_name: self.name.clone(),
                provider_display_name: self.display_name.clone(),
                resource_type_name: None,
                resource_type_display_name: None,
                name: operation.name,
                display_name: operation.display_name,
                description: operation.description,
                origin: operation.origin,
                properties: operation.properties,
                is_data_action: operation.is_data_action,
            })
            .collect::<Vec<_>>();

        flattened.extend(self.resource_types.iter().flat_map(|resource_type| {
            resource_type
                .operations
                .iter()
                .cloned()
                .map(|operation| AzureRoleOperation {
                    provider_name: self.name.clone(),
                    provider_display_name: self.display_name.clone(),
                    resource_type_name: Some(resource_type.name.clone()),
                    resource_type_display_name: resource_type.display_name.clone(),
                    name: operation.name,
                    display_name: operation.display_name,
                    description: operation.description,
                    origin: operation.origin,
                    properties: operation.properties,
                    is_data_action: operation.is_data_action,
                })
        }));

        flattened
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationResourceType {
    pub name: String,
    pub display_name: Option<String>,
    pub operations: Vec<AzureProviderOperation>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperation {
    pub name: RolePermissionAction,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub origin: Option<String>,
    pub properties: Option<AzureProviderOperationProperties>,
    pub is_data_action: bool,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationProperties {
    #[serde(alias = "ServiceSpecification")]
    pub service_specification: Option<AzureProviderOperationServiceSpecification>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationServiceSpecification {
    #[serde(alias = "LegacyMetricSpecifications")]
    pub legacy_metric_specifications: Option<Vec<AzureProviderOperationMetricSpecification>>,
    #[serde(alias = "MetricSpecifications")]
    pub metric_specifications: Option<Vec<AzureProviderOperationMetricSpecification>>,
    #[serde(alias = "LogSpecifications")]
    pub log_specifications: Option<Vec<AzureProviderOperationLogSpecification>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationMetricSpecification {
    pub aggregation_type: String,
    pub availabilities: Option<Vec<AzureProviderOperationAvailabilitySpecification>>,
    pub category: Option<String>,
    pub delegate_metric_name_override: Option<String>,
    pub description: Option<String>,
    pub dimensions: Option<Vec<AzureProviderOperationDimensionSpecification>>,
    pub name: String,
    pub display_name: String,
    pub display_description: Option<String>,
    pub unit: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_bool_from_bool_or_string")]
    pub enable_regional_mdm_account: Option<bool>,
    #[serde(alias = "LockAggregationType")]
    pub lock_aggregation_type: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_bool_from_bool_or_string")]
    pub fill_gap_with_zero: Option<bool>,
    pub internal_metric_name: Option<String>,
    pub is_dimension_required: Option<bool>,
    pub is_internal: Option<bool>,
    pub metric_filter_pattern: Option<String>,
    pub metric_class: Option<String>,
    pub resource_id_dimension_name_override: Option<String>,
    pub source_mdm_account: Option<String>,
    pub source_mdm_namespace: Option<String>,
    pub supported_aggregation_types: Option<Vec<String>>,
    pub supported_time_grain_types: Option<Vec<String>>,
    pub supports_instance_level_aggregation: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationAvailabilitySpecification {
    #[serde(alias = "BlobDuration")]
    pub blob_duration: Option<String>,
    #[serde(alias = "Retention")]
    pub retention: Option<String>,
    #[serde(alias = "TimeGrain")]
    pub time_grain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationDimensionSpecification {
    #[serde(alias = "DisplayName")]
    pub display_name: Option<String>,
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "DefaultDimensionValues")]
    pub default_dimension_values: Option<Vec<AzureProviderOperationDimensionDefaultValue>>,
    #[serde(alias = "InternalName")]
    pub internal_name: Option<String>,
    #[serde(alias = "IsHidden")]
    pub is_hidden: Option<bool>,
    #[serde(alias = "ToBeExportedForCustomer")]
    pub to_be_exported_for_customer: Option<bool>,
    #[serde(alias = "ToBeExportedForShoebox")]
    pub to_be_exported_for_shoebox: Option<bool>,
    #[serde(alias = "ToBeExportedToShoebox")]
    pub to_be_exported_to_shoebox: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationDimensionDefaultValue {
    #[serde(alias = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct AzureProviderOperationLogSpecification {
    #[serde(alias = "BlobDuration")]
    pub blob_duration: Option<String>,
    #[serde(alias = "CategoryGroups")]
    pub category_groups: Option<Vec<String>>,
    #[serde(alias = "Description")]
    pub description: Option<String>,
    #[serde(alias = "LogFilterPattern")]
    pub log_filter_pattern: Option<String>,
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "DisplayName")]
    pub display_name: String,
}

#[derive(Debug, Serialize, Eq, PartialEq, Clone)]
pub struct AzureRoleOperation {
    pub provider_name: String,
    pub provider_display_name: Option<String>,
    pub resource_type_name: Option<String>,
    pub resource_type_display_name: Option<String>,
    pub name: RolePermissionAction,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub origin: Option<String>,
    pub properties: Option<AzureProviderOperationProperties>,
    pub is_data_action: bool,
}

impl std::fmt::Display for AzureRoleOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(display_name) = &self.display_name {
            f.write_str(display_name)?;
            f.write_str(" [")?;
            std::fmt::Display::fmt(&self.name, f)?;
            f.write_str("]")?;
        } else {
            std::fmt::Display::fmt(&self.name, f)?;
        }

        f.write_str(" - ")?;
        f.write_str(&self.provider_name)?;

        if let Some(resource_type_name) = &self.resource_type_name {
            f.write_str(" / ")?;
            f.write_str(resource_type_name)?;
        }

        Ok(())
    }
}

#[must_use]
pub fn flatten_role_operations(
    providers: impl IntoIterator<Item = impl std::borrow::Borrow<AzureProviderOperationsMetadata>>,
) -> Vec<AzureRoleOperation> {
    providers
        .into_iter()
        .flat_map(|provider| provider.borrow().flatten_operations())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserializes_provider_operations_with_nested_properties() -> eyre::Result<()> {
        let provider: AzureProviderOperationsMetadata = serde_json::from_value(json!({
            "displayName": "Domain Services Resource Provider",
            "id": "/providers/Microsoft.Authorization/providerOperations/Microsoft.AAD",
            "name": "Microsoft.AAD",
            "operations": [
                {
                    "name": "Microsoft.AAD/register/action",
                    "displayName": "Subscription Registration Action",
                    "description": "Subscription Registration Action",
                    "origin": "user,system",
                    "properties": null,
                    "isDataAction": false
                },
                {
                    "name": "Microsoft.AAD/unregister/action",
                    "displayName": "Unregister Domain Service",
                    "description": "Unregister Domain Service",
                    "origin": "user,system",
                    "properties": {},
                    "isDataAction": false
                }
            ],
            "resourceTypes": [
                {
                    "name": "domainServices/providers/Microsoft.Insights/logDefinitions",
                    "displayName": "Domain Service Type",
                    "operations": [
                        {
                            "name": "Microsoft.AAD/domainServices/providers/Microsoft.Insights/logDefinitions/read",
                            "displayName": "Gets the available logs for Domain Service",
                            "description": "Gets the available logs for Domain Service",
                            "origin": "system",
                            "properties": {
                                "serviceSpecification": {
                                    "logSpecifications": [
                                        {
                                            "name": "SystemSecurity",
                                            "displayName": "SystemSecurity",
                                            "blobDuration": "PT1H"
                                        }
                                    ]
                                }
                            },
                            "isDataAction": false
                        }
                    ]
                },
                {
                    "name": "domainServices/providers/Microsoft.Insights/metricDefinitions",
                    "displayName": "Domain Service Type",
                    "operations": [
                        {
                            "name": "Microsoft.AAD/domainServices/providers/Microsoft.Insights/metricDefinitions/read",
                            "displayName": "Metrics for Domain Service",
                            "description": "Gets metrics for Domain Service",
                            "origin": "system",
                            "properties": {
                                "serviceSpecification": {
                                    "metricSpecifications": [
                                        {
                                            "name": "\\\\DNS\\\\Total Query Received/sec",
                                            "displayName": "DNS - Total Query Received/sec",
                                            "displayDescription": "Metric description",
                                            "unit": "CountPerSecond",
                                            "aggregationType": "Average",
                                            "fillGapWithZero": true
                                        }
                                    ]
                                }
                            },
                            "isDataAction": false
                        }
                    ]
                },
                {
                    "name": "Operations",
                    "displayName": "",
                    "operations": [
                        {
                            "name": "Microsoft.AAD/Operations/read",
                            "displayName": null,
                            "description": null,
                            "origin": "user,system",
                            "properties": null,
                            "isDataAction": false
                        }
                    ]
                }
            ],
            "type": "Microsoft.Authorization/providerOperations"
        }))?;

        assert_eq!(provider.name, "Microsoft.AAD");
        assert_eq!(provider.operations.len(), 2);
        assert_eq!(provider.resource_types.len(), 3);
        assert_eq!(provider.kind, "Microsoft.Authorization/providerOperations");

        let flattened = provider.flatten_operations();
        assert_eq!(flattened.len(), 5);
        assert_eq!(flattened[0].resource_type_name, None);
        let metric_definition = flattened
            .iter()
            .find(|operation| {
                operation.name
                    == RolePermissionAction::new(
                        "Microsoft.AAD/domainServices/providers/Microsoft.Insights/metricDefinitions/read",
                    )
            })
            .expect("expected flattened metric definition operation");
        assert_eq!(
            metric_definition.resource_type_name.as_deref(),
            Some("domainServices/providers/Microsoft.Insights/metricDefinitions")
        );
        assert!(metric_definition.properties.is_some());
        Ok(())
    }
}
