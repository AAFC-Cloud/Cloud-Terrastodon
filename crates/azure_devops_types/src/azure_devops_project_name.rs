use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// https://learn.microsoft.com/en-us/azure/devops/organizations/settings/naming-restrictions?view=azure-devops#project-names
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct AzureDevOpsProjectName {
    inner: CompactString,
}
impl AzureDevOpsProjectName {
    pub fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_azure_devops_project_name(&inner)?;
        Ok(Self { inner })
    }
}

/// Uniqueness
/// - Must not be identical to any other name in the project collection, the SharePoint Web application that supports the collection, or the instance of SQL Server Reporting Services that supports the collection.
/// - We will not check this here.
///
/// Reserves names
/// - Must not be a system reserved name.
///     - AUX, COM1, COM2, COM3, COM4, COM5, COM6, COM7, COM8, COM9, COM10, CON, DefaultCollection, LPT1, LPT2, LPT3, LPT4, LPT5, LPT6, LPT7, LPT8, LPT9, NUL, PRN, SERVER, SignalR, Web, WEB
///     - For more information about reserved names, see [File names, paths, and namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file).
/// - Must not be one of the hidden segments used for IIS request filtering like App_Browsers, App_code, App_Data, App_GlobalResources, App_LocalResources, App_Themes, App_WebResources, bin, or web.config.
///
/// Special characters
/// - Must not contain any Unicode control characters or surrogate characters.
/// - Must not contain the following printable characters: \ / : * ? " ' < > ; # $ * { } , + = [ ] |.
/// - Must not start with an underscore _.
/// - Must not start or end with a period .
fn validate_azure_devops_project_name(value: &CompactString) -> eyre::Result<()> {
    validate_azure_devops_project_name_inner(value)
        .wrap_err_with(|| format!("Invalid Azure DevOps project name: {}", value))
        .wrap_err("https://learn.microsoft.com/en-us/azure/devops/organizations/settings/naming-restrictions?view=azure-devops#project-names")
}

fn validate_azure_devops_project_name_inner(value: &CompactString) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if char_count == 0 || char_count > 64 {
        bail!("length must be between 1 and 64 characters");
    }
    let s: &str = value;

    // 1. Must not start with "_" or . / Must not end with "."
    if s.starts_with('_') {
        bail!("cannot start with underscore");
    }
    if s.starts_with('.') || s.ends_with('.') {
        bail!("cannot start or end with period");
    }

    // 2. Must not contain control or surrogate characters
    for (i, ch) in s.chars().enumerate() {
        if is_control_or_surrogate(ch) {
            bail!(
                "contains Unicode control or surrogate character: '{}' at index {}",
                ch,
                i
            );
        }
    }

    // 3. Must not contain forbidden printable characters
    for (i, ch) in s.chars().enumerate() {
        if FORBIDDEN_CHARS.contains(&ch) {
            bail!(
                "contains forbidden character in name: '{}' at index {}",
                ch,
                i
            );
        }
    }

    // 4. Case-insensitive check for reserved names and segments.
    let lower = s.to_ascii_lowercase();
    for &name in RESERVED_NAMES {
        if lower == name.to_ascii_lowercase() {
            bail!("name is reserved: {:?}", s);
        }
    }
    for &segment in IIS_HIDDEN_SEGMENTS {
        if lower == segment.to_ascii_lowercase() {
            bail!("name is reserved IIS segment: {:?}", s);
        }
    }

    Ok(())
}

const FORBIDDEN_CHARS: &[char] = &[
    '\\', '/', ':', '*', '?', '"', '\'', '<', '>', ';', '#', '$', '{', '}', ',', '+', '=', '[',
    ']', '|',
];
const RESERVED_NAMES: &[&str] = &[
    "AUX",
    "COM1",
    "COM2",
    "COM3",
    "COM4",
    "COM5",
    "COM6",
    "COM7",
    "COM8",
    "COM9",
    "COM10",
    "CON",
    "DefaultCollection",
    "LPT1",
    "LPT2",
    "LPT3",
    "LPT4",
    "LPT5",
    "LPT6",
    "LPT7",
    "LPT8",
    "LPT9",
    "NUL",
    "PRN",
    "SERVER",
    "SignalR",
    "Web",
    "WEB",
];
const IIS_HIDDEN_SEGMENTS: &[&str] = &[
    "App_Browsers",
    "App_code",
    "App_Data",
    "App_GlobalResources",
    "App_LocalResources",
    "App_Themes",
    "App_WebResources",
    "bin",
    "web.config",
];
fn is_control_or_surrogate(ch: char) -> bool {
    ch.is_control() || {
        let v = ch as u32;
        (0xD800..=0xDFFF).contains(&v)
    }
}

fn valid_first_char(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<char> {
    // Valid first char: any Unicode scalar value, but not _ or . or forbidden or control/surrogate
    for _ in 0..16 {
        let ch = rand_valid_nonforbidden_char(u)?;
        if ch != '_' && ch != '.' {
            return Ok(ch);
        }
    }
    // fallback
    Ok('A')
}
fn valid_last_char(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<char> {
    for _ in 0..16 {
        let ch = rand_valid_nonforbidden_char(u)?;
        if ch != '.' {
            return Ok(ch);
        }
    }
    Ok('B')
}

fn rand_valid_nonforbidden_char(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<char> {
    // 90% ascii, 10% random unicode
    if u.arbitrary::<u8>()? < 230 {
        // Pick from safe set
        const SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789- ";
        Ok(*u.choose(SAFE)? as char)
    } else {
        // unicode, filter all rules
        for _ in 0..16 {
            let ch = core::char::from_u32(u.int_in_range(0..=0x10000)?).unwrap_or('A');
            if !is_control_or_surrogate(ch)
                && !FORBIDDEN_CHARS.contains(&ch)
                && ch != '_'
                && ch != '.'
            {
                return Ok(ch);
            }
        }
        Ok('C')
    }
}
impl<'a> Arbitrary<'a> for AzureDevOpsProjectName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Loop until we get a name that passes validation
        for _ in 0..40 {
            let len = u.int_in_range(1..=64)?;
            let mut chars = Vec::with_capacity(len);

            if len == 1 {
                chars.push(valid_first_char(u)?);
            } else if len > 1 {
                chars.push(valid_first_char(u)?);
                for _ in 1..len - 1 {
                    chars.push(rand_valid_nonforbidden_char(u)?);
                }
                chars.push(valid_last_char(u)?);
            }

            let candidate: String = chars.into_iter().collect();
            // Ensure not a reserved or IIS segment name (case insensitive)
            let lower = candidate.to_ascii_lowercase();
            let is_reserved = RESERVED_NAMES
                .iter()
                .any(|&n| lower == n.to_ascii_lowercase())
                || IIS_HIDDEN_SEGMENTS
                    .iter()
                    .any(|&n| lower == n.to_ascii_lowercase());
            if is_reserved {
                continue;
            }

            // Quick check: validate contents
            if let Ok(name) = AzureDevOpsProjectName::try_new(candidate.as_str()) {
                return Ok(name);
            }
            // Else, try again
        }
        Err(arbitrary::Error::IncorrectFormat)
    }
}

impl Deref for AzureDevOpsProjectName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl std::fmt::Display for AzureDevOpsProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl AzureDevOpsProjectName {
    pub fn new(name: String) -> AzureDevOpsProjectName {
        AzureDevOpsProjectName { inner: name.into() }
    }
}
impl FromStr for AzureDevOpsProjectName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureDevOpsProjectName::new(s.to_string()))
    }
}
impl AsRef<str> for AzureDevOpsProjectName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl From<AzureDevOpsProjectName> for CompactString {
    fn from(val: AzureDevOpsProjectName) -> Self {
        val.inner
    }
}
impl From<AzureDevOpsProjectName> for String {
    fn from(val: AzureDevOpsProjectName) -> Self {
        val.inner.into()
    }
}

impl serde::Serialize for AzureDevOpsProjectName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzureDevOpsProjectName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_azure_devops_project_name as validate;
    use crate::azure_devops_project_name::FORBIDDEN_CHARS;
    use crate::azure_devops_project_name::IIS_HIDDEN_SEGMENTS;
    use crate::azure_devops_project_name::RESERVED_NAMES;
    use crate::prelude::AzureDevOpsProjectName;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use compact_str::CompactString;
    use std::str::FromStr;

    #[test]
    fn test_length_bounds() {
        assert!(validate(&CompactString::from("a")).is_ok());
        assert!(validate(&CompactString::from("a".repeat(64))).is_ok());
        assert!(validate(&CompactString::from("")).is_err());
        assert!(validate(&CompactString::from("a".repeat(65))).is_err());
    }

    #[test]
    fn test_start_and_end_rules() {
        assert!(validate(&CompactString::from("_abcdef")).is_err());
        assert!(validate(&CompactString::from(".abcdef")).is_err());
        assert!(validate(&CompactString::from("abc.")).is_err());
        assert!(validate(&CompactString::from("abc")).is_ok());
    }

    #[test]
    fn test_forbidden_characters() {
        for &c in FORBIDDEN_CHARS {
            let s = format!("a{}b", c);
            assert!(
                validate(&CompactString::from(s)).is_err(),
                "Should reject char {:?}",
                c
            );
        }
        assert!(validate(&CompactString::from(r"proj\name")).is_err());
        assert!(validate(&CompactString::from("project:name")).is_err());
    }

    #[test]
    fn test_control_and_surrogate_characters() {
        // Control char: '\x07'
        assert!(validate(&CompactString::from(format!("abc{}def", '\x07'))).is_err());
        // Surrogates (should never occur in Rust chars), but test anyway
        let surrogate = unsafe { CompactString::from_utf8_unchecked(0xD800u16.to_le_bytes()) };
        let result = validate(&CompactString::from(format!("foo{}bar", surrogate)));
        println!("{:?}", result);
        assert!(result.is_err(), "Surrogate character should be rejected");
    }

    #[test]
    fn test_reserved_names() {
        for &name in RESERVED_NAMES {
            assert!(
                validate(&CompactString::from(name)).is_err(),
                "Reserved: {}",
                name
            );
            // Also test lowercase
            assert!(
                validate(&CompactString::from(name.to_ascii_lowercase())).is_err(),
                "Reserved (lower): {}",
                name
            );
        }
    }

    #[test]
    fn test_iis_hidden_segments() {
        for &name in IIS_HIDDEN_SEGMENTS {
            assert!(
                validate(&CompactString::from(name)).is_err(),
                "IIS Segment: {}",
                name
            );
            // Also test lowercase
            assert!(
                validate(&CompactString::from(name.to_ascii_lowercase())).is_err(),
                "IIS Segment (lower): {}",
                name
            );
        }
    }

    #[test]
    fn test_valid_name() {
        assert!(validate(&CompactString::from("Valid-Project_123")).is_ok());
        assert!(validate(&CompactString::from("Project 1 2 3")).is_ok());
    }

    #[test]
    fn roundtrip_display_and_fromstr() {
        let name = "RustProj42";
        let proj = AzureDevOpsProjectName::from_str(name).unwrap();
        assert_eq!(proj.to_string(), name);
        assert_eq!(proj.as_ref(), name);
    }

    #[test]
    fn arbitrary_generates_valid_names() {
        // Run 100 random cases
        for _ in 0..100 {
            let raw: Vec<u8> = (0..128).map(|_| rand::random::<u8>()).collect();
            let mut u = Unstructured::new(&raw);
            if let Ok(proj) = AzureDevOpsProjectName::arbitrary(&mut u) {
                println!("Generated: {}", proj);
                let validation = validate(&proj.inner);
                assert!(
                    validation.is_ok(),
                    "Arbitrary produced invalid: {:?} - {:?}",
                    &proj.inner,
                    validation
                );
            }
        }
    }
}
