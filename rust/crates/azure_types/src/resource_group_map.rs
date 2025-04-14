use crate::prelude::ResourceGroup;
use crate::prelude::ResourceGroupId;
use crate::prelude::SubscriptionId;
use itertools::Itertools;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug)]
pub struct ResourceGroupMap {
    resource_groups: HashMap<ResourceGroupId, ResourceGroup>,
    resource_groups_by_subscription: HashMap<SubscriptionId, Vec<ResourceGroupId>>,
}
impl Deref for ResourceGroupMap {
    type Target = HashMap<ResourceGroupId, ResourceGroup>;

    fn deref(&self) -> &Self::Target {
        &self.resource_groups
    }
}
impl DerefMut for ResourceGroupMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource_groups
    }
}
impl From<Vec<ResourceGroup>> for ResourceGroupMap {
    fn from(resource_groups: Vec<ResourceGroup>) -> Self {
        let resource_groups: HashMap<ResourceGroupId, ResourceGroup> = resource_groups
            .into_iter()
            .map(|rg| (rg.id.clone(), rg))
            .collect();
        let resource_groups_by_subscription = resource_groups
            .values()
            .map(|rg| (rg.subscription_id.clone(), rg.id.clone()))
            .into_group_map();
        Self {
            resource_groups,
            resource_groups_by_subscription,
        }
    }
}
impl ResourceGroupMap {
    pub fn get_for_subscription(&self, subscription_id: &SubscriptionId) -> Vec<&ResourceGroup> {
        let mut rtn = vec![];
        if let Some(resource_group_ids) = self.resource_groups_by_subscription.get(subscription_id)
        {
            for resource_group_id in resource_group_ids {
                rtn.push(&self[resource_group_id]);
            }
        }
        rtn
    }
}
