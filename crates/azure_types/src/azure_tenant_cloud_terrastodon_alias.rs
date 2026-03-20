use compact_str::CompactString;
use eyre::WrapErr;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// A Cloud Terrastodon-specific alias for a tracked Azure tenant.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AzureTenantAlias {
    inner: CompactString,
}

impl AzureTenantAlias {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        validate_alias(&value)?;
        Ok(Self {
            inner: value.to_ascii_lowercase(),
        })
    }
}

fn validate_alias(value: &str) -> eyre::Result<()> {
    validate_alias_inner(value).wrap_err_with(|| format!("Invalid Azure tenant alias: {value}"))
}

fn validate_alias_inner(value: &str) -> eyre::Result<()> {
    if value.is_empty() {
        bail!("Alias cannot be empty");
    }
    if value.len() > 64 {
        bail!("Alias must be 64 characters or less, got {}", value.len());
    }

    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        bail!("Alias cannot be empty");
    };
    if !first.is_ascii_alphanumeric() {
        bail!("Alias must start with an ASCII letter or digit");
    }

    let mut last = first;
    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')) {
            bail!(
                "Alias contains invalid character '{}'; only ASCII letters, digits, '-', '_', and '.' are allowed",
                ch
            );
        }
        last = ch;
    }

    if !last.is_ascii_alphanumeric() {
        bail!("Alias must end with an ASCII letter or digit");
    }

    Ok(())
}

impl FromStr for AzureTenantAlias {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl TryFrom<&str> for AzureTenantAlias {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureTenantAlias {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<&String> for AzureTenantAlias {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl std::fmt::Display for AzureTenantAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl Deref for AzureTenantAlias {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl serde::Serialize for AzureTenantAlias {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzureTenantAlias {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:#}")))
    }
}

impl From<AzureTenantAlias> for CompactString {
    fn from(value: AzureTenantAlias) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod tests {
    use super::AzureTenantAlias;

    #[test]
    fn it_normalizes_to_lowercase() -> eyre::Result<()> {
        let alias = AzureTenantAlias::try_new("Prod.WEST")?;
        assert_eq!(alias.to_string(), "prod.west");
        Ok(())
    }

    #[test]
    fn it_rejects_bad_aliases() {
        assert!(AzureTenantAlias::try_new("").is_err());
        assert!(AzureTenantAlias::try_new("-prod").is_err());
        assert!(AzureTenantAlias::try_new("prod-").is_err());
        assert!(AzureTenantAlias::try_new("prod west").is_err());
    }
}
