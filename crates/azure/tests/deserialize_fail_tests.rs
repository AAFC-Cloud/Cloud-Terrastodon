use serde::Deserialize;

#[test]
fn bruh() -> eyre::Result<()> {
    let json = r#"{"id":1,"name":"Test"}"#;
    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct Response {
        id: u32,
        name: String,
    }
    let v: Response = serde_json::from_str(json)?;
    println!("Value: {v:#?}");
    Ok(())
}
#[test]
fn bruh2() -> eyre::Result<()> {
    let json = r#"{
    "id":1,
    "name":null
}"#;
    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct Response {
        id: u32,
        name: String,
    }
    let v: Result<Response, _> = serde_json::from_str(json);
    assert!(v.is_err());
    println!("Value: {v:#?}");
    Ok(())
}

#[test]
fn bruh3() -> eyre::Result<()> {
    let json = r#"[{
    "id":1,
    "name":"some"
},
{
    "id":2,
    "name":null
}
]"#;
    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct Response {
        id: u32,
        name: String,
    }
    let v: Result<Vec<Response>, _> = serde_json::from_str(json);
    assert!(v.is_err());
    println!("Value: {v:#?}");
    Ok(())
}
