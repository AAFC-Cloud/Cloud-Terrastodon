use bstr::ByteSlice;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::RetryBehaviour;

#[test]
fn encoding() {
    let x = "é";
    let bytes = x.as_bytes().to_vec();
    let _y = String::from_utf8(bytes).unwrap();

    let x = "�";
    let bytes = x.as_bytes().to_vec();
    let _y = String::from_utf8(bytes).unwrap();

    let x = "\u{FFFD}";
    let bytes = x.as_bytes().to_vec();
    let _y = String::from_utf8(bytes).unwrap();
}

#[tokio::test]
async fn encoding_2() {
    let mut cmd = CommandBuilder::new(CommandKind::Echo);
    // cmd.args(["ad","user","show","--id",""]);
    cmd.args(["aéa"]);
    let x = cmd.run_raw().await.unwrap();
    println!("Got {x:?}");
    println!("Expected: {:?}", "aéa".as_bytes());
    println!("Given:    {:?}", x.stdout.trim());
    // assert_eq!(x.stdout.trim(), "aéa".as_bytes());
    assert_eq!(x.stdout.trim().to_str().unwrap(), "aéa");
}

#[tokio::test]
#[ignore]
/// The Azure CLI uses system locale by default, which is latin-1 instead of UTF-8
/// https://github.com/Azure/azure-cli/issues/22616
async fn encoding_3() -> eyre::Result<()> {
    let user_id = cloud_terrastodon_user_input::prompt_line(
        "Enter the ID for the user who is experiencing encoding issues:",
    )
    .await?;
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.use_retry_behaviour(RetryBehaviour::Fail);
    cmd.args([
        "ad",
        "user",
        "show",
        "--id",
        user_id.as_ref(),
        "--query",
        "displayName",
    ]);
    let x = cmd.run_raw().await.unwrap().stdout;
    let bytes = x.as_ref() as &[u8];
    println!("Got {:?}", x);
    println!("Got {:?}", bytes);
    println!(
        "Got {:?}",
        bytes
            .iter()
            .map(|x| char::from_u32(*x as u32))
            .collect::<Vec<_>>()
    );
    let z = String::from_utf8(bytes.to_vec())?;
    println!("Decoded {z:?}");
    let y = x.to_str()?;
    println!("Decoded {y:?}");
    Ok(())
}

#[test]
fn encoding_4() -> eyre::Result<()> {
    let byte = 233_u8;
    println!("{byte} => {:?}", char::from_u32(byte as u32));
    let bytes = vec![byte, 101];
    println!("bytes: {bytes:?}");
    let str = String::from_utf8(bytes);
    println!("str: {str:?}");

    // 233 is a latin-1 not utf-8 valid.
    // Therefore, it should fail.
    assert!(str.is_err());
    Ok(())
}
