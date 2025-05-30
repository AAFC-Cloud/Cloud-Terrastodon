use crate::prelude::fetch_all_management_groups;
use crate::prelude::fetch_all_policy_assignments;
use crate::prelude::fetch_all_policy_definitions;
use crate::prelude::fetch_all_policy_set_definitions;
use crate::prelude::fetch_all_resource_groups;
use crate::prelude::fetch_all_storage_accounts;
use crate::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure_types::prelude::Scope;
use cloud_terrastodon_azure_types::prelude::ScopeImpl;
use cloud_terrastodon_azure_types::prelude::ScopeImplKind;
use eyre::Context;
use eyre::Result;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use tracing::debug;
use tracing::warn;

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
    debug!("Fetching names to populate cache for {kind:?}");
    let rtn: HashMap<ScopeImpl, String> = match kind {
        ScopeImplKind::PolicyDefinition => fetch_all_policy_definitions()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope_impl(), x.name))
            .collect(),
        ScopeImplKind::PolicySetDefinition => fetch_all_policy_set_definitions()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope_impl(), x.name))
            .collect(),
        ScopeImplKind::PolicyAssignment => fetch_all_policy_assignments()
            .await?
            .into_values()
            .flatten()
            .map(|x| (x.id.as_scope_impl(), x.name))
            .collect(),
        ScopeImplKind::ResourceGroup => fetch_all_resource_groups()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope_impl(), x.name.to_string()))
            .collect(),
        ScopeImplKind::ManagementGroup => fetch_all_management_groups()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope_impl(), x.name().to_owned()))
            .collect(),
        ScopeImplKind::Subscription => fetch_all_subscriptions()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope_impl(), x.name.to_string()))
            .collect(),
        ScopeImplKind::StorageAccount => fetch_all_storage_accounts()
            .await?
            .into_iter()
            .map(|x| (x.id.as_scope_impl(), x.name.to_string()))
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
    };
    Ok(rtn)
}
