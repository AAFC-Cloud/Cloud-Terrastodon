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
    type Target = Path;

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
    type Target = Path;

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
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl AsRef<Path> for ValidatedTFWorkDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedConfigOutTFWorkDir(PathBuf);
impl From<ValidatedTFWorkDir> for GeneratedConfigOutTFWorkDir {
    fn from(value: ValidatedTFWorkDir) -> Self {
        GeneratedConfigOutTFWorkDir(value.0)
    }
}
impl Deref for GeneratedConfigOutTFWorkDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl AsRef<Path> for GeneratedConfigOutTFWorkDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
#[derive(Debug, Clone)]
pub struct ProcessedTFWorkDir(PathBuf);
impl ProcessedTFWorkDir {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}
impl From<GeneratedConfigOutTFWorkDir> for ProcessedTFWorkDir {
    fn from(value: GeneratedConfigOutTFWorkDir) -> Self {
        ProcessedTFWorkDir(value.0)
    }
}
impl Deref for ProcessedTFWorkDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl AsRef<Path> for ProcessedTFWorkDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

pub trait TFWorkDir: AsRef<Path> {}
impl TFWorkDir for &FreshTFWorkDir {}
impl TFWorkDir for &InitializedTFWorkDir {}
impl TFWorkDir for &ValidatedTFWorkDir {}
impl TFWorkDir for &GeneratedConfigOutTFWorkDir {}
impl TFWorkDir for &ProcessedTFWorkDir {}
