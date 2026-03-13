use crate::cli::pick::pick_command::PickCommonArgs;
use crate::cli::pick::pick_command::write_selected_lines;
use clap::Args;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use serde_json::Value;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Clone)]
struct FsPickEntry {
    path: String,
    value: Value,
}

/// Pick from the current working directory.
#[derive(Args, Debug, Clone, Default)]
pub struct PickFsArgs {
    /// Recursively read child directories
    #[clap(long)]
    pub recursive: bool,
}

fn default_fs_query(query: &str) -> &str {
    if query == "*" {
        "path"
    } else {
        query
    }
}

fn collect_fs_entries(root: &Path, recursive: bool) -> Result<Vec<FsPickEntry>> {
    let mut entries = Vec::new();
    collect_fs_entries_inner(root, root, recursive, 0, &mut entries)?;
    Ok(entries)
}

fn collect_fs_entries_inner(
    root: &Path,
    dir: &Path,
    recursive: bool,
    depth: usize,
    entries: &mut Vec<FsPickEntry>,
) -> Result<()> {
    let mut dir_entries = std::fs::read_dir(dir)?.collect::<std::result::Result<Vec<_>, _>>()?;
    dir_entries.sort_by_key(|entry| entry.path());

    for entry in dir_entries {
        let file_type = entry.file_type()?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .to_string();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let entry_type = if file_type.is_dir() {
            "dir"
        } else if file_type.is_file() {
            "file"
        } else if file_type.is_symlink() {
            "symlink"
        } else {
            "other"
        };

        entries.push(FsPickEntry {
            path: relative_path.clone(),
            value: json!({
                "path": relative_path,
                "absolute_path": path.to_string_lossy(),
                "name": file_name,
                "stem": path.file_stem().map(|part| part.to_string_lossy().to_string()),
                "extension": path.extension().map(|part| part.to_string_lossy().to_string()),
                "depth": depth,
                "type": entry_type,
                "is_dir": file_type.is_dir(),
                "is_file": file_type.is_file(),
                "is_symlink": file_type.is_symlink(),
            }),
        });

        if recursive && file_type.is_dir() && !file_type.is_symlink() {
            collect_fs_entries_inner(root, &path, recursive, depth + 1, entries)?;
        }
    }

    Ok(())
}

impl PickFsArgs {
    pub(crate) async fn invoke(self, common: PickCommonArgs) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let entries = collect_fs_entries(&cwd, self.recursive)?;
        let query = default_fs_query(&common.query);

        let mut choices = Vec::with_capacity(entries.len());
        for entry in &entries {
            let key = common.query_engine.query(&entry.value, query)?;
            choices.push(Choice {
                key,
                value: entry.path.clone(),
            });
        }

        let rtn = PickerTui::new()
            .set_auto_accept(common.auto_accept)
            .set_query(common.default_query.unwrap_or_default())
            .pick_inner(common.many, choices)?;

        write_selected_lines(&rtn)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cli::pick::pick_fs_command::collect_fs_entries;
    use crate::cli::pick::pick_fs_command::default_fs_query;
    use std::fs;
    use std::path::MAIN_SEPARATOR;
    use tempfile::tempdir;

    #[test]
    fn uses_path_as_default_fs_query() {
        assert_eq!(default_fs_query("*"), "path");
        assert_eq!(default_fs_query("name"), "name");
    }

    #[test]
    fn collects_fs_entries_recursively() -> eyre::Result<()> {
        let dir = tempdir()?;
        let nested_file_path = format!("nested{MAIN_SEPARATOR}beta.txt");
        fs::write(dir.path().join("alpha.txt"), "alpha")?;
        fs::create_dir(dir.path().join("nested"))?;
        fs::write(dir.path().join("nested").join("beta.txt"), "beta")?;

        let top_level = collect_fs_entries(dir.path(), false)?;
        let recursive = collect_fs_entries(dir.path(), true)?;

        let top_level_paths: Vec<_> = top_level.iter().map(|entry| entry.path.as_str()).collect();
        assert_eq!(top_level_paths, vec!["alpha.txt", "nested"]);

        let recursive_paths: Vec<_> = recursive.iter().map(|entry| entry.path.as_str()).collect();
        assert_eq!(recursive_paths, vec!["alpha.txt", "nested", nested_file_path.as_str()]);

        let nested_dir = recursive.iter().find(|entry| entry.path == "nested").unwrap();
        assert_eq!(nested_dir.value["type"], "dir");
        assert_eq!(nested_dir.value["depth"], 0);

        let nested_file = recursive
            .iter()
            .find(|entry| entry.path == nested_file_path)
            .unwrap();
        assert_eq!(nested_file.value["name"], "beta.txt");
        assert_eq!(nested_file.value["extension"], "txt");
        assert_eq!(nested_file.value["depth"], 1);

        Ok(())
    }
}
