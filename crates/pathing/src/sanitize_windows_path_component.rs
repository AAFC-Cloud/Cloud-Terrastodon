/// Encode a string as one readable Windows filename or directory component.
///
/// Reserved characters, whitespace, controls, and `%` are escaped as uppercase
/// UTF-8 `%HH` bytes. Trailing dots and reserved device names (including names
/// with extensions) are escaped too. Empty strings become `%EMPTY`, distinct
/// from a literal `%EMPTY` input. Ordinary Unicode and letter case are preserved.
///
/// Components longer than 220 UTF-16 units use a readable prefix of at most 160
/// units followed by a BLAKE3 digest of the original string. The result is at most
/// 225 UTF-16 units. This is a component policy; it does not bound a complete
/// path or distinguish names by case on case-insensitive filesystems.
///
/// See Microsoft's [Windows naming conventions](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions).
#[must_use]
pub fn sanitize_windows_path_component(value: &str) -> String {
    if value.is_empty() {
        return "%EMPTY".to_owned();
    }
    let stem = value
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|suffix| {
                matches!(
                    suffix,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            });
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let trailing_dots = value.trim_end_matches('.').len();
    let mut encoded = String::new();
    for (index, character) in value.char_indices() {
        if character.is_whitespace()
            || character.is_control()
            || "%\\/:*?\"<>|".contains(character)
            || (index == 0 && device)
            || (character == '.' && index >= trailing_dots)
        {
            let mut bytes = [0; 4];
            for byte in character.encode_utf8(&mut bytes).bytes() {
                encoded.push('%');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0F) as usize] as char);
            }
        } else {
            encoded.push(character);
        }
    }
    if encoded.encode_utf16().count() > 220 {
        // Keep a readable prefix within Windows' component limit. The full
        // value's digest keeps shortened names independent.
        let mut units = 0;
        let mut prefix = String::new();
        for character in encoded.chars() {
            if units + character.len_utf16() > 160 {
                break;
            }
            prefix.push(character);
            units += character.len_utf16();
        }
        format!("{prefix}-{}", blake3::hash(value.as_bytes()).to_hex())
    } else {
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_windows_path_component;

    #[test]
    fn readable_components_avoid_reserved_names_and_bound_long_names() {
        assert_eq!(sanitize_windows_path_component("CON.txt"), "%43ON.txt");
        assert_eq!(sanitize_windows_path_component("COM¹.txt"), "%43OM¹.txt");
        assert_eq!(sanitize_windows_path_component(".."), "%2E%2E");
        assert_eq!(sanitize_windows_path_component(""), "%EMPTY");
        assert_eq!(sanitize_windows_path_component("%EMPTY"), "%25EMPTY");
        assert_eq!(sanitize_windows_path_component("Résumé"), "Résumé");

        for (character, prefix_length) in [("語", 160), ("🦀", 80)] {
            let long_name = character.repeat(300);
            let shortened = sanitize_windows_path_component(&long_name);
            assert!(shortened.starts_with(&character.repeat(prefix_length)));
            assert!(shortened.encode_utf16().count() <= 225);
            assert_ne!(
                shortened,
                sanitize_windows_path_component(&(long_name + "文"))
            );
        }
    }

    #[test]
    fn components_escape_separators_controls_and_trailing_dots() {
        assert_eq!(
            sanitize_windows_path_component(r#"\/:*?"<>|%"#),
            "%5C%2F%3A%2A%3F%22%3C%3E%7C%25"
        );
        assert_eq!(
            sanitize_windows_path_component("a b\t\n\0\u{a0}"),
            "a%20b%09%0A%00%C2%A0"
        );
        assert_eq!(sanitize_windows_path_component("name.. "), "name..%20");
        assert_eq!(sanitize_windows_path_component("name.."), "name%2E%2E");
        assert_eq!(sanitize_windows_path_component(".name"), ".name");
    }
}
