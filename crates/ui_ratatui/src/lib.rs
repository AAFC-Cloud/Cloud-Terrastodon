// Phase 0 builds and tests the replacement engine before the legacy UI is cut
// over. Remove this allowance as vertical paths begin consuming the module.
#[allow(dead_code)]
mod object_browser;
#[allow(dead_code)]
mod object_explorer;
mod projection_shapes;
pub mod role_assignment_picker_app;
mod ui_main;

pub use crate::ui_main::*;

#[cfg(test)]
mod architecture_tests {
    use std::path::{Path, PathBuf};

    fn rust_sources_below(directory: &Path, sources: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory).expect("module directory is readable") {
            let path = entry.expect("module entry is readable").path();
            if path.is_dir() {
                rust_sources_below(&path, sources);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                sources.push(path);
            }
        }
    }

    #[test]
    fn object_explorer_has_no_terminal_dependencies() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("object_explorer");
        let mut sources = Vec::new();
        rust_sources_below(&root, &mut sources);
        assert!(!sources.is_empty());

        for path in sources {
            let source = std::fs::read_to_string(&path).expect("Rust source is readable");
            for forbidden in ["ratatui::", "crossterm::"] {
                assert!(
                    !source.contains(forbidden),
                    "{} imports forbidden terminal dependency {forbidden}",
                    path.display()
                );
            }
        }
    }
}
