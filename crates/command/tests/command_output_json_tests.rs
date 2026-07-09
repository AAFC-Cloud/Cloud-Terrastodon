use bstr::BString;
use cloud_terrastodon_command::CommandOutput;

#[test]
fn command_output_bstrings_serialize_as_byte_arrays_without_proxies() -> eyre::Result<()> {
    let output = CommandOutput {
        stdout: BString::from(vec![104, 105, 255]),
        stderr: BString::from(vec![0, 1, 2]),
        status: 0,
    };

    let json = facet_json::to_string(&output).map_err(|error| eyre::eyre!("{error:?}"))?;
    assert_eq!(
        json,
        r#"{"stdout":[104,105,255],"stderr":[0,1,2],"status":0}"#
    );

    let round_trip: CommandOutput =
        facet_json::from_str(&json).map_err(|error| eyre::eyre!("{error:?}"))?;
    assert_eq!(round_trip, output);
    Ok(())
}
