use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MicrosoftGraphEntityId {
    inner: CompactString,
}

impl MicrosoftGraphEntityId {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = value.into();
        Ok(Self { inner })
    }
}

impl FromStr for MicrosoftGraphEntityId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MicrosoftGraphEntityId::try_new(s)
    }
}
impl TryFrom<&str> for MicrosoftGraphEntityId {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        MicrosoftGraphEntityId::try_new(value)
    }
}
impl TryFrom<String> for MicrosoftGraphEntityId {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
impl TryFrom<&String> for MicrosoftGraphEntityId {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl std::fmt::Display for MicrosoftGraphEntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for MicrosoftGraphEntityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for MicrosoftGraphEntityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for MicrosoftGraphEntityId {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for MicrosoftGraphEntityId {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<MicrosoftGraphEntityId> for CompactString {
    fn from(value: MicrosoftGraphEntityId) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for MicrosoftGraphEntityId {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Length: 1..=50 Unicode scalar values
        let len = u.int_in_range(1..=50)?;

        // Build string of `len` random Unicode scalar values
        let mut scalars = Vec::with_capacity(len);
        for _ in 0..len {
            // Restrict to legal scalar values, avoid surrogate code points (U+D800..=U+DFFF)
            let mut attempts = 0;
            let c = loop {
                let code = u.arbitrary::<u32>()? % 0x110000;
                // skip UTF-16 surrogates
                if (0xD800..=0xDFFF).contains(&code) || code > 0x10FFFF {
                    attempts += 1;
                    if attempts > 10 {
                        break ' ';
                    }
                    continue;
                }
                if let Some(c) = std::char::from_u32(code) {
                    break c;
                }
                attempts += 1;
                if attempts > 10 {
                    break ' ';
                }
            };
            scalars.push(c);
        }

        let s: String = scalars.into_iter().collect();
        // Wrap and validate, your constructor may have additional checks
        MicrosoftGraphEntityId::try_new(CompactString::from(s))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        use compact_str::CompactString;

        // Minimum valid length (1)
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from("a")).is_ok());
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from(" ")).is_ok());
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from("🌵")).is_ok());

        // Maximum valid length (50)
        let fifty_ascii = "a".repeat(50);
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from(&fifty_ascii)).is_ok());
        let fifty_unicode = "你".repeat(50);
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from(&fifty_unicode)).is_ok());

        // Over max length (51)
        let fifty_one_ascii = "a".repeat(51);
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from(&fifty_one_ascii)).is_err());
        let fifty_one_unicode = "你".repeat(51);
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from(&fifty_one_unicode)).is_err());

        // Empty string (too short)
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from("")).is_err());

        // Various valid names
        assert!(
            MicrosoftGraphEntityId::try_new(CompactString::from("My Azure Subscription")).is_ok()
        );
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from("Österreich #1")).is_ok());
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from("!@#$%^&*()")).is_ok());
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from("😊💡💯✨🍕")).is_ok());
        assert!(
            MicrosoftGraphEntityId::try_new(CompactString::from("東京サブスクリプション")).is_ok()
        );

        // Leading/trailing spaces are allowed
        assert!(MicrosoftGraphEntityId::try_new(CompactString::from("  spaced  ")).is_ok());

        Ok(())
    }

    #[test]
    pub fn fuzz() -> eyre::Result<()> {
        for _ in 0..100 {
            let mut raw = [0u8; 128];
            rand::rng().fill(&mut raw);
            let mut un = Unstructured::new(&raw);
            let name = MicrosoftGraphEntityId::arbitrary(&mut un)?;
            // Name is already validated during construction
            assert!(name.inner.chars().count() >= 1 && name.inner.chars().count() <= 50);
        }
        Ok(())
    }
}
