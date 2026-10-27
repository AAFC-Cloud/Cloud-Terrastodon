use crate::excalidraw_file_lock::ExcalidrawFileLock;
use eyre::Context;
use eyre::Result;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// A local drawing participating in the version-1 cooperative file protocol.
///
/// Writers acquire `<filename>.sync-lock`, reread, merge, and atomically replace
/// the file with a flushed sibling temporary file. Readers see one complete
/// old or new snapshot. This does not coordinate nonparticipating writers or
/// promise power-loss durability on every filesystem. Methods never wait for a
/// lock: callers decide when to retry and how to honour cancellation.
#[derive(Debug, Clone)]
pub struct ExcalidrawSharedFile {
    // Canonicalization ensures junction/symlink aliases use the same lock.
    path: PathBuf,
}

impl ExcalidrawSharedFile {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let parent = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
                let name = path.file_name().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "drawing needs a filename")
                })?;
                fs::canonicalize(parent)?.join(name)
            }
            Err(error) => return Err(error),
        };
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Opening a handle before reading retains the old snapshot across atomic
    /// replacement. Missing documents are distinct from invalid/unreadable ones.
    pub fn read(&self) -> io::Result<Option<String>> {
        const LIMIT: u64 = 32 * 1024 * 1024;
        let mut attempts = 0;
        let file = loop {
            match fs::File::open(&self.path) {
                Ok(file) => break file,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(error)
                    if cfg!(windows)
                        && attempts < 5
                        && matches!(error.raw_os_error(), Some(5 | 32 | 33)) =>
                {
                    attempts += 1;
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => return Err(error),
            }
        };
        let mut text = String::new();
        file.take(LIMIT + 1).read_to_string(&mut text)?;
        if text.len() as u64 > LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "interactive drawing exceeds the 32 MiB read limit",
            ));
        }
        Ok(Some(text))
    }

    /// Apply a change to the latest disk version under an exclusive lock.
    /// `None` or identical bytes leave the file untouched. On contention the
    /// error retains an `io::Error` with `WouldBlock` for caller retry policy.
    /// Keep the callback bounded and local; never perform service calls here.
    pub fn update(
        &self,
        change: impl FnOnce(Option<&str>) -> Result<Option<String>>,
    ) -> Result<bool> {
        let mut lock_name = self.path.as_os_str().to_os_string();
        lock_name.push(".sync-lock");
        let _lock = ExcalidrawFileLock::acquire(PathBuf::from(lock_name))?;
        let before = self
            .read()
            .wrap_err("reading shared drawing while locked")?;
        let Some(after) = change(before.as_deref())? else {
            return Ok(false);
        };
        if before.as_deref() == Some(after.as_str()) {
            return Ok(false);
        }
        let mut temporary = NamedTempFile::new_in(self.path.parent().unwrap())
            .wrap_err("creating drawing's sibling temporary file")?;
        temporary
            .write_all(after.as_bytes())
            .wrap_err("writing complete drawing snapshot")?;
        if let Ok(metadata) = fs::metadata(&self.path) {
            temporary
                .as_file()
                .set_permissions(metadata.permissions())?;
        }
        temporary
            .as_file()
            .sync_all()
            .wrap_err("flushing drawing snapshot")?;
        // Windows may briefly deny replacement while another application reads
        // the destination. Retain the complete temporary file and lock while
        // retrying; never unlink the old drawing as a replacement fallback.
        for attempt in 0..=5 {
            match temporary.persist(&self.path) {
                Ok(_) => break,
                Err(error)
                    if cfg!(windows)
                        && attempt < 5
                        && matches!(error.error.raw_os_error(), Some(5 | 32 | 33)) =>
                {
                    temporary = error.file;
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => return Err(error).wrap_err("atomically replacing shared drawing"),
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn failure_and_noop_preserve_file_and_release_lock() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let file = ExcalidrawSharedFile::new(directory.path().join("test.excalidraw"))?;
        assert!(file.update(|_| Ok(Some("original".into())))?);
        assert!(
            file.update(|_| {
                eyre::bail!("test failure");
            })
            .is_err()
        );
        assert_eq!(file.read()?.as_deref(), Some("original"));
        assert!(!file.update(|before| Ok(before.map(str::to_owned)))?);
        assert!(!file.update(|_| Ok(None))?);
        assert_eq!(fs::read_dir(directory.path())?.count(), 1);
        Ok(())
    }

    #[test]
    fn concurrent_writers_reread_and_readers_never_see_partial_snapshots() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let file = Arc::new(ExcalidrawSharedFile::new(
            directory.path().join("test.excalidraw"),
        )?);
        file.update(|_| Ok(Some("0".into())))?;
        std::thread::scope(|scope| {
            let writers: Vec<_> = (0..4)
                .map(|_| {
                    let file = file.clone();
                    scope.spawn(move || -> Result<()> {
                        for _ in 0..25 {
                            loop {
                                match file.update(|before| {
                                    Ok(Some((before.unwrap().parse::<u32>()? + 1).to_string()))
                                }) {
                                    Ok(true) => break,
                                    Err(error)
                                        if error.downcast_ref::<io::Error>().is_some_and(
                                            |error| error.kind() == io::ErrorKind::WouldBlock,
                                        ) =>
                                    {
                                        std::thread::yield_now()
                                    }
                                    other => {
                                        other?;
                                    }
                                }
                            }
                            file.read()?.unwrap().parse::<u32>()?;
                        }
                        Ok(())
                    })
                })
                .collect();
            for writer in writers {
                writer.join().unwrap()?;
            }
            Ok::<_, eyre::Report>(())
        })?;
        assert_eq!(file.read()?.as_deref(), Some("100"));
        Ok(())
    }

    #[test]
    fn abandoned_lock_is_never_stolen() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let file = ExcalidrawSharedFile::new(directory.path().join("test.excalidraw"))?;
        fs::write(
            directory.path().join("test.excalidraw.sync-lock"),
            "owner evidence",
        )?;
        let error = file
            .update(|_| panic!("contended transaction must not execute"))
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(
            fs::read_to_string(directory.path().join("test.excalidraw.sync-lock"))?,
            "owner evidence"
        );
        Ok(())
    }
}
