use crate::CommandBuilder;
pub use bstr;
use bstr::BString;
use bstr::ByteSlice;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Error;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
#[cfg(not(windows))]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::process::ExitStatus;
use std::process::Output;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CommandOutput {
    pub stdout: BString,
    pub stderr: BString,
    pub status: i32,
}
impl CommandOutput {
    pub fn success(&self) -> bool {
        #[cfg(windows)]
        return ExitStatus::from_raw(self.status as u32).success();
        #[cfg(not(windows))]
        return ExitStatus::from_raw(self.status).success();
    }
    #[track_caller]
    pub async fn try_interpret<T: DeserializeOwned>(
        &self,
        command: &CommandBuilder,
    ) -> eyre::Result<T> {
        match serde_json::from_slice(self.stdout.to_str_lossy().as_bytes()) {
            Ok(results) => Ok(results),
            Err(e) => {
                let dir = command.write_failure(self).await?;
                Err(eyre::Error::new(e)
                    .wrap_err(format!(
                        "Called from {}",
                        RelativeLocation::from(std::panic::Location::caller())
                    ))
                    .wrap_err(format!(
                        "deserializing `{}` failed, dumped to {:?}",
                        command.summarize().await,
                        dir
                    )))
            }
        }
    }
    /// Keeps only the first and last 500 lines of stdout and stderr
    pub fn shorten(&mut self) {
        fn keep_first_and_last_500_lines_with_warning(output: BString) -> BString {
            let lines: Vec<&[u8]> = output.lines().collect();
            let total = lines.len();

            if total <= 1000 {
                output
            } else {
                let mut trimmed = Vec::new();
                trimmed.extend_from_slice(&lines[..500]);

                // Add truncation warning
                let warning = b"...[output truncated: middle lines omitted]...";
                trimmed.push(warning);

                trimmed.extend_from_slice(&lines[total - 500..]);
                BString::from(trimmed.join(&b'\n'))
            }
        }
        let stdout = std::mem::take(&mut self.stdout);
        self.stdout = keep_first_and_last_500_lines_with_warning(stdout);

        let stderr = std::mem::take(&mut self.stderr);
        self.stderr = keep_first_and_last_500_lines_with_warning(stderr);
    }
}
impl std::error::Error for CommandOutput {}
impl std::fmt::Display for CommandOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "status={}\nstdout={}\nstderr={}",
            self.status, self.stdout, self.stderr
        ))
    }
}
impl TryFrom<Output> for CommandOutput {
    type Error = Error;
    fn try_from(value: Output) -> Result<Self> {
        Ok(CommandOutput {
            // stdout: String::from_utf8_lossy_owned(value.stdout),
            // stderr: String::from_utf8_lossy_owned(value.stderr),
            stdout: BString::from(value.stdout),
            stderr: BString::from(value.stderr),
            status: match value.status.code().unwrap_or(1) {
                x if x < 0 => 1,
                x => x,
            },
        })
    }
}
