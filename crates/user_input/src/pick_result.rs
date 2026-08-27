pub type PickResult<T, S = T> = Result<T, PickError<S>>;
pub type PickManyResult<T> = Result<Vec<T>, PickError<T>>;

pub trait PickResultExt {
    type Item;

    fn into_chosen_and_maybe_error(self) -> eyre::Result<(Vec<Self::Item>, eyre::Result<()>)>;
}

#[derive(Debug)]
pub enum PickError<T> {
    Eyre(eyre::Error, Vec<T>),
    Cancelled,
    NoChoicesProvided,
    ReloadRequested,
}
impl<T> std::fmt::Display for PickError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PickError::Eyre(e, _) => write!(f, "PickError: {}", e),
            PickError::Cancelled => write!(
                f,
                "PickError: The operation was cancelled by the user pressing Esc or Ctrl+C."
            ),
            PickError::NoChoicesProvided => {
                write!(f, "PickError: The list of choices to pick from was empty.")
            }
            PickError::ReloadRequested => {
                write!(f, "PickError: A reload of choices was requested.")
            }
        }
    }
}
impl<T> PartialEq for PickError<T> {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, other) {
            (PickError::Cancelled, PickError::Cancelled) => true,
            (PickError::NoChoicesProvided, PickError::NoChoicesProvided) => true,
            (PickError::ReloadRequested, PickError::ReloadRequested) => true,
            _ => false,
        }
    }
}
impl<T> From<PickError<T>> for eyre::Error {
    #[track_caller]
    fn from(value: PickError<T>) -> Self {
        match value {
            PickError::Eyre(e, _) => e,
            _ => eyre::eyre!(value.to_string()),
        }
    }
}
impl<S> From<eyre::Error> for PickError<S> {
    #[track_caller]
    fn from(value: eyre::Error) -> Self {
        PickError::Eyre(value, Vec::new())
    }
}
impl<S> From<std::io::Error> for PickError<S> {
    #[track_caller]
    fn from(value: std::io::Error) -> Self {
        PickError::Eyre(eyre::eyre!(value.to_string()), Vec::new())
    }
}

impl<T> PickResultExt for PickManyResult<T> {
    type Item = T;

    fn into_chosen_and_maybe_error(self) -> eyre::Result<(Vec<Self::Item>, eyre::Result<()>)> {
        match self {
            Ok(chosen) => Ok((chosen, Ok(()))),
            Err(PickError::Eyre(error, chosen)) => Ok((chosen, Err(error))),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::PickError;
    use crate::PickManyResult;
    use crate::PickResult;
    use crate::PickResultExt;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let result: PickResult<(), ()> = PickResult::Ok(());
        result?;
        assert_eq!((), ());
        Ok(())
    }

    #[test]
    fn pick_result_ext_preserves_successful_choices() {
        let result: PickManyResult<i32> = Ok(vec![1, 2]);
        let (chosen, maybe_error) = result
            .into_chosen_and_maybe_error()
            .expect("successful picker result should project");

        assert_eq!(chosen, vec![1, 2]);
        assert!(maybe_error.is_ok());
    }

    #[test]
    fn pick_result_ext_preserves_choices_from_eyre_errors() {
        let (chosen, maybe_error) = PickResultExt::into_chosen_and_maybe_error(Err(
            PickError::Eyre(eyre::eyre!("handler failed"), vec![1, 2]),
        ))
        .expect("recoverable picker error should project");

        assert_eq!(chosen, vec![1, 2]);
        assert!(maybe_error.is_err());
    }
}
