use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_hcl_types::AsHclString;

/// Formats HCL text through the configured Terraform executable.
#[expect(
    clippy::double_must_use,
    reason = "async-trait adds #[must_use] to an already must-use future"
)]
#[async_trait::async_trait]
pub trait AsFormattedHclString: AsHclString + Sync {
    /// Passes the text to `terraform fmt -` using the caller's cancellation scope.
    async fn as_formatted_hcl_string(
        &self,
        cancellation: &CancellationScope,
    ) -> eyre::Result<String> {
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.cancellation_safe();
        cmd.args(["fmt", "-"]);
        cmd.send_stdin(self.as_hcl_string());
        let output = cmd.run_raw(cancellation).await?;
        Ok(output.stdout.to_string())
    }
}

impl<T: AsHclString + Sync + ?Sized> AsFormattedHclString for T {}

#[cfg(test)]
mod test {
    use crate::AsFormattedHclString;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn send_stdin_fmt() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let content = r#"resource "time_static" "wait_1_second" {
depends_on = []
triggers_complete = null
}
"#;
                let expected = r#"resource "time_static" "wait_1_second" {
  depends_on        = []
  triggers_complete = null
}
"#;
                assert_eq!(
                    content.as_formatted_hcl_string(&cancellation).await?.trim(),
                    expected.trim()
                );
                Ok(())
            })
            .await
    }
}
