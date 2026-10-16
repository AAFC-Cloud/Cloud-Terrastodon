use bstr::ByteSlice;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

#[tokio::test]
async fn send_stdin_echo() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let mut cmd = CommandBuilder::new(CommandKind::Pwsh);
            cmd.args(["-NoProfile", "-Command", "-" /* Read from stdin */]); // For pwsh, "-" means read from stdin for command
            cmd.send_stdin("echo 'hello stdin'");
            let output = cmd.run_raw(&cancellation).await?;
            assert_eq!(output.stdout.trim(), "hello stdin".as_bytes());
            Ok(())
        })
        .await
}
