use crate::AzureResourceGroup;
use crate::AzureResourceGroupId;
use crate::SubscriptionId;
use itertools::Itertools;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

#[derive(Debug)]
pub struct AzureResourceGroupMap {
    resource_groups: HashMap<AzureResourceGroupId, Arc<AzureResourceGroup>>,
    resource_groups_by_subscription: HashMap<SubscriptionId, Vec<AzureResourceGroupId>>,
}
impl Deref for AzureResourceGroupMap {
    type Target = HashMap<AzureResourceGroupId, Arc<AzureResourceGroup>>;

    fn deref(&self) -> &Self::Target {
        &self.resource_groups
    }
}
impl DerefMut for AzureResourceGroupMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource_groups
    }
}
impl From<Vec<Arc<AzureResourceGroup>>> for AzureResourceGroupMap {
    fn from(resource_groups: Vec<Arc<AzureResourceGroup>>) -> Self {
        let resource_groups: HashMap<AzureResourceGroupId, Arc<AzureResourceGroup>> =
            resource_groups
                .into_iter()
                .map(|rg| (rg.id.clone(), rg))
                .collect();
        let resource_groups_by_subscription = resource_groups
            .values()
            .map(|rg| (rg.id.subscription_id, rg.id.clone()))
            .into_group_map();
        Self {
            resource_groups,
            resource_groups_by_subscription,
        }
    }
}
impl AzureResourceGroupMap {
    pub fn get_for_subscription(
        &self,
        subscription_id: &SubscriptionId,
    ) -> Vec<&AzureResourceGroup> {
        let mut rtn = vec![];
        if let Some(resource_group_ids) = self.resource_groups_by_subscription.get(subscription_id)
        {
            for resource_group_id in resource_group_ids {
                rtn.push(self[resource_group_id].as_ref());
            }
        }
        rtn
    }
}
