use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::Subscription;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use std::sync::Arc;

/// The selected IAM scope and its parents, ordered from the resource toward root.
pub(crate) struct ScopeIamAncestry {
    scopes: Vec<String>,
}

impl ScopeIamAncestry {
    pub fn new(
        scope: &ScopeImpl,
        subscriptions: &[Arc<Subscription>],
        groups: &[Arc<ManagementGroup>],
    ) -> Result<Self> {
        let mut scopes = Vec::new();
        let subscription_id = match scope {
            ScopeImpl::Subscription(id) => *id,
            ScopeImpl::ResourceGroup(group) => {
                scopes.push(group.expanded_form());
                group.subscription_id
            }
            _ => {
                bail!("IAM inventory requires a subscription or resource group scope");
            }
        };
        let subscription = subscriptions
            .iter()
            .find(|subscription| subscription.id == subscription_id)
            .ok_or_else(|| eyre!("Subscription {subscription_id} is missing from the inventory"))?;
        scopes.push(subscription_id.expanded_form());
        let ancestors = &subscription.management_group_ancestors_chain;
        scopes.extend(
            ancestors
                .iter()
                .map(|ancestor| ancestor.id().expanded_form()),
        );
        // Subscription inventory links subscriptions to their parents. The group
        // inventory can extend that chain even when the highest known parent is
        // not the root. Hidden group records do not remove known ancestry.
        if let Some(highest_known) = ancestors.last()
            && let Some(group) = groups.iter().find(|group| {
                group.tenant_id == subscription.tenant_id
                    && eq_ignore_case(group.id.name(), &highest_known.name)
            })
        {
            scopes.extend(
                group
                    .management_group_ancestors_chain
                    .iter()
                    .map(|ancestor| ancestor.id().expanded_form()),
            );
        }
        scopes.push("/".to_owned());
        Ok(Self { scopes })
    }

    pub fn contains(&self, expanded_scope: &str) -> bool {
        self.scopes
            .iter()
            .any(|scope| eq_ignore_case(scope, expanded_scope))
    }

    pub fn is_selected(&self, expanded_scope: &str) -> bool {
        self.scopes
            .first()
            .is_some_and(|scope| eq_ignore_case(scope, expanded_scope))
    }
}

fn eq_ignore_case(left: &str, right: &str) -> bool {
    if left.is_ascii() && right.is_ascii() {
        left.eq_ignore_ascii_case(right)
    } else {
        left.chars()
            .flat_map(char::to_lowercase)
            .eq(right.chars().flat_map(char::to_lowercase))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::ManagementGroupAncestorsChain;
    use cloud_terrastodon_azure::ManagementGroupAncestorsChainEntry;
    use cloud_terrastodon_azure::ManagementGroupId;
    use facet_reflect::Partial;

    fn chain(names: &[&str]) -> Result<ManagementGroupAncestorsChain> {
        let entries = names
            .iter()
            .map(|name| ManagementGroupAncestorsChainEntry {
                name: (*name).to_owned(),
                display_name: (*name).to_owned(),
            })
            .collect::<Vec<_>>();
        Ok(Partial::alloc::<ManagementGroupAncestorsChain>()?
            .begin_inner()?
            .set(entries)?
            .end()?
            .build()?
            .materialize()?)
    }

    #[test]
    fn resource_group_membership_uses_whole_ids_and_management_group_ancestry() -> Result<()> {
        const SUBSCRIPTION: &str = "/subscriptions/00000000-0000-0000-0000-000000000001";
        let tenant_id = "00000000-0000-0000-0000-000000000002".parse()?;
        let subscription = Arc::new(Subscription {
            id: SUBSCRIPTION.parse()?,
            name: "Example subscription".parse()?,
            tenant_id,
            management_group_ancestors_chain: chain(&["production"])?,
            tags: Default::default(),
        });
        let group = Arc::new(ManagementGroup {
            id: ManagementGroupId::from_name("production"),
            display_name: "Production".to_owned(),
            tenant_id,
            management_group_ancestors_chain: chain(&["root"])?,
        });
        let scope = format!("{SUBSCRIPTION}/resourceGroups/prod").parse()?;
        let inventory = [subscription];
        let ancestry = ScopeIamAncestry::new(&scope, &inventory, &[group])?;
        assert!(ancestry.is_selected(&scope.expanded_form()));
        assert!(!ancestry.is_selected(SUBSCRIPTION));
        for expected in [
            format!("{SUBSCRIPTION}/RESOURCEGROUPS/PROD"),
            SUBSCRIPTION.to_owned(),
            "/providers/microsoft.management/managementgroups/Production".to_owned(),
            ManagementGroupId::from_name("root").expanded_form(),
            "/".to_owned(),
        ] {
            assert!(ancestry.contains(&expected), "{expected}");
        }
        for unrelated in [
            format!("{SUBSCRIPTION}/resourceGroups/prod-east"),
            format!("{SUBSCRIPTION}/resourceGroups/prod/providers/Example/example/child"),
            ManagementGroupId::from_name("other").expanded_form(),
        ] {
            assert!(!ancestry.contains(&unrelated), "{unrelated}");
        }

        let subscription_scope = SUBSCRIPTION.parse()?;
        let subscription = ScopeIamAncestry::new(&subscription_scope, &inventory, &[])?;
        assert!(subscription.contains(&ManagementGroupId::from_name("production").expanded_form()));
        assert!(!subscription.contains(&scope.expanded_form()));
        assert!(ScopeIamAncestry::new(&scope, &[], &[]).is_err());

        let unicode_scope = format!("{SUBSCRIPTION}/resourceGroups/Über").parse()?;
        let ancestry = ScopeIamAncestry::new(&unicode_scope, &inventory, &[])?;
        assert!(ancestry.contains(&format!("{SUBSCRIPTION}/resourceGroups/über")));
        Ok(())
    }
}
