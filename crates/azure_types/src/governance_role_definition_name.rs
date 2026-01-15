use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::WrapErr;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// TODO: find documentation on the real limit, I made up this value.
pub const GOVERNANCE_ROLE_DEFINITION_NAME_MAX_LENGTH: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GovernanceRoleDefinitionName {
    inner: CompactString,
}

fn validate_governance_role_definition_name(value: &str) -> eyre::Result<()> {
    validate_governance_role_definition_name_inner(value)
        .wrap_err_with(|| format!("Invalid governance_role_definition name: {}", value))
}

fn validate_governance_role_definition_name_inner(value: &str) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if char_count == 0 {
        bail!("Governance role definition name cannot be empty");
    }
    if char_count > GOVERNANCE_ROLE_DEFINITION_NAME_MAX_LENGTH {
        bail!(
            "Governance role definition name must be {GOVERNANCE_ROLE_DEFINITION_NAME_MAX_LENGTH} characters or less, got {char_count}",
        );
    }
    Ok(())
}

impl GovernanceRoleDefinitionName {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = value.into();
        validate_governance_role_definition_name(&inner)?;
        Ok(Self { inner })
    }
}

impl FromStr for GovernanceRoleDefinitionName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GovernanceRoleDefinitionName::try_new(s)
    }
}
impl TryFrom<&str> for GovernanceRoleDefinitionName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        GovernanceRoleDefinitionName::try_new(value)
    }
}
impl TryFrom<String> for GovernanceRoleDefinitionName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
impl TryFrom<&String> for GovernanceRoleDefinitionName {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl std::fmt::Display for GovernanceRoleDefinitionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for GovernanceRoleDefinitionName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for GovernanceRoleDefinitionName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for GovernanceRoleDefinitionName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for GovernanceRoleDefinitionName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<GovernanceRoleDefinitionName> for CompactString {
    fn from(value: GovernanceRoleDefinitionName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for GovernanceRoleDefinitionName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Length: 1..=50 Unicode scalar values
        let len = u.int_in_range(1..=GOVERNANCE_ROLE_DEFINITION_NAME_MAX_LENGTH)?;

        // Build string of `len` random Unicode scalar values
        let mut scalars = Vec::with_capacity(len);
        for _ in 0..len {
            // Restrict to legal scalar values, avoid surrogate code points (U+D800..=U+DFFF)
            let mut attempts = 0;
            let c = loop {
                let code = u.arbitrary::<u32>()? % 0x110000;
                // skip UTF-16 surrogates
                if (0xD800..=0xDFFF).contains(&code) || code > 0x10FFFF {
                    attempts += 1;
                    if attempts > 10 {
                        break ' ';
                    }
                    continue;
                }
                if let Some(c) = std::char::from_u32(code) {
                    break c;
                }
                attempts += 1;
                if attempts > 10 {
                    break ' ';
                }
            };
            scalars.push(c);
        }

        let s: String = scalars.into_iter().collect();
        // Wrap and validate, your constructor may have additional checks
        GovernanceRoleDefinitionName::try_new(CompactString::from(s))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}
#[cfg(test)]
mod test {
    use crate::prelude::GOVERNANCE_ROLE_DEFINITION_NAME_MAX_LENGTH;
    use crate::prelude::GovernanceRoleDefinitionName;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 128];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = GovernanceRoleDefinitionName::arbitrary(&mut un)?;
            // Name is already validated during construction
            assert!(
                name.inner.chars().count() >= 1
                    && name.inner.chars().count() <= GOVERNANCE_ROLE_DEFINITION_NAME_MAX_LENGTH
            );
        }
        Ok(())
    }
}
