use eyre::Result;
pub(crate) use serde_json::json;
use std::io::Write;
pub(crate) type Value = serde_json::Value;

pub(crate) fn from_str<T>(input: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_json::from_str(input)?)
}

pub(crate) fn to_string<T>(value: &T) -> Result<String>
where
    T: serde::Serialize + ?Sized,
{
    Ok(serde_json::to_string(value)?)
}

pub(crate) fn to_writer_pretty<W, T>(writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: serde::Serialize + ?Sized,
{
    serde_json::to_writer_pretty(writer, value)?;
    Ok(())
}
