use chrono::DateTime;
use chrono::Local;
use cloud_terrastodon_azure_resource_types::ResourceType;
use facet::Facet;
use facet_json::RawJson;
use ipnetwork::Ipv4Network;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct StringMapDefaultNullProxy(Option<HashMap<String, String>>);

impl From<StringMapDefaultNullProxy> for HashMap<String, String> {
    fn from(value: StringMapDefaultNullProxy) -> Self {
        value.0.unwrap_or_default()
    }
}

impl From<&HashMap<String, String>> for StringMapDefaultNullProxy {
    fn from(value: &HashMap<String, String>) -> Self {
        Self(Some(value.clone()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct VecDefaultNullProxy<T>(Option<Vec<T>>);

impl<T> From<VecDefaultNullProxy<T>> for Vec<T> {
    fn from(value: VecDefaultNullProxy<T>) -> Self {
        value.0.unwrap_or_default()
    }
}

impl<T: Clone> From<&Vec<T>> for VecDefaultNullProxy<T> {
    fn from(value: &Vec<T>) -> Self {
        Self(Some(value.clone()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct HashMapDefaultNullProxy<T>(Option<HashMap<String, T>>);

impl<T> From<HashMapDefaultNullProxy<T>> for HashMap<String, T> {
    fn from(value: HashMapDefaultNullProxy<T>) -> Self {
        value.0.unwrap_or_default()
    }
}

impl<T: Clone> From<&HashMap<String, T>> for HashMapDefaultNullProxy<T> {
    fn from(value: &HashMap<String, T>) -> Self {
        Self(Some(value.clone()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct OptionalNonEmptyStringProxy(pub Option<String>);

impl<T: FromStr> TryFrom<OptionalNonEmptyStringProxy> for Option<T>
where
    T::Err: Into<eyre::Error>,
{
    type Error = eyre::Error;

    fn try_from(value: OptionalNonEmptyStringProxy) -> Result<Self, Self::Error> {
        match value.0 {
            None => Ok(None),
            Some(ref s) if s.is_empty() => Ok(None),
            Some(s) => Ok(Some(s.parse().map_err(Into::into)?)),
        }
    }
}

impl<T> From<&Option<T>> for OptionalNonEmptyStringProxy
where
    T: ToString,
{
    fn from(value: &Option<T>) -> Self {
        Self(value.as_ref().map(|s| s.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(proxy = RawJson<'static>)]
pub struct OptionalBoolOrStringProxy(Option<bool>);

impl TryFrom<RawJson<'static>> for OptionalBoolOrStringProxy {
    type Error = eyre::Error;

    fn try_from(value: RawJson<'static>) -> Result<Self, Self::Error> {
        if value.as_str() == "null" {
            return Ok(Self(None));
        }
        if let Ok(value) = facet_json::from_str::<bool>(value.as_str()) {
            return Ok(Self(Some(value)));
        }
        Ok(Self(Some(
            facet_json::from_str::<String>(value.as_str())?.parse()?,
        )))
    }
}

impl TryFrom<&OptionalBoolOrStringProxy> for RawJson<'static> {
    type Error = eyre::Error;

    fn try_from(value: &OptionalBoolOrStringProxy) -> Result<Self, Self::Error> {
        let json = match value.0 {
            Some(value) => facet_json::to_string(&value)?,
            None => "null".to_owned(),
        };
        Ok(RawJson::from_owned(json))
    }
}

impl From<OptionalBoolOrStringProxy> for Option<bool> {
    fn from(value: OptionalBoolOrStringProxy) -> Self {
        value.0
    }
}

impl From<&Option<bool>> for OptionalBoolOrStringProxy {
    fn from(value: &Option<bool>) -> Self {
        Self(*value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Facet)]
#[facet(transparent)]
pub struct LocalDateTimeEpochSecondsProxy(i64);

impl TryFrom<LocalDateTimeEpochSecondsProxy> for DateTime<Local> {
    type Error = String;

    fn try_from(value: LocalDateTimeEpochSecondsProxy) -> Result<Self, Self::Error> {
        let datetime = DateTime::from_timestamp(value.0, 0)
            .ok_or_else(|| "invalid or out-of-range datetime".to_string())?;
        Ok(datetime.with_timezone(&Local))
    }
}

impl From<&DateTime<Local>> for LocalDateTimeEpochSecondsProxy {
    fn from(value: &DateTime<Local>) -> Self {
        Self(value.timestamp())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct IpAddrProxy(String);

impl TryFrom<IpAddrProxy> for IpAddr {
    type Error = std::net::AddrParseError;

    fn try_from(value: IpAddrProxy) -> Result<Self, Self::Error> {
        value.0.parse()
    }
}

impl From<&IpAddr> for IpAddrProxy {
    fn from(value: &IpAddr) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct OptionalIpAddrProxy(Option<IpAddrProxy>);

impl TryFrom<OptionalIpAddrProxy> for Option<IpAddr> {
    type Error = std::net::AddrParseError;

    fn try_from(value: OptionalIpAddrProxy) -> Result<Self, Self::Error> {
        value.0.map(IpAddr::try_from).transpose()
    }
}

impl From<&Option<IpAddr>> for OptionalIpAddrProxy {
    fn from(value: &Option<IpAddr>) -> Self {
        Self(value.as_ref().map(IpAddrProxy::from))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct IpAddrVecProxy(Vec<IpAddrProxy>);

impl TryFrom<IpAddrVecProxy> for Vec<IpAddr> {
    type Error = std::net::AddrParseError;

    fn try_from(value: IpAddrVecProxy) -> Result<Self, Self::Error> {
        value.0.into_iter().map(IpAddr::try_from).collect()
    }
}

impl From<&Vec<IpAddr>> for IpAddrVecProxy {
    fn from(value: &Vec<IpAddr>) -> Self {
        Self(value.iter().map(IpAddrProxy::from).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct IpAddrVecDefaultNullProxy(Option<Vec<IpAddrProxy>>);

impl TryFrom<IpAddrVecDefaultNullProxy> for Vec<IpAddr> {
    type Error = std::net::AddrParseError;

    fn try_from(value: IpAddrVecDefaultNullProxy) -> Result<Self, Self::Error> {
        value
            .0
            .unwrap_or_default()
            .into_iter()
            .map(IpAddr::try_from)
            .collect()
    }
}

impl From<&Vec<IpAddr>> for IpAddrVecDefaultNullProxy {
    fn from(value: &Vec<IpAddr>) -> Self {
        Self(Some(value.iter().map(IpAddrProxy::from).collect()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct ResourceTypeProxy(String);

impl TryFrom<ResourceTypeProxy> for ResourceType {
    type Error = std::convert::Infallible;

    fn try_from(value: ResourceTypeProxy) -> Result<Self, Self::Error> {
        ResourceType::from_str(&value.0)
    }
}

impl From<&ResourceType> for ResourceTypeProxy {
    fn from(value: &ResourceType) -> Self {
        Self(value.as_ref().to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct Ipv4NetworkProxy(String);

impl TryFrom<Ipv4NetworkProxy> for Ipv4Network {
    type Error = ipnetwork::IpNetworkError;

    fn try_from(value: Ipv4NetworkProxy) -> Result<Self, Self::Error> {
        value.0.parse()
    }
}

impl From<&Ipv4Network> for Ipv4NetworkProxy {
    fn from(value: &Ipv4Network) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct Ipv4NetworkVecProxy(Vec<Ipv4NetworkProxy>);

impl TryFrom<Ipv4NetworkVecProxy> for Vec<Ipv4Network> {
    type Error = ipnetwork::IpNetworkError;

    fn try_from(value: Ipv4NetworkVecProxy) -> Result<Self, Self::Error> {
        value.0.into_iter().map(Ipv4Network::try_from).collect()
    }
}

impl From<&Vec<Ipv4Network>> for Ipv4NetworkVecProxy {
    fn from(value: &Vec<Ipv4Network>) -> Self {
        Self(value.iter().map(Ipv4NetworkProxy::from).collect())
    }
}
