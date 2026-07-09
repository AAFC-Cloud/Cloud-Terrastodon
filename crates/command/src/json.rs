use eyre::Result;
use facet::Facet;
use std::io::Write;

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
