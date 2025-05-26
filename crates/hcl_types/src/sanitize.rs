use hcl_primitives::ident::is_id_continue;
use hcl_primitives::ident::is_id_start;

pub trait Sanitizable {
    /// Terraform says:
    /// A name must start with a letter or underscore and may contain only letters, digits, underscores, and dashes.
    fn sanitize(&self) -> String;
}
impl<T: AsRef<str>> Sanitizable for T {
    fn sanitize(&self) -> String {
        let s = self.as_ref();
        
        // First, sanitize all characters
        let sanitized: String = s
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 && is_id_start(c) || is_id_continue(c) {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        
        // Then check if we need to prepend ZZZ_ based on the first character
        match sanitized.chars().next() {
            Some(c) if is_id_start(c) => sanitized,
            Some(_) => format!("ZZZ_{}", sanitized),
            None => "ZZZ_".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Sanitizable;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let x = "123abc";
        let y = x.sanitize();
        assert_eq!(format!("ZZZ_{x}"), y);
        Ok(())
    }

    #[test]
    pub fn test_valid_name_unchanged() -> eyre::Result<()> {
        let x = "valid_name123";
        let y = x.sanitize();
        assert_eq!("valid_name123", y);
        Ok(())
    }

    #[test]
    pub fn test_spaces() -> eyre::Result<()> {
        let x = "some name";
        let y = x.sanitize();
        assert_eq!("some_name", y);
        Ok(())
    }

    #[test]
    pub fn test_policy() -> eyre::Result<()> {
        let x = "3GC PBMM Policy Set";
        let y = x.sanitize();
        println!("{y:?}");
        assert!(!y.contains(" "));
        Ok(())
    }

    #[test]
    pub fn test_underscore_start() -> eyre::Result<()> {
        let x = "_starts_with_underscore";
        let name = x.sanitize();
        assert_eq!("_starts_with_underscore", name);
        Ok(())
    }

    #[test]
    pub fn test_empty_string() -> eyre::Result<()> {
        let x = "";
        let name = x.sanitize();
        assert_eq!("ZZZ_", name);
        Ok(())
    }

    #[test]
    pub fn test_special_characters() -> eyre::Result<()> {
        let x = "hello@world#test";
        let name = x.sanitize();
        assert_eq!("hello_world_test", name);
        Ok(())
    }

    #[test]
    pub fn test_dashes_in_name() -> eyre::Result<()> {
        let x = "valid-name-with-dashes";
        let name = x.sanitize();
        assert_eq!("valid-name-with-dashes", name);
        Ok(())
    }

    #[test]
    pub fn test_only_invalid_characters() -> eyre::Result<()> {
        let x = "@#$%";
        let name = x.sanitize();
        assert_eq!("____", name);
        Ok(())
    }

    #[test]
    pub fn test_unicode_characters() -> eyre::Result<()> {
        let x = "café_naïve";
        let name = x.sanitize();
        assert_eq!("café_naïve", name);
        Ok(())
    }

    #[test]
    pub fn test_numbers_and_letters_mixed() -> eyre::Result<()> {
        let x = "abc123def456";
        let name = x.sanitize();
        assert_eq!("abc123def456", name);
        Ok(())
    }

    #[test]
    pub fn test_single_character_invalid() -> eyre::Result<()> {
        let x = "9";
        let name = x.sanitize();
        assert_eq!("ZZZ_9", name);
        Ok(())
    }

    #[test]
    pub fn test_spaces_and_punctuation() -> eyre::Result<()> {
        let x = "hello world! how are you?";
        let name = x.sanitize();
        assert_eq!("hello_world__how_are_you_", name);
        Ok(())
    }

    #[test]
    pub fn test_consecutive_invalid_chars() -> eyre::Result<()> {
        let x = "valid@@@name";
        let name = x.sanitize();
        assert_eq!("valid___name", name);
        Ok(())
    }
}
