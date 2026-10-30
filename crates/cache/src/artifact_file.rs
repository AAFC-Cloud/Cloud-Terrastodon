use crate::ArtifactFilePath;
use crate::artifact_directory::ArtifactDirectory;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fs;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::Write;
use std::path::PathBuf;

/// A file address whose I/O remains confined to its original artifact root.
///
/// Directory checks run for each operation, including handles constructed before
/// an ancestor was replaced. Existing leaf links and nonregular files are rejected.
/// These checks do not promise coordination with unrelated filesystem writers.
#[derive(Debug)]
pub(crate) struct ArtifactFile {
    // Private fields keep the validated leaf address and its parent together.
    parent: ArtifactDirectory,
    path: PathBuf,
}

impl ArtifactFile {
    pub fn new(directory: &ArtifactDirectory, path: &ArtifactFilePath) -> Result<Self> {
        let path = path.join_to(directory.path());
        let parent = ArtifactDirectory::new(
            directory.root(),
            path.parent()
                .ok_or_eyre("artifact file has no parent directory")?,
        )?;
        Ok(Self { parent, path })
    }

    pub fn read(&self) -> Result<Option<Vec<u8>>> {
        if !self.parent.exists_checked()? || !self.is_regular_file()? {
            return Ok(None);
        }
        fs::read(&self.path)
            .map(Some)
            .wrap_err("reading artifact file")
    }

    /// Remove a mutable selector only after checking its owner-relative address.
    /// Missing records are already invalidated; directories and links are errors.
    pub fn remove(&self) -> Result<()> {
        if !self.parent.exists_checked()? || !self.is_regular_file()? {
            return Ok(());
        }
        fs::remove_file(&self.path).wrap_err("removing reusable artifact selection")
    }

    /// Create a new immutable file, rejecting existing paths and physical aliases.
    pub fn create_new(&self, bytes: &[u8]) -> Result<()> {
        self.parent.ensure_exists()?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
            .wrap_err("creating unique artifact file")?;
        file.write_all(bytes).wrap_err("writing artifact file")?;
        file.flush().wrap_err("flushing artifact file")
    }

    pub fn create_json<T: for<'a> Facet<'a>>(&self, value: &T) -> Result<()> {
        self.create_new(&Self::encode_json(value)?)
    }

    /// Replace a mutable record in an existing directory. The caller owns
    /// publication locking and quarantine; serialization completes before truncation.
    pub fn replace_json<T: for<'a> Facet<'a>>(&self, value: &T) -> Result<()> {
        let bytes = Self::encode_json(value)?;
        ensure!(
            self.parent.exists_checked()?,
            "artifact replacement requires an existing parent directory"
        );
        self.is_regular_file()?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .wrap_err("opening mutable artifact record")?;
        file.write_all(&bytes)
            .wrap_err("writing mutable artifact record")?;
        file.flush().wrap_err("flushing mutable artifact record")
    }

    fn is_regular_file(&self) -> Result<bool> {
        let metadata = match fs::symlink_metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error).wrap_err("inspecting artifact file"),
        };
        ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "artifact file must be a regular file, not a link"
        );
        Ok(true)
    }

    fn encode_json<T: for<'a> Facet<'a>>(value: &T) -> Result<Vec<u8>> {
        facet_json::to_string(value)
            .map(String::into_bytes)
            .wrap_err("encoding artifact metadata")
    }
}
