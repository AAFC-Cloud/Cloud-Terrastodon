use crate::scopes::HasScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

pub const TEST_ID_PREFIX: &str = "/tests/";

/// A zero-assumption thing for usage in tests
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TestResourceId {
    expanded: String,
}

impl TestResourceId {
    fn new(slug: &str) -> Self {
        Self {
            expanded: format!("{}{}", TEST_ID_PREFIX, slug),
        }
    }
}

impl Scope for TestResourceId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn short_form(&self) -> &str {
        self.expanded_form().strip_prefix(TEST_ID_PREFIX).unwrap()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        expanded.parse()
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::Test
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::TestResource(self.clone())
    }
}

impl FromStr for TestResourceId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let slug = s.strip_prefix(TEST_ID_PREFIX).unwrap_or(s);
        Ok(TestResourceId::new(slug))
    }
}
impl Serialize for TestResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for TestResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded.parse().map_err(D::Error::custom)?;
        Ok(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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

impl HasScope for TestResource {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &TestResource {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
