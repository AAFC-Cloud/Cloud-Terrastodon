use std::error::Error;

pub type PickResult<T> = Result<T, PickError>;

#[derive(Debug)]
pub enum PickError {
    Eyre(eyre::Error),
    Cancelled,
    NoChoicesProvided,
}
impl std::fmt::Display for PickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PickError::Eyre(e) => write!(f, "PickError: {}", e),
            PickError::Cancelled => write!(
                f,
                "PickError: The operation was cancelled by the user hitting the Esc key."
            ),
            PickError::NoChoicesProvided => {
                write!(f, "PickError: The list of choices to pick from was empty.")
            }
        }
    }
}
impl PartialEq for PickError {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, other) {
            (PickError::Cancelled, PickError::Cancelled) => true,
            (PickError::NoChoicesProvided, PickError::NoChoicesProvided) => true,
            _ => false,
        }
    }
}
impl From<PickError> for eyre::Error {
    #[track_caller]
    fn from(value: PickError) -> Self {
        match value {
            PickError::Eyre(e) => e,
            _ => eyre::eyre!(value.to_string()),
        }
    }
}
impl<T: Error> From<T> for PickError {
    #[track_caller]
    fn from(value: T) -> Self {
        PickError::Eyre(eyre::eyre!(value.to_string()))
    }
}

#[cfg(test)]
mod test {
    use crate::PickResult;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let resp = PickResult::Ok(())?;
        assert_eq!(resp, ());
        Ok(())
    }
}
