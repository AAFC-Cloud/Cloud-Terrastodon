#[cfg(test)]
mod tests {
    use crate::prelude::ResourceType;

    use super::*;

    #[test]
    fn resource_group() {
        for x in [
            "microsoft.resources/subscriptions/resourcegroups",
            "Microsoft.Resources/Subscriptions/ResourceGroups",
            "MICROSOFT.RESOURCES/SUBSCRIPTIONS/RESOURCEGROUPS",
        ] {
            let y: ResourceType = x.parse().unwrap();
            assert_eq!(y, ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS);
        }
    }
}
