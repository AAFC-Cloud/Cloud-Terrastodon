use bstr::ByteSlice;
use compact_str::CompactString;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::PathBuf;

pub trait NoSpaces {
    type Output;
    fn no_spaces(self) -> Self::Output;
}

impl NoSpaces for &[u8] {
    type Output = Vec<u8>;

    fn no_spaces(self) -> Self::Output {
        match self.trim_ascii().replace(" ", "_") {
            s if s.is_empty() => "(blankstring)".as_bytes().to_owned(),
            s => s,
        }
    }
}
impl NoSpaces for Vec<u8> {
    type Output = Vec<u8>;

    fn no_spaces(self) -> Self::Output {
        self.as_slice().no_spaces()
    }
}
impl NoSpaces for CompactString {
    type Output = CompactString;

    fn no_spaces(self) -> Self::Output {
        CompactString::from_utf8(self.as_bytes().no_spaces().into_iter()).unwrap()
    }
}
impl NoSpaces for &CompactString {
    type Output = CompactString;

    fn no_spaces(self) -> Self::Output {
        CompactString::from_utf8(self.as_bytes().no_spaces().into_iter()).unwrap()
    }
}
impl NoSpaces for String {
    type Output = String;

    fn no_spaces(self) -> Self::Output {
        String::from_utf8(self.as_bytes().no_spaces()).unwrap()
    }
}
impl NoSpaces for &String {
    type Output = String;

    fn no_spaces(self) -> Self::Output {
        self.as_str().no_spaces()
    }
}
impl NoSpaces for &str {
    type Output = String;

    fn no_spaces(self) -> Self::Output {
        String::from_utf8(self.as_bytes().no_spaces()).unwrap()
    }
}
impl NoSpaces for &PathBuf {
    type Output = PathBuf;

    fn no_spaces(self) -> Self::Output {
        self.to_path_buf().no_spaces()
    }
}
impl NoSpaces for &mut PathBuf {
    type Output = PathBuf;

    fn no_spaces(self) -> Self::Output {
        self.to_path_buf().no_spaces()
    }
}
impl NoSpaces for PathBuf {
    type Output = PathBuf;
    fn no_spaces(self) -> Self::Output {
        let mut data = VecDeque::new();
        let mut components = Vec::new();
        for component in self.components() {
            match component {
                std::path::Component::Normal(os_str) => {
                    let mut bytes = os_str.as_encoded_bytes().trim_ascii().replace(" ", "_");
                    if bytes.is_empty() {
                        bytes = "(blankstring)".as_bytes().to_owned();
                    }
                    let os_string = unsafe { OsString::from_encoded_bytes_unchecked(bytes) };
                    data.push_back(os_string);
                    components.push(None);
                }
                x => components.push(Some(x)),
            }
        }
        let mut path_buf = PathBuf::new();
        for component in components {
            match component {
                Some(x) => path_buf.push(x),
                None => {
                    let last = data.pop_front().unwrap();
                    let component = std::path::Component::Normal(last.as_os_str());
                    path_buf.push(component);
                }
            }
        }
        path_buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compact_str::CompactString;

    #[test]
    fn test_no_spaces() {
        let input = "Hello World";
        let expected = "Hello_World";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_no_spaces_empty() {
        let input = "";
        let expected = "(blankstring)";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_no_spaces_already_no_spaces() {
        let input = "HelloWorld";
        let expected = "HelloWorld";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_compact_string() {
        let input = CompactString::new("  Hello   World  ");
        let expected = "Hello___World";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_string_type() {
        let input = String::from("  Foo Bar Baz  ");
        let expected = "Foo_Bar_Baz";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_string_ref_type() {
        let input = String::from("A B C");
        let expected = "A_B_C";
        assert_eq!((&input).no_spaces(), expected);
    }

    #[test]
    fn test_str_type() {
        let input: &str = "  Rust Language  ";
        let expected = "Rust_Language";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_compact_string_ref_type() {
        let input = CompactString::new("  Compact String  ");
        let expected = "Compact_String";
        assert_eq!((&input).no_spaces(), expected);
    }

    #[test]
    fn test_multiple_spaces() {
        let input = "a   b    c";
        let expected = "a___b____c";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_only_spaces() {
        let input = "     ";
        let expected = "(blankstring)";
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_pathbuf_no_spaces_simple() {
        let input = PathBuf::from("foo bar/baz qux");
        let expected = PathBuf::from("foo_bar/baz_qux");
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_pathbuf_no_spaces_leading_trailing_spaces() {
        let input = PathBuf::from("  foo bar  /  baz  ");
        let expected = PathBuf::from("foo_bar/baz");
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_pathbuf_no_spaces_multiple_spaces() {
        let input = PathBuf::from("a   b/c    d");
        let expected = PathBuf::from("a___b/c____d");
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_pathbuf_no_spaces_already_clean() {
        let input = PathBuf::from("no_spaces/already_clean");
        let expected = PathBuf::from("no_spaces/already_clean");
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn test_pathbuf_no_spaces_only_spaces() {
        let input = PathBuf::from("     /     ");
        dbg!(&input);
        dbg!(input.components().collect::<Vec<_>>());
        let expected = PathBuf::from_iter(vec!["(blankstring)", "(blankstring)"]);
        dbg!(&expected);
        dbg!(expected.components().collect::<Vec<_>>());
        assert_eq!(input.no_spaces(), expected);
    }

    #[test]
    fn understand_pathbuf() {
        let input = PathBuf::from("/");
        dbg!(
            input
                .components()
                .map(|c| (PathBuf::from_iter(vec![&c]), c.as_os_str(), c))
                .collect::<Vec<_>>()
        );
    }
}
