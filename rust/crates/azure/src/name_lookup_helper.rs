use anyhow::Context;
use anyhow::Result;
use azure_types::prelude::Scope;
use azure_types::prelude::ScopeImpl;
use azure_types::prelude::ScopeImplKind;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use tracing::info;
use tracing::warn;

use crate::prelude::fetch_all_management_groups;
use crate::prelude::fetch_all_policy_assignments;
use crate::prelude::fetch_all_policy_definitions;
use crate::prelude::fetch_all_policy_set_definitions;
use crate::prelude::fetch_all_resource_groups;
use crate::prelude::fetch_all_subscriptions;

#[derive(Default)]
pub struct NameLookupHelper {
    cache: HashMap<ScopeImplKind, HashMap<ScopeImpl, String>>,
}
impl NameLookupHelper {
    pub async fn get_name_for_scope(&mut self, scope: &ScopeImpl) -> Result<Option<&String>> {
        let kind = scope.kind();
        if let Entry::Vacant(e) = self.cache.entry(kind) {
            let names = fetch_names_for(kind).await.context(format!(
                "Error after cache miss getting name for scope {scope}"
            ))?;
            e.insert(names);
        }
        Ok(self.cache.get(&kind).and_then(|names| names.get(scope)))
    }
}

pub async fn fetch_names_for(kind: ScopeImplKind) -> Result<HashMap<ScopeImpl, String>> {
    info!("Fetching names to populate cache for {kind:?}");
    Ok(match kind {
        ScopeImplKind::PolicyDefinition => fetch_all_policy_definitions()
            .await?
            .into_values()
            .flatten()
            .map(|x| (x.id.as_scope(), x.name))
            .collect(),
        ScopeImplKind::PolicySetDefinition => fetch_all_policy_set_definitions()
            .await?
            .into_values()
            .flatten()
            .map(|x| (x.id.as_scope(), x.name))
            .collect(),
        ScopeImplKind::PolicyAssignment => fetch_all_policy_assignments()
            .await?
            .into_values()
            .flatten()
            .map(|x| (x.id.as_scope(), x.name))
            .collect(),
        ScopeImplKind::ResourceGroup => fetch_all_resource_groups()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope(), x.name))
            .collect(),
        ScopeImplKind::ManagementGroup => fetch_all_management_groups()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope(), x.name))
            .collect(),
        ScopeImplKind::Subscription => fetch_all_subscriptions()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope(), x.name))
            .collect(),
        _ => {
            warn!(
                "Name lookup for data block generation missing impl for {kind:?} in {} {}:{}",
                file!(),
                line!(),
                column!()
            );
            Default::default()
        }
    })
}
