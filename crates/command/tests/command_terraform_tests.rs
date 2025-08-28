use bstr::ByteSlice;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

#[tokio::test]
async fn send_stdin_terraform_fmt() -> eyre::Result<()> {
    let content = r#"resource "time_static" "wait_1_second" {
depends_on = []
triggers_complete = null
}
"#;
    let mut cmd = CommandBuilder::new(CommandKind::Terraform);
    cmd.args(["fmt", "-"]);
    cmd.send_stdin(content);
    let output = cmd.run_raw().await?;
    println!("Stdout: {:?}", output.stdout);
    let expected = r#"resource "time_static" "wait_1_second" {
  depends_on        = []
  triggers_complete = null
}
"#;
    assert_eq!(output.stdout.trim(), expected.trim().as_bytes());
    Ok(())
}
