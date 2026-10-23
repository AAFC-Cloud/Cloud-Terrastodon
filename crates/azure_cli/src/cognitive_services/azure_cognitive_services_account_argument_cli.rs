use arbitrary::Arbitrary;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountResource;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountResourceId;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountResourceName;
use cloud_terrastodon_azure::Scope;
use eyre::Result;
use eyre::bail;
use std::borrow::Cow;
use std::str::FromStr;

/// Cognitive Services account can be specified as an id, a validated name, or a wildcard pattern.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum CognitiveServicesAccountArgument<'a> {
    Id(Cow<'a, AzureCognitiveServicesAccountResourceId>),
    Name(Cow<'a, AzureCognitiveServicesAccountResourceName>),
    Pattern(Cow<'a, str>),
}

impl std::fmt::Display for CognitiveServicesAccountArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CognitiveServicesAccountArgument::Id(id) => write!(f, "{}", id.expanded_form()),
            CognitiveServicesAccountArgument::Name(name) => name.fmt(f),
            CognitiveServicesAccountArgument::Pattern(pattern) => pattern.fmt(f),
        }
    }
}

impl From<AzureCognitiveServicesAccountResourceId> for CognitiveServicesAccountArgument<'_> {
    fn from(value: AzureCognitiveServicesAccountResourceId) -> Self {
        Self::Id(Cow::Owned(value))
    }
}

impl<'a> From<&'a AzureCognitiveServicesAccountResourceId>
    for CognitiveServicesAccountArgument<'a>
{
    fn from(value: &'a AzureCognitiveServicesAccountResourceId) -> Self {
        Self::Id(Cow::Borrowed(value))
    }
}

impl From<AzureCognitiveServicesAccountResourceName> for CognitiveServicesAccountArgument<'_> {
    fn from(value: AzureCognitiveServicesAccountResourceName) -> Self {
        Self::Name(Cow::Owned(value))
    }
}

impl<'a> From<&'a AzureCognitiveServicesAccountResourceName>
    for CognitiveServicesAccountArgument<'a>
{
    fn from(value: &'a AzureCognitiveServicesAccountResourceName) -> Self {
        Self::Name(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a str> for CognitiveServicesAccountArgument<'a> {
    fn from(value: &'a str) -> Self {
        Self::Pattern(Cow::Borrowed(value))
    }
}

impl CognitiveServicesAccountArgument<'_> {
    pub fn into_owned(self) -> CognitiveServicesAccountArgument<'static> {
        match self {
            CognitiveServicesAccountArgument::Id(id) => {
                CognitiveServicesAccountArgument::Id(Cow::Owned(id.into_owned()))
            }
            CognitiveServicesAccountArgument::Name(name) => {
                CognitiveServicesAccountArgument::Name(Cow::Owned(name.into_owned()))
            }
            CognitiveServicesAccountArgument::Pattern(pattern) => {
                CognitiveServicesAccountArgument::Pattern(Cow::Owned(pattern.into_owned()))
            }
        }
    }

    pub fn matches(&self, account: &AzureCognitiveServicesAccountResource) -> bool {
        match self {
            CognitiveServicesAccountArgument::Id(id) => &account.id == id.as_ref(),
            CognitiveServicesAccountArgument::Name(name) => account
                .name
                .as_str()
                .eq_ignore_ascii_case(name.as_ref().as_str()),
            CognitiveServicesAccountArgument::Pattern(pattern) => {
                wildcard_matches(pattern.as_ref(), &account.id.expanded_form())
                    || wildcard_matches(pattern.as_ref(), account.name.as_str())
            }
        }
    }
}

impl<'a> FromStr for CognitiveServicesAccountArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim();
        if value.contains('*') {
            return Ok(CognitiveServicesAccountArgument::Pattern(Cow::Owned(
                value.to_string(),
            )));
        }
        if let Ok(id) = value.parse::<AzureCognitiveServicesAccountResourceId>() {
            Ok(CognitiveServicesAccountArgument::Id(Cow::Owned(id)))
        } else if let Ok(name) = value.parse::<AzureCognitiveServicesAccountResourceName>() {
            Ok(CognitiveServicesAccountArgument::Name(Cow::Owned(name)))
        } else {
            bail!(
                "'{value}' is not a valid Cognitive Services account id, name, or wildcard pattern"
            )
        }
    }
}

fn wildcard_matches(pattern: &str, candidate: &str) -> bool {
    if !pattern.contains('*') {
        return candidate.eq_ignore_ascii_case(pattern);
    }

    let normalized_pattern = pattern.to_ascii_lowercase();
    let normalized_candidate = candidate.to_ascii_lowercase();
    if normalized_pattern == "*" {
        return true;
    }

    let starts_anchored = !normalized_pattern.starts_with('*');
    let ends_anchored = !normalized_pattern.ends_with('*');
    let parts = normalized_pattern
        .split('*')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return true;
    }

    let mut search_index = 0usize;
    let mut last_match_end = 0usize;
    for (index, part) in parts.iter().enumerate() {
        let Some(relative_start) = normalized_candidate[search_index..].find(part) else {
            return false;
        };
        let absolute_start = search_index + relative_start;
        let absolute_end = absolute_start + part.len();

        if index == 0 && starts_anchored && absolute_start != 0 {
            return false;
        }

        search_index = absolute_end;
        last_match_end = absolute_end;
    }

    !ends_anchored || last_match_end == normalized_candidate.len()
}

#[cfg(test)]
mod tests {
    use super::CognitiveServicesAccountArgument;
    use super::wildcard_matches;
    use cloud_terrastodon_azure::AzureCognitiveServicesAccountResource;

    fn sample_account() -> AzureCognitiveServicesAccountResource {
        // todo: replace with Arbitrary trait usage
        facet_json::from_str(
            r#"
            {
                "id": "/subscriptions/fe120e1b-a5bf-4e2d-8b00-66a68aabe412/resourceGroups/my-resource-group/providers/Microsoft.CognitiveServices/accounts/my-openai",
                "tenantId": "fe120e1b-a5bf-4e2d-8b00-66a68aabe412",
                "name": "my-openai",
                "kind": "OpenAI",
                "location": "canadaeast",
                "tags": {},
                "properties": {
                    "provisioningState": "Succeeded",
                    "capabilities": [],
                    "endpoints": {},
                    "associatedProjects": []
                }
            }
            "#,
        )
        .unwrap()
    }

    #[test]
    fn wildcard_matches_supports_contains_startswith_and_endswith() {
        assert!(wildcard_matches("*open*", "my-openai"));
        assert!(wildcard_matches("my-*", "my-openai"));
        assert!(wildcard_matches("*openai", "my-openai"));
        assert!(!wildcard_matches("other*", "my-openai"));
    }

    #[test]
    fn argument_matches_account_name_and_id_patterns() -> eyre::Result<()> {
        let account = sample_account();
        assert!(
            "my-openai"
                .parse::<CognitiveServicesAccountArgument<'static>>()?
                .matches(&account)
        );
        assert!(
            "*open*"
                .parse::<CognitiveServicesAccountArgument<'static>>()?
                .matches(&account)
        );
        assert!(
            "*/accounts/my-openai"
                .parse::<CognitiveServicesAccountArgument<'static>>()?
                .matches(&account)
        );
        assert!(
            !"other*"
                .parse::<CognitiveServicesAccountArgument<'static>>()?
                .matches(&account)
        );
        Ok(())
    }
}

impl<'a> TryFrom<String> for CognitiveServicesAccountArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&CognitiveServicesAccountArgument<'_>> for String {
    fn from(value: &CognitiveServicesAccountArgument<'_>) -> Self {
        value.to_string()
    }
}
