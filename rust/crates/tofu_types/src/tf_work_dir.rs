use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FreshTFWorkDir(PathBuf);
impl From<&Path> for FreshTFWorkDir {
    fn from(value: &Path) -> Self {
        FreshTFWorkDir(value.to_path_buf())
    }
}
impl From<PathBuf> for FreshTFWorkDir {
    fn from(value: PathBuf) -> Self {
        FreshTFWorkDir(value)
    }
}
impl Deref for FreshTFWorkDir {
    type Target=Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl AsRef<Path> for FreshTFWorkDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct InitializedTFWorkDir(PathBuf);
impl From<FreshTFWorkDir> for InitializedTFWorkDir {
    fn from(value: FreshTFWorkDir) -> Self {
        InitializedTFWorkDir(value.0)
    }
}
impl Deref for InitializedTFWorkDir {
    type Target=Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl AsRef<Path> for InitializedTFWorkDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedTFWorkDir(PathBuf);
impl From<InitializedTFWorkDir> for ValidatedTFWorkDir {
    fn from(value: InitializedTFWorkDir) -> Self {
        ValidatedTFWorkDir(value.0)
    }
}
impl Deref for ValidatedTFWorkDir {
    type Target=Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl AsRef<Path> for ValidatedTFWorkDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

pub trait TFWorkDir: AsRef<Path> {}
impl TFWorkDir for &FreshTFWorkDir {}
impl TFWorkDir for &InitializedTFWorkDir {}
impl TFWorkDir for &ValidatedTFWorkDir {}
