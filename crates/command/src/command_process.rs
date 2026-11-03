use crate::CancellationScope;
use crate::CommandBuilder;
use crate::CommandInputWorkspace;
use crate::CommandOutput;
use crate::OutputBehaviour;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Child;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::debug;
use tracing::error;
use tracing::info;

/// One spawned process and the adjacent files needed for its lifetime.
///
/// This owner performs no cache lookup, retry, decoding, or artifact publication.
/// Its caller owns cancellation policy around the attempt and its completion work.
pub(crate) struct CommandProcess {
    // Keep the child before the workspace so dropping an unfinished attempt
    // requests process termination before removing its staged files.
    child: Child,
    inputs: Option<CommandInputWorkspace>,
}

impl CommandProcess {
    pub(crate) async fn spawn(
        builder: &CommandBuilder,
        cancellation: &CancellationScope,
    ) -> Result<Self> {
        let mut command = Command::new(builder.kind.program().await?);
        match builder.output_behaviour {
            OutputBehaviour::Capture => {
                command.stdin(Stdio::piped());
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
            OutputBehaviour::Display => {
                if builder.stdin_content.is_some() {
                    command.stdin(Stdio::piped());
                }
            }
        }
        let inputs = builder
            .kind
            .apply_args_and_envs(builder, &mut command)
            .await
            .wrap_err("applying command arguments and environment")?;
        if let Some(directory) = &builder.run_dir {
            command.current_dir(directory);
        }
        if builder.should_announce {
            info!("Executing command");
        } else {
            debug!("Executing command");
        }
        command.kill_on_drop(true);
        cancellation.bail_if_cancelled()?;
        let child = command.spawn().wrap_err("Failed to spawn command")?;
        Ok(Self { child, inputs })
    }

    pub(crate) async fn wait(
        self,
        stdin_content: Option<&str>,
        timeout_duration: Option<Duration>,
    ) -> Result<CommandOutput> {
        let Self { mut child, inputs } = self;
        let stdin = child.stdin.take();
        let wait = async move {
            let write_stdin = async move {
                if let Some(content) = stdin_content
                    && let Some(mut stdin) = stdin
                    && let Err(error) = stdin.write_all(content.as_bytes()).await
                {
                    // Preserve the existing policy: process output and status
                    // remain authoritative when the child closes its input early.
                    error!("Failed to write to stdin: {error:?}");
                }
            };
            let (output, ()) = tokio::join!(child.wait_with_output(), write_stdin);
            output
        };
        let output = if let Some(duration) = timeout_duration {
            match timeout(duration, wait).await {
                Ok(output) => output,
                Err(elapsed) => {
                    bail!(
                        "Command timeout, {elapsed:?} ({})",
                        humantime::format_duration(duration)
                    );
                }
            }
        } else {
            wait.await
        };
        // Retain files through both output collection and the owned stdin writer.
        drop(inputs);
        output
            .wrap_err("Acquiring result of command execution")?
            .try_into()
            .wrap_err("Converting output of command")
    }
}
