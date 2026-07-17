use eyre::Context;
use eyre::bail;
use std::ffi::CStr;
use std::ffi::CString;
use std::str::FromStr;

/// A validated target name used to address a generic Windows Credential Manager entry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WindowsCredentialManagerTargetName(CString);

impl WindowsCredentialManagerTargetName {
    /// Construct a target name suitable for the ANSI Windows Credential Manager APIs.
    pub fn try_new(target_name: impl Into<String>) -> eyre::Result<Self> {
        let target_name = target_name.into();
        if target_name.is_empty() {
            bail!("Windows Credential Manager target name cannot be empty");
        }

        let target_name = CString::new(target_name)
            .wrap_err("Windows Credential Manager target name cannot contain NUL bytes")?;
        Ok(Self(target_name))
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0
            .to_str()
            .expect("Windows Credential Manager target names are constructed from UTF-8 strings")
    }
}

impl FromStr for WindowsCredentialManagerTargetName {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for WindowsCredentialManagerTargetName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for WindowsCredentialManagerTargetName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<&String> for WindowsCredentialManagerTargetName {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_new(value.clone())
    }
}

impl std::fmt::Display for WindowsCredentialManagerTargetName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::WindowsCredentialManagerTargetName;

    #[test]
    fn accepts_a_valid_target_name() -> eyre::Result<()> {
        let target_name = WindowsCredentialManagerTargetName::try_new("cloud-terrastodon:test")?;

        assert_eq!(target_name.as_str(), "cloud-terrastodon:test");
        assert_eq!(target_name.to_string(), "cloud-terrastodon:test");
        Ok(())
    }

    #[test]
    fn rejects_empty_target_names() {
        let error = WindowsCredentialManagerTargetName::try_new("")
            .expect_err("empty target names should be rejected");

        assert!(error.to_string().contains("cannot be empty"));
    }

    #[test]
    fn rejects_target_names_with_nul_bytes() {
        let error = WindowsCredentialManagerTargetName::try_new("cloud\0terrastodon")
            .expect_err("target names with NUL bytes should be rejected");

        assert!(error.to_string().contains("cannot contain NUL bytes"));
    }
}
