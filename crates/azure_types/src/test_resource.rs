use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::strip_prefix_case_insensitive;
use eyre::Result;
use std::str::FromStr;

pub const TEST_ID_PREFIX: &str = "/CloudTerrastodon/testResources/";

/// A zero-assumption thing for usage in tests
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(json::proxy = String)]
pub struct TestResourceId {
    expanded: String,
}
crate::impl_facet_string_proxy!(TestResourceId, value => value.expanded_form());

impl TestResourceId {
    pub fn new(slug: &str) -> Self {
        Self {
            expanded: format!("{TEST_ID_PREFIX}{slug}"),
        }
    }
}

impl Scope for TestResourceId {
    type Err = <Self as std::str::FromStr>::Err;
    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        expanded.parse()
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::Test
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::TestResource(self.clone())
    }
}

impl FromStr for TestResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let slug = strip_prefix_case_insensitive(s, TEST_ID_PREFIX)?;
        Ok(TestResourceId::new(slug))
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
pub struct TestResource {
    pub id: TestResourceId,
    pub name: String,
}
impl TestResource {
    pub fn new(id_slug: &str, name: &str) -> Self {
        TestResource {
            id: TestResourceId::new(id_slug),
            name: name.to_owned(),
        }
    }
}

impl AsScope for TestResource {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &TestResource {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> Result<()> {
        let resource = TestResource::new("example", "Example");
        let json = facet_json::to_string(&resource)?;
        let reparsed = facet_json::from_str::<TestResource>(&json)?;
        assert_eq!(resource, reparsed);
        Ok(())
    }
}
