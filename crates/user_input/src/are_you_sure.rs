use crate::FzfArgs;
use crate::pick;

pub fn are_you_sure(message: impl AsRef<str>) -> eyre::Result<bool> {
    let choices = ["No", "Yes"];
    let chosen = pick(FzfArgs {
        choices: choices.into(),
        header: Some(message.as_ref().to_string()),
        prompt: Some("Are you sure? ".to_string()),
        ..Default::default()
    })?;
    Ok(chosen == "Yes")
}
