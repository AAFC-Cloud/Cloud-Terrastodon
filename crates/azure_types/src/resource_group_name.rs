use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::WrapErr;
use eyre::bail;
use facet::Facet;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;
use unicode_categories::UnicodeCategories;

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftresources
///
/// Underscores, hyphens, periods, parentheses, and letters or digits as defined by the Char.IsLetterOrDigit function
///
/// Valid characters are members of the following categories in UnicodeCategory:
/// UppercaseLetter,
/// LowercaseLetter,
/// TitlecaseLetter,
/// ModifierLetter,
/// OtherLetter,
/// DecimalDigitNumber.
///
/// Can't end with period.
///
/// See also: https://github.com/Azure/azure-rest-api-specs/blob/6c548b0bd279f5e233661b1c81fb5b61b19965cd/specification/storage/resource-manager/Microsoft.Storage/stable/2025-01-01/storage.json#L5841-L5852
#[derive(Debug, Clone, Eq, PartialOrd, Ord, Facet)]
#[facet(transparent)]
pub struct ResourceGroupName(CompactString);

impl PartialEq for ResourceGroupName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
impl Hash for ResourceGroupName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}
impl Slug for ResourceGroupName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_resource_group_name(&inner)?;
        Ok(Self(inner))
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_resource_group_name(&self.0)?;
        Ok(())
    }
}

impl FromStr for ResourceGroupName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ResourceGroupName::try_new(s)
    }
}
impl TryFrom<&str> for ResourceGroupName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ResourceGroupName::try_new(value)
    }
}
impl TryFrom<String> for ResourceGroupName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
impl TryFrom<&String> for ResourceGroupName {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

fn is_valid_rg_char(ch: char) -> bool {
    matches!(ch, '_' | '-' | '.' | '(' | ')')
        || ch.is_uppercase()
        || ch.is_letter_uppercase()
        || ch.is_letter_lowercase()
        || ch.is_letter_modifier()
        || ch.is_letter_other()
        || ch.is_number_decimal_digit()
}
fn validate_resource_group_name_inner(value: &str) -> eyre::Result<()> {
    if value.is_empty() {
        bail!("Resource group name cannot be empty");
    }
    // Use chars().count() instead of len() because len() counts bytes, not Unicode characters
    // Azure limits are specified in characters, not bytes
    let char_count = value.chars().count();
    if char_count > 90 {
        bail!(
            "Resource group name must be 90 characters or less, got {}",
            char_count
        );
    }

    let mut last_char: Option<char> = None;

    for (i, ch) in value.chars().enumerate() {
        // Allow specific punctuation
        if matches!(ch, '_' | '-' | '.' | '(' | ')') {
            last_char = Some(ch);
            continue;
        }

        // Allow all valid unicode letters/digits
        if is_valid_rg_char(ch) {
            last_char = Some(ch);
            continue;
        }

        // Invalid character
        bail!(
            "Resource group name contains invalid character '{}' at position {}",
            ch,
            i
        );
    }

    // Cannot end with period
    if let Some('.') = last_char {
        bail!("Resource group name cannot end with a period");
    }

    Ok(())
}

fn validate_resource_group_name(value: &str) -> eyre::Result<()> {
    validate_resource_group_name_inner(value)
        .wrap_err_with(|| format!("Invalid resource group name: {}", value))
        .wrap_err("https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftresources")
}

impl std::fmt::Display for ResourceGroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl Deref for ResourceGroupName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TryFrom<CompactString> for ResourceGroupName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<ResourceGroupName> for CompactString {
    fn from(value: ResourceGroupName) -> Self {
        value.0
    }
}

impl<'a> Arbitrary<'a> for ResourceGroupName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Length 1..=90 (Azure RG name max length)
        let len = u.int_in_range(1..=90)?;

        // Helper: generate *one* valid random char
        fn rand_valid_char(u: &mut Unstructured<'_>) -> arbitrary::Result<char> {
            // 90%: pick from common ASCII-valid set, 10%: try a random Unicode letter/digit
            if u.arbitrary::<u8>()? < 230 {
                const COMMON: &[char] = &['_', '-', '.', '(', ')'];
                // Letters and digits, a-zA-Z0-9
                let letters = ('a'..='z').chain('A'..='Z');
                let digits = '0'..='9';
                let mut options: Vec<char> = letters.chain(digits).collect();
                options.extend(COMMON.iter().copied());
                Ok(*u.choose(&options)?)
            } else {
                // Try a random Unicode codepoint in BMP, filter by allowed
                for _ in 0..16 {
                    let c: char = core::char::from_u32(u.int_in_range(0..=0xD7FF)?).unwrap_or('a');
                    if is_valid_rg_char(c) {
                        return Ok(c);
                    }
                }
                // Fallback: always '_'
                Ok('_')
            }
        }

        // Generate candidate
        let mut chars = Vec::with_capacity(len as usize);
        for _ in 0..len {
            chars.push(rand_valid_char(u)?);
        }
        // Cannot end with '.'
        if chars.last() == Some(&'.') {
            chars.pop();
            chars.push('_');
        }
        let name: String = chars.into_iter().collect();
        ResourceGroupName::try_new(CompactString::from(name))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(ResourceGroupName);
cloud_terrastodon_registry::register_arbitrary!(ResourceGroupName);

#[cfg(test)]
mod test {
    use super::*;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::RngExt;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        assert!(ResourceGroupName::try_new("bruh").is_ok());
        assert!(ResourceGroupName::try_new("-").is_ok());
        assert!(ResourceGroupName::try_new("a-b-c").is_ok());
        assert!(ResourceGroupName::try_new("hi+hi").is_err()); // '+' is not allowed
        assert!(ResourceGroupName::try_new("").is_err()); // too short
        assert!(ResourceGroupName::try_new("a").is_ok());
        assert!(ResourceGroupName::try_new("aa").is_ok());
        assert!(ResourceGroupName::try_new("JEOFF").is_ok());
        assert!(ResourceGroupName::try_new("caPital").is_ok());
        assert!(ResourceGroupName::try_new("aaaa").is_ok());
        assert!(ResourceGroupName::try_new("a".repeat(89)).is_ok());
        assert!(ResourceGroupName::try_new("a".repeat(90)).is_ok());
        assert!(ResourceGroupName::try_new("a".repeat(91)).is_err()); // too long
        // Test Unicode letters
        assert!(ResourceGroupName::try_new("αβΓδЖЩ").is_ok());
        // Test period at the end
        assert!(ResourceGroupName::try_new("abc.").is_err());
        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 128];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = ResourceGroupName::arbitrary(&mut un)?;
            assert!(name.validate_slug().is_ok());
        }
        Ok(())
    }

    #[test]
    pub fn json_roundtrips() -> eyre::Result<()> {
        let name = ResourceGroupName::try_new("My-RG")?;
        crate::facet_json_equivalence::assert_json_serialize_equivalent(&name)?;
        crate::facet_json_equivalence::assert_json_roundtrip_equivalent::<ResourceGroupName>(
            "\"My-RG\"",
        )?;
        Ok(())
    }
}
