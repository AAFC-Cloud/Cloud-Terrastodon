#[cfg(test)]
mod tests {
    use cloud_terrastodon_command::{CommandBuilder, CommandKind};

    #[tokio::test]
    #[ignore = "ran manually to rebuild type definition"]
    async fn gen_definitions() -> eyre::Result<()> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["account","list-location"])
        Ok(())
    }
}