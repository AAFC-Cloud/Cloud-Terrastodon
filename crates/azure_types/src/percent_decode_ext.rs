use eyre::Result;
use eyre::WrapErr;
use eyre::eyre;
use std::borrow::Cow;

/// Decode percent escapes in a UTF-8 URI component exactly once.
///
/// Unlike form decoding, `+` remains a literal plus. Malformed escapes and
/// invalid UTF-8 are errors; components without escapes are borrowed unchanged.
pub trait PercentDecodeExt {
    fn percent_decode(&self) -> Result<Cow<'_, str>>;
}

impl<T: AsRef<str> + ?Sized> PercentDecodeExt for T {
    fn percent_decode(&self) -> Result<Cow<'_, str>> {
        let value = self.as_ref();
        let Some(start) = value.find('%') else {
            return Ok(Cow::Borrowed(value));
        };
        let bytes = value.as_bytes();
        let mut decoded = Vec::with_capacity(bytes.len());
        decoded.extend_from_slice(&bytes[..start]);
        let mut index = start;
        while index < bytes.len() {
            if bytes[index] == b'%' {
                let high = bytes
                    .get(index + 1)
                    .and_then(|byte| char::from(*byte).to_digit(16));
                let low = bytes
                    .get(index + 2)
                    .and_then(|byte| char::from(*byte).to_digit(16));
                let (Some(high), Some(low)) = (high, low) else {
                    return Err(eyre!(
                        "invalid percent escape at byte {index}: expected two hexadecimal digits"
                    ));
                };
                decoded.push(((high << 4) | low) as u8);
                index += 3;
            } else {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
        String::from_utf8(decoded)
            .map(Cow::Owned)
            .wrap_err("percent-decoded URI component is not valid UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::PercentDecodeExt;
    use crate::PercentEncodeExt;
    use std::borrow::Cow;

    #[test]
    fn decodes_components_once_without_form_semantics() -> eyre::Result<()> {
        assert!(matches!("é+雪".percent_decode()?, Cow::Borrowed("é+雪")));
        assert_eq!("%C3%a9+%E9%9B%AA%2F%252F".percent_decode()?, "é+雪/%2F");
        for value in ["", "abc-._~123", "O'Neil@example.com", "é/雪+ %2F"] {
            assert_eq!(value.percent_encode().percent_decode()?, value);
        }
        Ok(())
    }

    #[test]
    fn rejects_malformed_escapes_and_invalid_utf8() {
        for value in ["%", "suffix%0", "%GG", "%0g", "%FF", "%C3%28", "%C0%AF"] {
            assert!(value.percent_decode().is_err(), "{value}");
        }
    }
}
