use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use indexmap::IndexSet;
use itertools::Itertools;
use std::fmt::Display;
use std::io::Write;
use std::ops::Deref;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug)]
pub struct Choice<T> {
    pub key: String,
    pub value: T,
}
impl<T> std::fmt::Display for Choice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key)
    }
}

impl<T> From<T> for Choice<T>
where
    T: Display,
{
    fn from(value: T) -> Self {
        Choice {
            key: value.to_string(),
            value,
        }
    }
}

impl<T> Deref for Choice<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct FzfArgs<T> {
    pub choices: Vec<T>,
    pub prompt: Option<String>,
    pub header: Option<String>,
}

/// Prompt the user to pick from a predetermined list of options.
pub fn 	pick_many<T>(args: FzfArgs<T>) -> Result<Vec<T>>
where
    T: Into<Choice<T>>,
{
    // Prepare choices
    let choices: Vec<Choice<T>> = args.choices.into_iter().map(|x| x.into()).collect_vec();

    // Spawn the fzf process
    let mut fzf = Command::new("fzf");
    fzf.stdin(Stdio::piped());
    fzf.stdout(Stdio::piped());
    fzf.arg("--multi");
    fzf.args(["--height", "90%"]);
    if let Some(prompt) = args.prompt {
        fzf.arg("--prompt");
        fzf.arg(prompt);
    }
    if let Some(header) = args.header {
        fzf.arg("--header");
        fzf.arg(header);
    }
    fzf.args([
        "--bind",
        "ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all",
    ]);
    let mut fzf = fzf.spawn()?;

    // Send choices to fzf's stdin
    {
        let stdin = fzf.stdin.as_mut().context("Failed to open stdin")?;
        let choices = choices.iter().map(|choice| choice.to_string()).join("\n");
        stdin.write_all(choices.as_bytes())?;
    }

    // Read the output from fzf's stdout
    let output = fzf.wait_with_output()?;
    if output.status.success() {
        let response_string = String::from_utf8_lossy(&output.stdout);
        let chosen_set = response_string.lines().collect::<IndexSet<&str>>();
        let chosen = choices
            .into_iter()
            .filter(|c| chosen_set.contains(c.key.as_str()))
            .map(|c| c.value)
            .collect_vec();
        Ok(chosen)
    } else {
        let mut error_message = String::from_utf8_lossy(&output.stderr).to_string();
        if error_message.is_empty() {
            error_message = "<empty stderr>".to_string();
        }
        Err(Error::msg(error_message).context("did you ctrl+c?"))
    }
}

/// Prompt the user to pick from a predetermined list of options.
pub fn pick<T>(args: FzfArgs<T>) -> Result<T>
where
    T: Into<Choice<T>>,
{
    // Prepare choices
    let choices: Vec<Choice<T>> = args.choices.into_iter().map(|x| x.into()).collect_vec();

    // Spawn the fzf process
    let mut fzf = Command::new("fzf");
    fzf.stdin(Stdio::piped());
    fzf.stdout(Stdio::piped());
    fzf.args(["--height", "90%"]);
    if let Some(prompt) = args.prompt {
        fzf.arg("--prompt");
        fzf.arg(prompt);
    }
    if let Some(header) = args.header {
        fzf.arg("--header");
        fzf.arg(header);
    }
    fzf.args([
        "--bind",
        "ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all",
    ]);
    let mut fzf = fzf.spawn()?;

    // Send choices to fzf's stdin
    {
        let stdin = fzf.stdin.as_mut().context("Failed to open stdin")?;
        let choices = choices.iter().map(|choice| choice.to_string()).join("\n");
        stdin.write_all(choices.as_bytes())?;
    }

    // Read the output from fzf's stdout
    let output = fzf.wait_with_output()?;
    if output.status.success() {
        let response_string = String::from_utf8_lossy(&output.stdout);
        let chosen_set = response_string.lines().collect::<IndexSet<&str>>();
        let chosen = choices
            .into_iter()
            .filter(|c| chosen_set.contains(c.key.as_str()))
            .map(|c| c.value)
            .next()
            .ok_or(anyhow::anyhow!("No choice present"))?;
        Ok(chosen)
    } else {
        let mut error_message = String::from_utf8_lossy(&output.stderr).to_string();
        if error_message.is_empty() {
            error_message = "<empty stderr>".to_string();
        }
        Err(Error::msg(error_message).context("did you ctrl+c?"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_works() -> Result<()> {
        let choices = vec!["Choice 1", "Choice 2", "Choice 3", "Choice 4"];
        let options = FzfArgs {
            choices,
            prompt: Some("Pick things".to_string()),
            header: Some("bruh".to_string()),
        };
        let chosen = pick_many(options)?;
        println!("You chose: {:?}", chosen);

        Ok(())
    }
}
