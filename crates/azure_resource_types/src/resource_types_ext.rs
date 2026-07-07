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

impl From<&ResourceType> for String {
    fn from(value: &ResourceType) -> Self {
        value.as_ref().to_string()
    }
}

impl From<String> for ResourceType {
    fn from(value: String) -> Self {
        value.parse().expect("ResourceType::from_str is infallible")
    }
}
