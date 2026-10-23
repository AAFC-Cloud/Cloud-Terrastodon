use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::IntoBlocks;
use std::collections::HashSet;

#[expect(
    clippy::double_must_use,
    reason = "async-trait adds #[must_use] to an already must-use future"
)]
#[async_trait::async_trait]
pub trait AsHclString {
    fn as_hcl_string(&self) -> String;
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
impl AsHclString for String {
    fn as_hcl_string(&self) -> String {
        self.to_owned()
    }
}
impl AsHclString for &str {
    fn as_hcl_string(&self) -> String {
        self.to_string()
    }
}

pub trait TryAsHclBlocks {
    fn try_as_hcl_blocks(&self) -> Result<IntoBlocks>;
}
impl<T: AsHclString> TryAsHclBlocks for T {
    fn try_as_hcl_blocks(&self) -> Result<IntoBlocks> {
        Ok(self.as_hcl_string().parse::<Body>()?.into_blocks())
    }
}

impl<T> AsHclString for Vec<T>
where
    T: AsHclString,
{
    fn as_hcl_string(&self) -> String {
        let mut rtn = String::new();
        for v in self.iter() {
            rtn.push_str(v.as_hcl_string().as_str());
            rtn.push('\n');
        }
        rtn
    }
}
impl<T> AsHclString for HashSet<T>
where
    T: AsHclString,
{
    fn as_hcl_string(&self) -> String {
        let mut rtn = String::new();
        for v in self.iter() {
            rtn.push_str(v.as_hcl_string().as_str());
            rtn.push('\n');
        }
        rtn
    }
}

impl AsHclString for Block {
    fn as_hcl_string(&self) -> String {
        Body::builder().block(self.clone()).build().to_string()
    }
}

impl AsHclString for Body {
    fn as_hcl_string(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::strings::AsHclString;
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
