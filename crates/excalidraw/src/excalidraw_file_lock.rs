use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Exclusive ownership of one shared drawing's sibling lock file. Never steals
/// an abandoned lock: deleting one requires the operator to verify its owner.
pub(crate) struct ExcalidrawFileLock {
    path: PathBuf,
    file: Option<File>,
}

impl ExcalidrawFileLock {
    pub(crate) fn acquire(path: PathBuf) -> io::Result<Self> {
        let mut attempts = 0;
        let file = loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => break file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        format!(
                            "drawing is locked: {} (verify its owner before removing an abandoned lock)",
                            path.display()
                        ),
                    ));
                }
                // A Windows lock just unlinked by its owner can briefly remain
                // delete-pending. Do not misreport permanent ACL failures as a
                // busy lock: after a bounded retry retain the original error.
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
        let mut lock = Self {
            path,
            file: Some(file),
        };
        let pid = std::process::id();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        // Only generated numeric values are interpolated; this matches the
        // extension's version-1 shared-file protocol, not scene metadata.
        writeln!(
            lock.file.as_mut().unwrap(),
            "{{\"version\":1,\"owner\":\"ct:{pid}:{nonce}\",\"pid\":{pid}}}"
        )?;
        Ok(lock)
    }
}

impl Drop for ExcalidrawFileLock {
    fn drop(&mut self) {
        // Windows requires closing the handle before unlinking the lock.
        drop(self.file.take());
        for attempt in 0..=5 {
            match std::fs::remove_file(&self.path) {
                Err(error)
                    if cfg!(windows)
                        && attempt < 5
                        && matches!(error.raw_os_error(), Some(5 | 32 | 33)) =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                _ => break,
            }
        }
    }
}
