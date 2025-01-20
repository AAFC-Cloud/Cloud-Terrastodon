use crate::prelude::pick;
use crate::prelude::FzfArgs;

pub fn are_you_sure(message: String) -> anyhow::Result<bool> {
    let choices = ["Yes", "No"];
    let chosen = pick(FzfArgs {
        choices: choices.into(),
        header: Some(message),
        prompt: Some("Are you sure? ".to_string())
    })?;
    Ok(chosen == "Yes")
}
