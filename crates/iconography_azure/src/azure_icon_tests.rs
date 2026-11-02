use crate::icon_for_resource_type;
use cloud_terrastodon_azure_resource_types::ResourceType;

#[cfg(feature = "resources")]
#[test]
fn resource_group_aliases_resolve_across_known_and_preserved_resource_types() {
    use crate::AzureResourceGroupIcon;
    use cloud_terrastodon_iconography::SvgIcon;

    for resource_type in [
        ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_RESOURCEGROUPS,
        ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS,
        ResourceType::Other("microsoft.resources/RESOURCEGROUPS".to_owned()),
        ResourceType::Other("Microsoft.Resources/Subscriptions/ResourceGroups".to_owned()),
    ] {
        let icon = icon_for_resource_type(&resource_type).expect("enabled resource-group icon");
        assert_eq!(icon.svg(), AzureResourceGroupIcon.svg());
        assert_eq!(icon.name(), AzureResourceGroupIcon.name());
    }
}

#[cfg(not(feature = "resources"))]
#[test]
fn disabled_category_does_not_resolve_resource_types() {
    assert!(
        icon_for_resource_type(&ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_RESOURCEGROUPS)
            .is_none()
    );
}

#[test]
fn unrecognized_resource_type_has_no_icon() {
    assert!(
        icon_for_resource_type(&ResourceType::Other(
            "Contoso.Example/unmappedResources".to_owned(),
        ))
        .is_none()
    );
}
