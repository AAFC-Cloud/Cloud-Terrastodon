use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::SUBSCRIPTION_ID_PREFIX;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_azure::strip_prefix_case_insensitive;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use std::collections::HashSet;
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
        let expanded = scope.expanded_form();
        ensure!(
            expanded.starts_with('/'),
            "IAM requires an absolute ARM scope"
        );
        let mut scopes = Vec::new();
        let mut current = expanded.as_str();
        while !current.is_empty() && current != "/" {
            let (path, name) = current
                .rsplit_once('/')
                .ok_or_else(|| eyre!("ARM scope is missing a resource name"))?;
            let (parent, collection) = path
                .rsplit_once('/')
                .ok_or_else(|| eyre!("ARM scope is missing a resource collection"))?;
            ensure!(
                !["", ".", ".."].contains(&name) && !["", ".", ".."].contains(&collection),
                "ARM scope contains an invalid path segment"
            );
            // Provider namespace markers divide resource ID segments without
            // creating their own parent scope. The type/name pairs on both
            // sides can still carry assignments, including extension resources.
            if !collection.eq_ignore_ascii_case("providers") {
                scopes.push(current.to_owned());
            }
            current = parent;
        }
        ensure!(
            expanded == "/" || scopes.first() == Some(&expanded),
            "IAM scope must identify a resource or resource container"
        );
        let subscription_id = scopes.iter().find_map(|scope| {
            strip_prefix_case_insensitive(scope, SUBSCRIPTION_ID_PREFIX)
                .ok()?
                .parse::<SubscriptionId>()
                .ok()
        });
        if let Some(subscription_id) = subscription_id {
            let subscription = subscriptions
                .iter()
                .find(|subscription| subscription.id == subscription_id)
                .ok_or_else(|| {
                    eyre!("Subscription {subscription_id} is missing from the inventory")
                })?;
            let ancestors = &subscription.management_group_ancestors_chain;
            scopes.extend(
                ancestors
                    .iter()
                    .map(|ancestor| ancestor.id().expanded_form()),
            );
            // A visible group might extend the subscription's known chain.
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
        } else if let Some(group_id) = scopes
            .iter()
            .find_map(|scope| ManagementGroupId::try_from_expanded(scope).ok())
            && let Some(group) = groups
                .iter()
                .find(|group| eq_ignore_case(group.id.name(), group_id.name()))
        {
            scopes.extend(
                group
                    .management_group_ancestors_chain
                    .iter()
                    .map(|ancestor| ancestor.id().expanded_form()),
            );
        }
        scopes.push("/".to_owned());
        let mut seen = HashSet::with_capacity(scopes.len());
        scopes.retain(|scope| seen.insert(scope_key(scope)));
        Ok(Self { scopes })
    }

    pub fn scopes(&self) -> impl Iterator<Item = &str> {
        self.scopes.iter().map(String::as_str)
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

pub(crate) fn scope_key(scope: &str) -> String {
    if scope.is_ascii() {
        scope.to_ascii_lowercase()
    } else {
        scope.chars().flat_map(char::to_lowercase).collect()
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
        let groups = [group];
        let ancestry = ScopeIamAncestry::new(&scope, &inventory, &groups)?;
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

        let management_group = ScopeImpl::ManagementGroup(groups[0].id.clone());
        let ancestry = ScopeIamAncestry::new(&management_group, &[], &groups)?;
        assert!(ancestry.is_selected(&management_group.expanded_form()));
        assert!(ancestry.contains(&ManagementGroupId::from_name("root").expanded_form()));
        assert!(!ancestry.contains(SUBSCRIPTION));
        Ok(())
    }

    #[test]
    fn nested_resources_inherit_from_type_name_pairs_across_provider_namespaces() -> Result<()> {
        const SUBSCRIPTION: &str = "/subscriptions/00000000-0000-0000-0000-000000000001";
        let subscription = Arc::new(Subscription {
            id: SUBSCRIPTION.parse()?,
            name: "Example subscription".parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000002".parse()?,
            management_group_ancestors_chain: chain(&["production"])?,
            tags: Default::default(),
        });
        let group = format!("{SUBSCRIPTION}/resourceGroups/prod");
        let gallery = format!("{group}/providers/Microsoft.Compute/galleries/assets");
        let image = format!("{gallery}/images/demo");
        let id = format!("{image}/providers/Example.Widget/extensions/one");
        let scope = ScopeImpl::Unknown(id.clone().into());
        let ancestry = ScopeIamAncestry::new(&scope, &[subscription], &[])?;
        assert!(ancestry.is_selected(&id));
        for parent in [&image, &gallery, &group] {
            assert!(ancestry.contains(parent), "{parent}");
            assert!(!ancestry.is_selected(parent), "{parent}");
        }
        assert!(ancestry.contains(SUBSCRIPTION));
        assert!(ancestry.contains(&ManagementGroupId::from_name("production").expanded_form()));
        assert!(!ancestry.contains(&format!("{image}/providers/Example.Widget")));
        assert!(!ancestry.contains(&format!("{image}/providers/Example.Widget/extensions/two")));
        Ok(())
    }
}
