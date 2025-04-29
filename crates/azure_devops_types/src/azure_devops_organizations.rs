use std::ops::Deref;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsOrganizationName(String);
impl Deref for AzureDevOpsOrganizationName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Display for AzureDevOpsOrganizationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AzureDevOpsOrganizationName {
    pub fn new(name: String) -> AzureDevOpsOrganizationName {
        AzureDevOpsOrganizationName(name)
    }
}
impl FromStr for AzureDevOpsOrganizationName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureDevOpsOrganizationName::new(s.to_string()))
    }
}
impl AsRef<str> for AzureDevOpsOrganizationName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
