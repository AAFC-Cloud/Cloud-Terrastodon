#[cfg(test)]
mod tests {
    use crate::prelude::ResourceType;

    #[test]
    fn resource_group() {
        for x in [
            "microsoft.resources/subscriptions/resourcegroups",
            "Microsoft.Resources/Subscriptions/ResourceGroups",
            "MICROSOFT.RESOURCES/SUBSCRIPTIONS/RESOURCEGROUPS",
        ] {
            let y: ResourceType = x.parse().unwrap();
            assert_eq!(
                y,
                ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS
            );
        }
    }

    #[test]
    fn serialization() {
        let x = ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS;
        let y = serde_json::to_string(&x).unwrap();
        assert_eq!(
            y.to_lowercase(),
            "\"Microsoft.Resources/Subscriptions/ResourceGroups\"".to_lowercase()
        );
        let z = serde_json::from_str::<ResourceType>(&y).unwrap();
        assert_eq!(x, z);
    }

    #[test]
    fn other() {
        let x = "SOME VALUE";
        let y = x.parse::<ResourceType>().unwrap();
        assert_eq!(y, ResourceType::Other(x.to_owned()));
    }
}
