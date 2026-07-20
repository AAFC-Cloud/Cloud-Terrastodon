pub trait PercentEncodeExt {
    fn percent_encode(&self) -> String;
}

impl<T: AsRef<str> + ?Sized> PercentEncodeExt for T {
    fn percent_encode(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";

        let value = self.as_ref();
        let mut encoded = String::with_capacity(value.len());
        for byte in value.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                encoded.push(byte as char);
            } else {
                encoded.push('%');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0F) as usize] as char);
            }
        }
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::PercentEncodeExt;

    #[test]
    fn percent_encodes_reserved_characters() {
        assert_eq!(
            "O'Neil@example.com".percent_encode(),
            "O%27Neil%40example.com"
        );
    }

    #[test]
    fn preserves_unreserved_characters() {
        assert_eq!("abc-._~123".percent_encode(), "abc-._~123");
    }
}
