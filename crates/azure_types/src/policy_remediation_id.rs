use crate::PolicyRemediationName;
use crate::PolicyRemediationScope;
use arbitrary::Arbitrary;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// ARM identity of a remediation at a supported management or resource group.
///
/// See the [2024-10-01 remediation schema](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediation).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct PolicyRemediationId {
    pub scope: PolicyRemediationScope,
    pub name: PolicyRemediationName,
}

impl Display for PolicyRemediationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/providers/Microsoft.PolicyInsights/remediations/{}",
            self.scope, self.name
        )
    }
}

impl FromStr for PolicyRemediationId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> eyre::Result<Self> {
        let (parent, name) = value
            .rsplit_once('/')
            .ok_or_else(|| eyre!("a remediation ID must contain an ARM scope and name"))?;
        let suffix = "/providers/Microsoft.PolicyInsights/remediations";
        let split = parent
            .len()
            .checked_sub(suffix.len())
            .filter(|index| parent.is_char_boundary(*index))
            .ok_or_else(|| {
                eyre!("a remediation ID must contain the PolicyInsights remediation route")
            })?;
        ensure!(
            parent[split..].eq_ignore_ascii_case(suffix),
            "expected the PolicyInsights remediation route"
        );
        Ok(Self {
            scope: parent[..split]
                .parse()
                .wrap_err("invalid remediation scope")?,
            name: name.parse().wrap_err("invalid remediation name")?,
        })
    }
}

crate::impl_facet_string_proxy!(PolicyRemediationId, value => value.to_string());

cloud_terrastodon_registry::register_thing!(PolicyRemediationId);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationId);

#[cfg(test)]
mod tests {
    use super::PolicyRemediationId;
    use crate::PolicyRemediationScope;

    #[test]
    fn preserves_scoped_identity_and_rejects_incomplete_or_nested_routes() -> eyre::Result<()> {
        let resource_group: PolicyRemediationId = "/subscriptions/00000000-0000-0000-0000-000000000001/resourceGroups/example/providers/microsoft.policyinsights/remediations/example-task".parse()?;
        let management_group: PolicyRemediationId = "/providers/Microsoft.Management/managementGroups/example/providers/Microsoft.PolicyInsights/remediations/example-task".parse()?;
        assert!(matches!(
            resource_group.scope,
            PolicyRemediationScope::ResourceGroup(_)
        ));
        assert!(matches!(
            management_group.scope,
            PolicyRemediationScope::ManagementGroup(_)
        ));
        assert_ne!(resource_group, management_group);
        assert_eq!(
            resource_group.to_string().parse::<PolicyRemediationId>()?,
            resource_group
        );
        assert_eq!(
            management_group
                .to_string()
                .parse::<PolicyRemediationId>()?,
            management_group
        );
        for suffix in ["/child", "/.."] {
            assert!(
                format!("{management_group}{suffix}")
                    .parse::<PolicyRemediationId>()
                    .is_err()
            );
        }
        assert!("/subscriptions/00000000-0000-0000-0000-000000000001/providers/Microsoft.PolicyInsights/remediations/task".parse::<PolicyRemediationId>().is_err());
        assert!("/providers/Microsoft.Management/managementGroups/example/providers/Microsoft.PolicyInsights/remediations/".parse::<PolicyRemediationId>().is_err());
        Ok(())
    }
}
