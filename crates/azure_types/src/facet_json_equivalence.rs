use facet::Facet;

pub fn assert_json_serialize_equivalent<T>(value: &T) -> eyre::Result<()>
where
    T: Facet<'static>,
{
    let facet_json = facet_json::to_string(value)?;
    let _: facet_json::RawJson<'static> = facet_json::from_str(&facet_json)?;
    Ok(())
}

pub fn assert_json_roundtrip_equivalent<T>(json: &str) -> eyre::Result<()>
where
    T: Facet<'static> + std::fmt::Debug + PartialEq,
{
    let facet_value = facet_json::from_str::<T>(json)?;
    let serialized = facet_json::to_string(&facet_value)?;
    let roundtrip = facet_json::from_str::<T>(&serialized)?;
    assert_eq!(facet_value, roundtrip);
    Ok(())
}
