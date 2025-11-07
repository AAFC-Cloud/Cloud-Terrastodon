use crate::prelude::ManagementGroupAncestorsChain;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionName;
use crate::prelude::TenantId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::HclProviderBlock;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub name: SubscriptionName,
    pub tenant_id: TenantId,
    pub management_group_ancestors_chain: ManagementGroupAncestorsChain,
}

impl AsScope for Subscription {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &Subscription {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl Hash for Subscription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Subscription {}
impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
impl Subscription {
    pub fn into_provider_block(&self) -> HclProviderBlock {
        HclProviderBlock::AzureRM {
            alias: Some(self.name.sanitize()),
            subscription_id: Some(self.id.short_form().to_owned()),
        }
    }
}
impl From<Subscription> for HclProviderBlock {
    fn from(value: Subscription) -> Self {
        value.into_provider_block()
    }
}
