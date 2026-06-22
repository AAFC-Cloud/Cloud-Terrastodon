use facet::Facet;
use facet_json::RawJson;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum AllOr<T> {
    All,
    AllTrusted,
    MicrosoftAdminPortals,
    None,
    Some(T),
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct AllOrVecProxy(Vec<RawJson<'static>>);

impl<T> TryFrom<AllOrVecProxy> for Vec<AllOr<T>>
where
    T: Facet<'static>,
{
    type Error = eyre::Error;

    fn try_from(value: AllOrVecProxy) -> Result<Self, Self::Error> {
        value.0.into_iter().map(AllOr::try_from).collect()
    }
}

impl<T> TryFrom<&Vec<AllOr<T>>> for AllOrVecProxy
where
    T: Facet<'static>,
{
    type Error = eyre::Error;

    fn try_from(value: &Vec<AllOr<T>>) -> Result<Self, Self::Error> {
        value
            .iter()
            .map(RawJson::try_from)
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

impl<T> TryFrom<RawJson<'static>> for AllOr<T>
where
    T: Facet<'static>,
{
    type Error = eyre::Error;

    fn try_from(value: RawJson<'static>) -> Result<Self, Self::Error> {
        if value.as_str() == "null" {
            return Ok(Self::None);
        }
        if let Ok(text) = facet_json::from_str::<String>(value.as_str()) {
            if text.eq_ignore_ascii_case("all") {
                return Ok(Self::All);
            }
            if text.eq_ignore_ascii_case("none") {
                return Ok(Self::None);
            }
            if text.eq_ignore_ascii_case("alltrusted") {
                return Ok(Self::AllTrusted);
            }
            if text.eq_ignore_ascii_case("microsoftadminportals") {
                return Ok(Self::MicrosoftAdminPortals);
            }
        }
        Ok(Self::Some(facet_json::from_str(value.as_str())?))
    }
}

impl<T> TryFrom<&AllOr<T>> for RawJson<'static>
where
    T: Facet<'static>,
{
    type Error = eyre::Error;

    fn try_from(value: &AllOr<T>) -> Result<Self, Self::Error> {
        let json = match value {
            AllOr::All => facet_json::to_string("All")?,
            AllOr::None => "null".to_owned(),
            AllOr::AllTrusted => facet_json::to_string("AllTrusted")?,
            AllOr::MicrosoftAdminPortals => facet_json::to_string("MicrosoftAdminPortals")?,
            AllOr::Some(value) => facet_json::to_string(value)?,
        };
        Ok(RawJson::from_owned(json))
    }
}

#[cfg(test)]
mod test {
    use super::AllOr;
    use facet_json::RawJson;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let json_string = "\"all\"";
        let all_or: super::AllOr<String> = RawJson::from_owned(json_string.to_owned()).try_into()?;
        assert_eq!(all_or, super::AllOr::All);

        let json_value = "\"some_value\"";
        let all_or: super::AllOr<String> = RawJson::from_owned(json_value.to_owned()).try_into()?;
        assert_eq!(all_or, super::AllOr::Some("some_value".to_string()));

        let json_value = "42";
        let all_or: super::AllOr<i32> = RawJson::from_owned(json_value.to_owned()).try_into()?;
        assert_eq!(all_or, super::AllOr::Some(42));

        Ok(())
    }

    #[test]
    fn json_matches_serde_through_proxy_field() -> eyre::Result<()> {
        #[derive(Debug, PartialEq, facet::Facet)]
        struct Fixture {
            #[facet(opaque, proxy = crate::AllOrVecProxy)]
            values: Vec<AllOr<String>>,
        }

        let json = r#"{
            "values": ["all", "none", "AllTrusted", "MicrosoftAdminPortals", "some_value"]
        }"#;

        let fixture: Fixture = facet_json::from_str(json)?;
        assert_eq!(
            fixture.values,
            vec![
                AllOr::All,
                AllOr::None,
                AllOr::AllTrusted,
                AllOr::MicrosoftAdminPortals,
                AllOr::Some("some_value".to_string())
            ]
        );
        let reparsed: Fixture = facet_json::from_str(&facet_json::to_string(&fixture)?)?;
        assert_eq!(fixture, reparsed);
        Ok(())
    }
}
