use bstr::BString;
use eyre::Result;
use facet::Facet;
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct BStringJsonProxy(Vec<u8>);

impl From<BStringJsonProxy> for BString {
    fn from(value: BStringJsonProxy) -> Self {
        BString::from(value.0)
    }
}

impl From<&BString> for BStringJsonProxy {
    fn from(value: &BString) -> Self {
        Self(value.as_slice().to_vec())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct BStringMapJsonProxy(BTreeMap<String, BStringJsonProxy>);

impl From<&BTreeMap<String, BString>> for BStringMapJsonProxy {
    fn from(value: &BTreeMap<String, BString>) -> Self {
        Self(
            value
                .iter()
                .map(|(key, value)| (key.clone(), BStringJsonProxy::from(value)))
                .collect(),
        )
    }
}

impl From<BStringMapJsonProxy> for BTreeMap<String, BString> {
    fn from(value: BStringMapJsonProxy) -> Self {
        value
            .0
            .into_iter()
            .map(|(key, value)| (key, BString::from(value)))
            .collect()
    }
}

pub fn from_slice<T>(input: &[u8]) -> Result<T>
where
    T: Facet<'static>,
{
    facet_json::from_slice(input).map_err(|error| eyre::eyre!("{error:?}"))
}

pub fn to_vec<'facet, T>(value: &T) -> Result<Vec<u8>>
where
    T: Facet<'facet>,
{
    facet_json::to_vec(value).map_err(|error| eyre::eyre!("{error:?}"))
}

pub fn to_vec_pretty<'facet, T>(value: &T) -> Result<Vec<u8>>
where
    T: Facet<'facet>,
{
    facet_json::to_vec_pretty(value).map_err(|error| eyre::eyre!("{error:?}"))
}

pub fn to_string<'facet, T>(value: &T) -> Result<String>
where
    T: Facet<'facet>,
{
    facet_json::to_string(value).map_err(|error| eyre::eyre!("{error:?}"))
}

pub fn to_string_pretty<'facet, T>(value: &T) -> Result<String>
where
    T: Facet<'facet>,
{
    facet_json::to_string_pretty(value).map_err(|error| eyre::eyre!("{error:?}"))
}

pub fn to_writer_pretty<'facet, W, T>(writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: Facet<'facet>,
{
    facet_json::to_writer_std_pretty(writer, value).map_err(|error| eyre::eyre!("{error:?}"))
}
