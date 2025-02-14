use eyre::eyre;
use eyre::ContextCompat;
use eyre::Error;
use eyre::Result;
use indexmap::IndexSet;
use itertools::Itertools;
use std::ffi::OsStr;
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

impl<T> PartialEq for Choice<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}
impl<T> Eq for Choice<T> where T: Eq {}

#[derive(Debug)]
pub struct FzfArgs<T> {
    pub choices: Vec<T>,
    pub prompt: Option<String>,
    pub header: Option<String>,
}
impl<T> Default for FzfArgs<T> {
    fn default() -> Self {
        Self {
            choices: Default::default(),
            prompt: Default::default(),
            header: Default::default(),
        }
    }
}
impl<T> From<Vec<T>> for FzfArgs<T> {
    fn from(value: Vec<T>) -> Self {
        FzfArgs {
            choices: value,
            ..Default::default()
        }
    }
}

fn inner<T, I, S>(choices: Vec<Choice<T>>, additional_args: I) -> Result<Vec<T>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("fzf");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.args(["--height", "~9999"]);
    cmd.args(["--read0", "--print0"]);
    cmd.arg("--highlight-line");
    cmd.args([
        "--bind",
        "ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all",
    ]);
    cmd.args(additional_args);
    let mut child = cmd.spawn()?;

    // Write the choices to fzf's stdin
    {
        let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
        let choices = choices.iter().map(|choice| choice.to_string()).join("\0");
        stdin.write_all(choices.as_bytes())?;
    }

    // Wait for output
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.is_empty() {
            stdout = "<empty stdout>".to_string();
        }
        let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if stderr.is_empty() {
            stderr = "<empty stderr>".to_string();
        }
        let msg = format!(
            "exit={}\nstdout={stdout}\nstderr={stderr}",
            output
                .status
                .code()
                .map(|x| x.to_string())
                .unwrap_or("code was None".to_string())
        );
        return Err(Error::msg(msg).wrap_err("did you ctrl+c?"));
    }

    // Parse output
    let response_string = String::from_utf8_lossy(&output.stdout);
    let chosen_set = response_string.split("\0").collect::<IndexSet<&str>>();
    let chosen = choices
        .into_iter()
        .filter(|c| chosen_set.contains(c.key.as_str()))
        .map(|c| c.value)
        .collect_vec();
    Ok(chosen)
}

fn outer<T, I, S>(args: impl Into<FzfArgs<T>>, additional_args: I) -> Result<Vec<T>>
where
    T: Into<Choice<T>>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let options: FzfArgs<_> = args.into();
    let choices = options.choices.into_iter().map(|x| x.into()).collect_vec();
    let mut args: Vec<&OsStr> = Vec::new();
    if let Some(prompt) = &options.prompt {
        args.push("--prompt".as_ref());
        args.push(prompt.as_ref());
    }
    if let Some(header) = &options.header {
        args.push("--header".as_ref());
        args.push(header.as_ref());
    }
    let holder = additional_args.into_iter().collect_vec();
    for arg in holder.iter() {
        args.push(arg.as_ref())
    }

    inner(choices, args)
}

/// Prompt the user to pick from a predetermined list of options.
pub fn pick<T>(args: impl Into<FzfArgs<T>>) -> Result<T>
where
    T: Into<Choice<T>>,
{
    outer(args, [] as [&str; 0])?
        .into_iter()
        .next()
        .ok_or(eyre!("No results were received"))
}

/// Prompt the user to pick from a predetermined list of options.
pub fn pick_many<T>(args: impl Into<FzfArgs<T>>) -> Result<Vec<T>>
where
    T: Into<Choice<T>>,
{
    outer(args, ["--multi"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_works() -> Result<()> {
        let choices = vec![
            "Choice 1",
            "Choice 2",
            "Choice 3",
            "Choice 4",
            "Choice 5\nspans two lines",
            "Choice 6\nalso does",
        ];
        let options = FzfArgs {
            choices,
            prompt: Some("Pick things".to_string()),
            header: Some("bruh".to_string()),
        };
        let chosen = pick_many(options)?;
        println!("You chose: {:?}", chosen);

        Ok(())
    }
    #[test]
    #[ignore]
    fn multiline() -> Result<()> {
        let choices = vec![
            "First\nSecond\nThird",
            "A\nB\nC",
            "IMPORT BRUH\nDO THING\nWOOHOO!",
            "single item",
            "another single item",
        ];
        let chosen = pick_many(choices)?;
        println!("You chose: {chosen:#?}");
        Ok(())
    }
}
