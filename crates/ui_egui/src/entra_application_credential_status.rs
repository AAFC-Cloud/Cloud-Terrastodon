use crate::entra_application_credential_metadata::EntraApplicationCredentialMetadata;
use cloud_terrastodon_azure::EntraApplicationRegistration;
use jiff::Timestamp;

/// Summarizes whether the application has any currently valid keys or secrets.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EntraApplicationCredentialStatus {
    None,
    Expired,
    NotYetValid,
    Unknown,
    Current,
}

impl EntraApplicationCredentialStatus {
    pub fn for_application(registration: &EntraApplicationRegistration, now: Timestamp) -> Self {
        // A known active credential establishes availability. Otherwise an
        // unreadable credential leaves availability unknown.
        registration
            .key_credentials
            .iter()
            .chain(&registration.password_credentials)
            .map(|credential| {
                EntraApplicationCredentialMetadata::read(credential)
                    .map_or(Self::Unknown, |metadata| metadata.status(now))
            })
            .max()
            .unwrap_or(Self::None)
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Expired => "Expired",
            Self::NotYetValid => "Not yet valid",
            Self::Unknown => "Unknown",
            Self::Current => "Current",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validity_starts_at_the_start_time_and_ends_at_expiration() -> eyre::Result<()> {
        let start: Timestamp = "2025-01-01T00:00:00Z".parse()?;
        let end: Timestamp = "2026-01-01T00:00:00Z".parse()?;
        let metadata = EntraApplicationCredentialMetadata {
            display_name: None,
            start_date_time: Some(start),
            end_date_time: Some(end),
        };
        assert_eq!(
            metadata.status(start - std::time::Duration::from_secs(1)),
            EntraApplicationCredentialStatus::NotYetValid
        );
        assert_eq!(
            metadata.status(start),
            EntraApplicationCredentialStatus::Current
        );
        assert_eq!(
            metadata.status(end),
            EntraApplicationCredentialStatus::Expired
        );
        Ok(())
    }
}
