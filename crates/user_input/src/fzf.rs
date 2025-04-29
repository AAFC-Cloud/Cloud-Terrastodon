use eyre::Context;
use eyre::ContextCompat;
use eyre::Error;
use eyre::Result;
use eyre::eyre;
use indexmap::IndexSet;
use itertools::Itertools;
use std::ffi::OsStr;
use std::fmt::Display;
use std::io::ErrorKind;
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
    /// The list of items being chosen from
    pub choices: Vec<T>,

    /// The term that appears before the user's text input
    pub prompt: Option<String>,

    /// The term that appears between the item list and the user's text input
    pub header: Option<String>,

    /// The default search term
    pub query: Option<String>,
}
impl<T> Default for FzfArgs<T> {
    fn default() -> Self {
        Self {
            choices: Default::default(),
            prompt: Default::default(),
            header: Default::default(),
            query: Default::default(),
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
    let mut child = match cmd.spawn() {
        Ok(x) => x,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            #[cfg(windows)]
            return Err(e).wrap_err("Is fzf installed?\nhttps://github.com/junegunn/fzf?tab=readme-ov-file#windows-packages");
            #[cfg(not(windows))]
            return Err(e).wrap_err("Is fzf installed?\nhttps://github.com/junegunn/fzf?tab=readme-ov-file#linux-packages");
        }
        x => x?,
    };

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
    if let Some(query) = &options.query {
        args.push("--query".as_ref());
        args.push(query.as_ref());
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
