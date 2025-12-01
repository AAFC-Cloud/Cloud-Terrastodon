use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;

pub trait PathMapper: Send + Sync + 'static + std::fmt::Debug {
    fn map_path<'a>(&self, path: &'a Path) -> Cow<'a, Path>;
}

#[derive(Debug)]
pub struct PrefixPathMapper {
    pub prefix: OsString,
}
impl PrefixPathMapper {
    pub fn new<S: AsRef<OsStr>>(prefix: S) -> Self {
        Self {
            prefix: prefix.as_ref().to_os_string(),
        }
    }
}
impl PathMapper for PrefixPathMapper {
    fn map_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        if path.starts_with(&self.prefix) {
            return Cow::Borrowed(path);
        }
        // Add the prefix, but don't use [`PathBuf::join`] since we don't want the path separator
        let mut new_path = self.prefix.clone();
        new_path.push(path);
        Cow::Owned(new_path.into())
    }
}

#[cfg(test)]
mod test {
    use crate::PathMapper;
    use crate::PrefixPathMapper;
    use std::path::{Path, PathBuf};

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let mapper = PrefixPathMapper::new("@");
        let path = PathBuf::from("abc.json");
        let mapped = mapper.map_path(path.as_path());
        assert_eq!(mapped.as_ref(), Path::new("@abc.json"));
        Ok(())
    }
}
