//! A helper for converting std::panic::Location to relative paths with displayed.
//! 
//! When crossing crate boundaries calling a function annotated with 
//! 
//! ```rust
//! #[track_caller]
//! ```
//! 
//! the value returned by 
//! 
//! ```rust
//! std::panic::Location::caller()
//! ```
//! 
//! uses an absolute path.
//! 
//! This crate strips the path segments that are also present in the path of this crate at build time.
//! 
//! This means it probably won't work outside of building Cloud Terrastodon 
//! 
//! ㄟ( ▔, ▔ )ㄏ

use std::panic::Location;
use std::path::PathBuf;

pub struct RelativeLocation {
    pub inner: &'static Location<'static>,
}

impl From<&'static Location<'static>> for RelativeLocation {
    fn from(value: &'static Location<'static>) -> Self {
        Self { inner: value }
    }
}

impl std::fmt::Display for RelativeLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Get the manifest directory from the environment variable
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR").trim());
        // Get the full file path from the panic Location
        let full_path = PathBuf::from(self.inner.file());

        // Split both paths into their components
        let manifest_components: Vec<_> = manifest_dir.components().collect();
        let file_components: Vec<_> = full_path.components().collect();

        // Determine the number of common components
        let mut common = 0;
        while common < manifest_components.len()
            && common < file_components.len()
            && manifest_components[common] == file_components[common]
        {
            common += 1;
        }

        // Set the number of parents from the common prefix to retain.
        let retain_n_parents = 1;
        // We'll skip only the components that are not part of the last two common components.
        // If there are fewer common components than retain_n_parents, we skip none.
        let skip = common.saturating_sub(retain_n_parents);

        // Build the resulting relative path:
        let mut relative_path = PathBuf::new();
        for component in file_components.iter().skip(skip) {
            relative_path.push(component.as_os_str());
        }

        // Fallback to current directory if relative_path is empty.
        if relative_path.as_os_str().is_empty() {
            relative_path.push(".");
        }

        // Write out the final relative path using fully qualified std::fmt.
        write!(
            f,
            "{}:{}:{}",
            &relative_path.display(),
            self.inner.line(),
            self.inner.column()
        )
    }
}
