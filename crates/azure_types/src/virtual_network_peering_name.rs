use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftnetwork
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord)]
pub struct VirtualNetworkPeeringName {
    #[validate(
        length(min = 1, max = 80),
        custom(function = "validate_virtual_network_peering_name")
    )]
    inner: CompactString,
}
fn validate_virtual_network_peering_name(value: &CompactString) -> Result<(), ValidationError> {
    let len = value.len();
    if !(1..=80).contains(&len) {
        let mut err = ValidationError::new("length");
        err.add_param(std::borrow::Cow::from("length"), &len);
        err.add_param(std::borrow::Cow::from("min"), &1);
        err.add_param(std::borrow::Cow::from("max"), &80);
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

impl Slug for VirtualNetworkPeeringName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let rtn = Self { inner: name.into() };
        rtn.validate()?;
        Ok(rtn)
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        self.validate()?;
        Ok(())
    }
}

impl FromStr for VirtualNetworkPeeringName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        VirtualNetworkPeeringName::try_new(s)
    }
}
impl TryFrom<&str> for VirtualNetworkPeeringName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        VirtualNetworkPeeringName::try_new(value)
    }
}
impl std::fmt::Display for VirtualNetworkPeeringName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for VirtualNetworkPeeringName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for VirtualNetworkPeeringName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for VirtualNetworkPeeringName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for VirtualNetworkPeeringName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<VirtualNetworkPeeringName> for CompactString {
    fn from(value: VirtualNetworkPeeringName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for VirtualNetworkPeeringName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Generate a length between 1 and 80
        let len = u.int_in_range(1..=80)?;

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

        VirtualNetworkPeeringName::try_new(name).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::VirtualNetworkPeeringName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;
    use validator::Validate;

    #[test]
    fn valid_names() -> eyre::Result<()> {
        assert!(VirtualNetworkPeeringName::try_new("my-route-table").is_ok());
        assert!(VirtualNetworkPeeringName::try_new("VirtualNetworkPeering1").is_ok());
        assert!(VirtualNetworkPeeringName::try_new("route_table_underscore").is_ok());
        assert!(VirtualNetworkPeeringName::try_new("route.table.period").is_ok());
        assert!(VirtualNetworkPeeringName::try_new("a").is_ok()); // min length
        assert!(VirtualNetworkPeeringName::try_new(&"a".repeat(80)).is_ok()); // max length
        assert!(VirtualNetworkPeeringName::try_new("myVirtualNetworkPeering1_").is_ok());
        Ok(())
    }

    #[test]
    fn invalid_names() {
        assert!(VirtualNetworkPeeringName::try_new("").is_err()); // empty
        assert!(VirtualNetworkPeeringName::try_new(&"a".repeat(81)).is_err()); // too long
        assert!(VirtualNetworkPeeringName::try_new("_route").is_err()); // starts with underscore
        assert!(VirtualNetworkPeeringName::try_new(".route").is_err()); // starts with period
        assert!(VirtualNetworkPeeringName::try_new("-route").is_err()); // starts with hyphen
        assert!(VirtualNetworkPeeringName::try_new("route!").is_err()); // invalid char
        assert!(VirtualNetworkPeeringName::try_new("myRoute.").is_err()); // ends with period
        assert!(VirtualNetworkPeeringName::try_new("myRoute-").is_err()); // ends with hyphen
    }

    #[test]
    #[ignore]
    pub fn preview_failure() -> eyre::Result<()> {
        VirtualNetworkPeeringName::try_new("abc123B321")?;
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 64];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = VirtualNetworkPeeringName::arbitrary(&mut un)?;
            assert!(name.validate().is_ok());
            println!("{name}");
        }
        Ok(())
    }
}
