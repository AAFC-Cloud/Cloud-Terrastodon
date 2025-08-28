use bstr::ByteSlice;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

#[tokio::test]
async fn echo_works() {
    let mut cmd = CommandBuilder::new(CommandKind::Echo);
    cmd.args(["ahoy", "world"]);
    let x = cmd.run_raw().await.unwrap();
    println!("Got {:?}", x.stdout);
    assert_eq!(x.stdout.trim(), "ahoy world".as_bytes());
}

#[tokio::test]
async fn echo_works2() {
    let mut cmd = CommandBuilder::new(CommandKind::Echo);
    cmd.args(["a\"ho\"y", "w'or\nl'd"]);
    let x = cmd.run_raw().await.unwrap();
    println!("Got {:?}", x.stdout);
    assert_eq!(x.stdout.trim(), "a\"ho\"y w'or\nl'd".as_bytes());
}
