use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

// Rules from: https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftnetwork
// Virtual networks:
// - Name: 2-64 characters.
// - Can contain letters, numbers, underscores, periods, and hyphens.
// - Must start with a letter or number.
// - Must end with a letter, number, or underscore.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord)]
pub struct VirtualNetworkName {
    #[validate(custom(function = "validate_virtual_network_name_contents"))]
    inner: CompactString,
}

impl Slug for VirtualNetworkName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let s = Self { inner: name.into() };
        s.validate_slug()?;
        Ok(s)
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        self.validate()?;
        Ok(())
    }
}

fn validate_virtual_network_name_contents(value: &CompactString) -> Result<(), ValidationError> {
    let len = value.len();
    if !(2..=64).contains(&len) {
        let mut err = ValidationError::new("length");
        err.add_param(std::borrow::Cow::from("length"), &len);
        err.add_param(std::borrow::Cow::from("min"), &2);
        err.add_param(std::borrow::Cow::from("max"), &64);
        return Err(err);
    }

    let chars: Vec<char> = value.chars().collect();

    // Must start with alphanumeric
    let first_char = chars.first().ok_or_else(|| {
        let mut err = ValidationError::new("first_char");
        err.message = Some(std::borrow::Cow::from("Name cannot be empty"));
        err
    })?;

    if !first_char.is_alphanumeric() {
        let mut err = ValidationError::new("first_char_alphanumeric");
        err.message = Some(std::borrow::Cow::from("Name must start with alphanumeric."));
        return Err(err);
    }

    // Must end with alphanumeric or underscore
    let last_char = chars.last().ok_or_else(|| {
        let mut err = ValidationError::new("last_char");
        err.message = Some(std::borrow::Cow::from("Name cannot be empty"));
        err
    })?;

    if !(last_char.is_alphanumeric() || *last_char == '_') {
        let mut err = ValidationError::new("last_char_invalid");
        err.message = Some(std::borrow::Cow::from(
            "Name must end with alphanumeric or underscore.",
        ));
        return Err(err);
    }

    // All characters must be alphanumeric, underscore, period, or hyphen
    for char_code in &chars {
        if !(char_code.is_alphanumeric()
            || *char_code == '_'
            || *char_code == '.'
            || *char_code == '-')
        {
            let mut err = ValidationError::new("invalid_char");
            err.message = Some(std::borrow::Cow::from(
                "Name can only contain alphanumerics, underscores, periods, and hyphens.",
            ));
            return Err(err);
        }
    }

    Ok(())
}

impl FromStr for VirtualNetworkName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl TryFrom<&str> for VirtualNetworkName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl std::fmt::Display for VirtualNetworkName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl serde::Serialize for VirtualNetworkName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for VirtualNetworkName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = CompactString::deserialize(deserializer)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

impl Deref for VirtualNetworkName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFrom<CompactString> for VirtualNetworkName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<VirtualNetworkName> for CompactString {
    fn from(value: VirtualNetworkName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for VirtualNetworkName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Generate a length between 2 and 64
        let len = u.int_in_range(2..=64)?;

        // Start with an alphanumeric character
        let first_char = if u.ratio(1, 2)? {
            // Generate a letter
            if u.ratio(1, 2)? {
                u.int_in_range(b'a'..=b'z')? as char
            } else {
                u.int_in_range(b'A'..=b'Z')? as char
            }
        } else {
            // Generate a digit
            u.int_in_range(b'0'..=b'9')? as char
        };

        let mut name = String::with_capacity(len);
        name.push(first_char);

        // Generate middle characters (if any)
        for _ in 1..len.saturating_sub(1) {
            let char = match u.int_in_range(0..=5)? {
                0..=1 => u.int_in_range(b'a'..=b'z')? as char, // lowercase letter
                2..=3 => u.int_in_range(b'A'..=b'Z')? as char, // uppercase letter
                4 => u.int_in_range(b'0'..=b'9')? as char,     // digit
                5 => match u.int_in_range(0..=2)? {
                    0 => '_',
                    1 => '.',
                    _ => '-',
                },
                _ => unreachable!(),
            };
            name.push(char);
        }

        // Ensure we end with alphanumeric or underscore (if length > 1)
        if len > 1 {
            let last_char = if u.ratio(3, 4)? {
                // Generate alphanumeric (75% chance)
                if u.ratio(1, 3)? {
                    // Generate a digit
                    u.int_in_range(b'0'..=b'9')? as char
                } else if u.ratio(1, 2)? {
                    // Generate lowercase letter
                    u.int_in_range(b'a'..=b'z')? as char
                } else {
                    // Generate uppercase letter
                    u.int_in_range(b'A'..=b'Z')? as char
                }
            } else {
                // Generate underscore (25% chance)
                '_'
            };
            name.push(last_char);
        }

        VirtualNetworkName::try_new(name).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_names() -> eyre::Result<()> {
        assert!(VirtualNetworkName::try_new("my-vnet").is_ok());
        assert!(VirtualNetworkName::try_new("VNet1").is_ok());
        assert!(VirtualNetworkName::try_new("vnet_underscore").is_ok());
        assert!(VirtualNetworkName::try_new("vnet.period").is_ok());
        assert!(VirtualNetworkName::try_new("a2").is_ok()); // min length
        assert!(VirtualNetworkName::try_new(&"a".repeat(64)).is_ok()); // max length
        assert!(VirtualNetworkName::try_new("myVNet1_").is_ok());
        Ok(())
    }

    #[test]
    fn invalid_names() {
        assert!(VirtualNetworkName::try_new("a").is_err()); // too short
        assert!(VirtualNetworkName::try_new(&"a".repeat(65)).is_err()); // too long
        assert!(VirtualNetworkName::try_new("_vnet").is_err()); // starts with underscore
        assert!(VirtualNetworkName::try_new(".vnet").is_err()); // starts with period
        assert!(VirtualNetworkName::try_new("-vnet").is_err()); // starts with hyphen
        assert!(VirtualNetworkName::try_new("vnet!").is_err()); // invalid char
        assert!(VirtualNetworkName::try_new("myVNet.").is_err()); // ends with period
        assert!(VirtualNetworkName::try_new("myVNet-").is_err()); // ends with hyphen
    }

    #[test]
    fn validate_slug_method() -> eyre::Result<()> {
        let name = VirtualNetworkName {
            inner: "my-vnet".into(),
        };
        name.validate_slug()?;
        Ok(())
    }

    #[test]
    fn validate_slug_method_invalid() {
        let name = VirtualNetworkName { inner: "a".into() }; // too short
        assert!(name.validate_slug().is_err());
    }
}
