use crate::ArtifactFilePath;
use crate::artifact_file::ArtifactFile;
use crate::artifact_file_digest::ArtifactFileDigest;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

/// A directory address confined to one explicit artifact-store root.
///
/// Construction establishes lexical confinement without accessing the filesystem.
/// Every I/O operation checks the original root and its descendants again; a
/// previously constructed handle does not imply that directories remain valid.
/// Ancestors above the explicitly selected root remain the caller's responsibility.
#[derive(Clone, Debug)]
pub(crate) struct ArtifactDirectory {
    // Private fields preserve the relationship between the owner root and path.
    root: PathBuf,
    path: PathBuf,
}

impl ArtifactDirectory {
    pub fn new(root: &Path, path: impl Into<PathBuf>) -> Result<Self> {
        ensure!(root.is_absolute(), "artifact root must be absolute");
        ensure!(
            !root
                .components()
                .any(|component| component == Component::ParentDir),
            "artifact root must not contain unresolved parent traversal"
        );
        let path = path.into();
        let relative = path
            .strip_prefix(root)
            .wrap_err("artifact path escaped its owner root")?;
        ensure!(
            relative
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
            "artifact path contains a nonrelative component"
        );
        Ok(Self {
            root: root.to_path_buf(),
            path,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Address a nonempty relative descendant without changing its trust boundary.
    pub fn child(&self, relative: impl AsRef<Path>) -> Result<Self> {
        let relative = relative.as_ref();
        ensure!(
            !relative.as_os_str().is_empty()
                && relative
                    .components()
                    .all(|component| matches!(component, Component::Normal(_))),
            "artifact child must be a nonempty relative nontraversing path"
        );
        Self::new(&self.root, self.path.join(relative))
    }

    /// Create an exclusive descendant after checking and creating its parents.
    /// An existing directory, file, or link at the final address is an error.
    pub fn create_new_child(&self, relative: impl AsRef<Path>) -> Result<Self> {
        let child = self.child(relative)?;
        let parent = Self::new(&self.root, child.path.parent().unwrap())?;
        parent.ensure_exists()?;
        fs::create_dir(&child.path).wrap_err("creating unique artifact directory")?;
        Ok(child)
    }

    /// Create missing directories while rejecting existing files or links.
    pub fn ensure_exists(&self) -> Result<()> {
        ensure!(
            self.inspect(true)?,
            "artifact directory disappeared while being created"
        );
        Ok(())
    }

    /// Check the whole owner-relative directory chain without creating anything.
    pub fn exists_checked(&self) -> Result<bool> {
        self.inspect(false)
    }

    pub fn file(&self, path: &ArtifactFilePath) -> Result<ArtifactFile> {
        ArtifactFile::new(self, path)
    }

    /// Write one immutable inventory. Exclusive file creation rejects physical
    /// aliases, including case aliases on case-insensitive filesystems.
    pub fn write_files(
        &self,
        files: BTreeMap<ArtifactFilePath, Vec<u8>>,
    ) -> Result<Vec<ArtifactFileDigest>> {
        let mut records = Vec::with_capacity(files.len());
        for (path, bytes) in files {
            self.file(&path)?.create_new(&bytes)?;
            records.push(ArtifactFileDigest::new(path, &bytes));
        }
        Ok(records)
    }

    /// Read an inventory only when every listed file and digest is intact.
    /// Empty inventories do not require a physical directory to have been created.
    pub fn read_verified_files(
        &self,
        files: &[ArtifactFileDigest],
    ) -> Result<Option<BTreeMap<ArtifactFilePath, Vec<u8>>>> {
        let mut result = BTreeMap::new();
        for file in files {
            let Some(bytes) = self.file(&file.path)?.read()? else {
                return Ok(None);
            };
            if !file.digest.matches(&bytes) || result.insert(file.path.clone(), bytes).is_some() {
                return Ok(None);
            }
        }
        Ok(Some(result))
    }

    fn inspect(&self, create: bool) -> Result<bool> {
        if create {
            fs::create_dir_all(&self.root).wrap_err("creating explicit artifact root")?;
        }
        if !Self::is_directory(&self.root)? {
            return Ok(false);
        }
        let mut current = self.root.clone();
        // The constructor establishes this relationship, and fields are private.
        for component in self.path.strip_prefix(&self.root)?.components() {
            current.push(component);
            if create {
                match fs::create_dir(&current) {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error).wrap_err("creating artifact directory"),
                }
            }
            if !Self::is_directory(&current)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn is_directory(path: &Path) -> Result<bool> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error).wrap_err("inspecting artifact directory"),
        };
        ensure!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "artifact directory must be a directory, not a file or link"
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descendant_handles_retain_the_owner_root_without_filesystem_access() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("not-created");
        let owner = ArtifactDirectory::new(&root, &root)?;
        let nested = owner.child("attempts")?.child("one/inputs")?;
        assert_eq!(nested.root(), root);
        assert_eq!(nested.path(), root.join("attempts/one/inputs"));
        assert!(!root.exists());
        for invalid in ["", ".", "..", "nested/../outside"] {
            assert!(owner.child(invalid).is_err());
        }
        assert!(owner.child(temporary.path()).is_err());
        assert!(ArtifactDirectory::new(&root, temporary.path().join("outside")).is_err());
        assert!(ArtifactDirectory::new(&root, root.join("child/../outside")).is_err());
        assert!(
            ArtifactDirectory::new(Path::new("relative"), Path::new("relative/child")).is_err()
        );
        assert!(ArtifactDirectory::new(&root.join("child/.."), root.join("child/../a")).is_err());
        Ok(())
    }

    #[test]
    fn previously_constructed_handles_recheck_replaced_ancestors() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let owner = ArtifactDirectory::new(temporary.path(), temporary.path())?;
        let ancestor = owner.child("ancestor")?;
        let nested = ancestor.child("nested")?;
        nested.ensure_exists()?;
        let file = nested.file(&ArtifactFilePath::new("result.bin")?)?;
        fs::rename(ancestor.path(), temporary.path().join("original-ancestor"))?;
        fs::write(ancestor.path(), b"replacement file")?;

        assert!(nested.exists_checked().is_err());
        assert!(nested.ensure_exists().is_err());
        assert!(file.read().is_err());
        assert!(file.create_new(b"must not be written").is_err());
        assert!(
            file.replace_json(&String::from("must not be written"))
                .is_err()
        );
        assert_eq!(fs::read(ancestor.path())?, b"replacement file");
        assert!(
            !temporary
                .path()
                .join("original-ancestor/nested/result.bin")
                .exists()
        );
        Ok(())
    }
}
