use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::WrapErr;
use eyre::bail;
use facet::Facet;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Facet)]
#[facet(transparent)]
pub struct GiteaTenantAlias(CompactString);

impl GiteaTenantAlias {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        validate_tenant_alias(&value)?;
        Ok(Self(value.to_ascii_lowercase()))
    }
}

fn validate_tenant_alias(value: &str) -> eyre::Result<()> {
    validate_tenant_alias_inner(value)
        .wrap_err_with(|| format!("Invalid Gitea tenant alias: {value}"))
}

fn validate_tenant_alias_inner(value: &str) -> eyre::Result<()> {
    if value.is_empty() {
        bail!("Alias cannot be empty");
    }
    if value.len() > 64 {
        bail!("Alias must be 64 characters or less");
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

impl<'a> Arbitrary<'a> for GiteaTenantAlias {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(1..=64)?;
        let mut alias = String::with_capacity(len);
        alias.push(gitea_alias_edge_char(u)?);
        while alias.len() + 1 < len {
            alias.push(gitea_alias_middle_char(u)?);
        }
        if len > 1 {
            alias.push(gitea_alias_edge_char(u)?);
        }
        GiteaTenantAlias::try_new(alias).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

fn gitea_alias_edge_char<'a>(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<char> {
    let idx = u.int_in_range(0..=35)?;
    Ok(match idx {
        0..=25 => (b'a' + idx as u8) as char,
        _ => (b'0' + (idx - 26) as u8) as char,
    })
}

fn gitea_alias_middle_char<'a>(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<char> {
    let idx = u.int_in_range(0..=38)?;
    Ok(match idx {
        0..=25 => (b'a' + idx as u8) as char,
        26..=35 => (b'0' + (idx - 26) as u8) as char,
        36 => '-',
        37 => '_',
        _ => '.',
    })
}
impl Display for GiteaTenantAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for GiteaTenantAlias {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for GiteaTenantAlias {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for GiteaTenantAlias {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}
