#![cfg(test)]

use bstr::ByteSlice;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

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
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let mut cmd = CommandBuilder::new(CommandKind::Echo);
            // cmd.args(["ad","user","show","--id",""]);
            cmd.args(["aéa"]);
            let x = cmd.run_raw(&cancellation).await.unwrap();
            println!("Got {x:?}");
            println!("Expected: {:?}", "aéa".as_bytes());
            println!("Given:    {:?}", x.stdout.trim());
            // assert_eq!(x.stdout.trim(), "aéa".as_bytes());
            assert_eq!(x.stdout.trim().to_str().unwrap(), "aéa");

            Ok::<_, eyre::Report>(())
        })
        .await
        .unwrap();
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
