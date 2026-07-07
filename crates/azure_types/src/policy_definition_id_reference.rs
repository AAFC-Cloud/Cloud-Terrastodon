use crate::PolicyDefinitionId;
use crate::PolicySetDefinitionId;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use eyre::bail;
use std::str::FromStr;

/// An ID for either a policy definition or a policy set definition
#[derive(Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum PolicyDefinitionIdReference {
    PolicyDefinitionId(PolicyDefinitionId),
    PolicySetDefinitionId(PolicySetDefinitionId),
}
crate::impl_facet_string_proxy!(PolicyDefinitionIdReference, value => value.expanded_form());

impl Scope for PolicyDefinitionIdReference {
    type Err = <Self as std::str::FromStr>::Err;
    fn expanded_form(&self) -> String {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id.expanded_form()
            }
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id.expanded_form()
            }
        }
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        Self::from_str(expanded)
    }

    fn as_scope_impl(&self) -> crate::ScopeImpl {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id.as_scope_impl()
            }
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id.as_scope_impl()
            }
        }
    }

    fn kind(&self) -> crate::ScopeImplKind {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id.kind()
            }
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id.kind()
            }
        }
    }
}

impl FromStr for PolicyDefinitionIdReference {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match (
            PolicySetDefinitionId::try_from_expanded(s),
            PolicyDefinitionId::try_from_expanded(s),
        ) {
            (Ok(a), Ok(b)) => {
                bail!(
                    "Matched both PolicyDefinitionId and PolicySetDefinitionId, this shouldnt happen. Got {} and {}",
                    a.expanded_form(),
                    b.expanded_form()
                );
            }
            (Ok(a), Err(_)) => Ok(PolicyDefinitionIdReference::PolicySetDefinitionId(a)),
            (Err(_), Ok(b)) => Ok(PolicyDefinitionIdReference::PolicyDefinitionId(b)),
            (Err(a), Err(b)) => {
                bail!(
                    "Failed to determine policy definition id kind. PolicySetDefinitionIdError={a}, PolicyDefinitionIdError={b}"
                )
            }
        }
    }
}
