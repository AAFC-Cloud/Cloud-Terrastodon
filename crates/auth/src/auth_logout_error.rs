use eyre::Report;
use std::fmt;

/// Retains every failed cleanup when logging out of independent sources.
#[derive(Debug)]
pub(crate) struct AuthLogoutError {
    pub errors: Vec<Report>,
}

impl fmt::Display for AuthLogoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "Some authentication sources could not be logged out:"
        )?;
        for error in &self.errors {
            writeln!(formatter, "- {error:#}")?;
        }
        Ok(())
    }
}

impl std::error::Error for AuthLogoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.errors.first().map(|error| error.as_ref())
    }
}
