use cloud_terrastodon_azure::SubscriptionId;

/// One distinct tag name and value, with the resources that report the pair.
pub(crate) struct AzureTagInventoryEntry {
    pub name: String,
    resources: Vec<(usize, Option<SubscriptionId>)>,
}

impl AzureTagInventoryEntry {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            resources: Vec::new(),
        }
    }

    pub fn add_resource(&mut self, name: &str, index: usize, subscription: Option<SubscriptionId>) {
        // Azure providers may preserve different casing for the same tag name.
        // Use a stable spelling and count each resource once for this value.
        if name < self.name.as_str() {
            self.name = name.to_owned();
        }
        if self
            .resources
            .last()
            .is_none_or(|&(previous, _)| previous != index)
        {
            self.resources.push((index, subscription));
        }
    }

    pub fn resource_count(&self, subscription: Option<SubscriptionId>) -> usize {
        subscription.map_or(self.resources.len(), |subscription| {
            self.resources(Some(subscription)).count()
        })
    }

    pub fn resources(
        &self,
        subscription: Option<SubscriptionId>,
    ) -> impl Iterator<Item = (usize, Option<SubscriptionId>)> + '_ {
        self.resources
            .iter()
            .copied()
            .filter(move |&(_, owner)| subscription.is_none() || subscription == owner)
    }
}
