use bstr::ByteSlice;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

#[tokio::test]
async fn echo_works() {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let mut cmd = CommandBuilder::new(CommandKind::Echo);
            cmd.args(["ahoy", "world"]);
            let x = cmd.run_raw(&cancellation).await.unwrap();
            assert_eq!(x.stdout.trim(), "ahoy world".as_bytes());

            Ok::<_, eyre::Report>(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn echo_works2() {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let mut cmd = CommandBuilder::new(CommandKind::Echo);
            cmd.args(["a\"ho\"y", "w'or\nl'd"]);
            let x = cmd.run_raw(&cancellation).await.unwrap();
            assert_eq!(x.stdout.trim(), "a\"ho\"y w'or\nl'd".as_bytes());

            Ok::<_, eyre::Report>(())
        })
        .await
        .unwrap();
}
