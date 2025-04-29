use crate::resource_types::ResourceType;

impl ResourceType {
    pub fn is_resource_group(&self) -> bool {
        matches!(
            self,
            ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_RESOURCEGROUPS
                | ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS
        )
    }
    pub fn is_subscription(&self) -> bool {
        self == &ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
