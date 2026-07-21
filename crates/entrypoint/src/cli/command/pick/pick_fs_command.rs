use crate::cli::pick::pick_command::PickCommonArgs;
use crate::cli::pick::pick_command::write_selected_lines;
use crate::serde_json_isolation::Value;
use crate::serde_json_isolation::json;
use cloud_terrastodon_user_input::CandidateSink;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::collections::VecDeque;
use std::path::Path;
use tracing::Instrument;
use tracing::instrument;
use tracing::trace_span;

#[derive(Debug, Clone)]
struct FsPickEntry {
    path: String,
    value: Value,
}

/// Pick from the current working directory.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct PickFsArgs {
    /// Recursively read child directories
    #[facet(figue::named)]
    pub recursive: bool,
}

#[instrument(
    name = "pick_fs_visit_entries",
    skip_all,
    fields(recursive = recursive),
)]
async fn visit_fs_entries(
    root: &Path,
    recursive: bool,
    mut visit: impl FnMut(Vec<FsPickEntry>) -> Result<()>,
) -> Result<()> {
    let mut directories = VecDeque::from([(root.to_path_buf(), 0)]);
    while let Some((dir, depth)) = directories.pop_front() {
        let dir_entries = async {
            let mut read_dir = tokio::fs::read_dir(&dir).await?;
            let mut dir_entries = Vec::new();
            while let Some(entry) = read_dir.next_entry().await? {
                dir_entries.push(entry);
            }
            dir_entries.sort_by_key(|entry| entry.path());
            Ok::<_, std::io::Error>(dir_entries)
        }
        .instrument(trace_span!("pick_fs_read_directory"))
        .await?;

        let mut batch = Vec::with_capacity(dir_entries.len());
        for entry in dir_entries {
            let file_type = entry.file_type().await?;
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

            if recursive && file_type.is_dir() && !file_type.is_symlink() {
                directories.push_back((path.clone(), depth + 1));
            }

            batch.push(FsPickEntry {
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
        }

        if !batch.is_empty() {
            visit(batch)?;
        }
    }

    Ok(())
}

impl PickFsArgs {
    #[instrument(name = "pick_fs_command", skip_all)]
    pub(crate) async fn invoke(self, common: PickCommonArgs) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let recursive = self.recursive;
        let query = common.query.clone();
        let query_engine = common.query_engine.clone();
        let empty_choices = Vec::<Choice<String>>::new();

        let rtn = PickerTui::<_>::new()
            .set_auto_accept(common.auto_accept)
            .set_query(common.default_query.unwrap_or_default())
            .add_event_handler(move |event, sink| {
                let cwd = cwd.clone();
                let query = query.clone();
                let query_engine = query_engine.clone();
                async move {
                    if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                        stream_fs_choices(&cwd, recursive, &query_engine, &query, &sink).await?;
                    }
                    Ok(())
                }
            })
            .pick_inner(!common.single, empty_choices)
            .await?;

        write_selected_lines(&rtn)?;
        Ok(())
    }
}

#[instrument(
    name = "pick_fs_stream",
    skip_all,
    fields(recursive = recursive),
)]
async fn stream_fs_choices(
    root: &Path,
    recursive: bool,
    query_engine: &crate::cli::pick::pick_command::QueryEngine,
    query: &str,
    sink: &CandidateSink<String>,
) -> Result<()> {
    visit_fs_entries(root, recursive, |entries| {
        trace_span!("pick_fs_build_choice_batch").in_scope(|| {
            let mut choices = Vec::with_capacity(entries.len());
            for entry in entries {
                choices.push(Choice {
                    key: query_engine.query(&entry.value, query)?,
                    value: entry.path,
                });
            }
            sink.push(choices)?;
            Ok(())
        })
    })
    .await
}

#[cfg(test)]
mod test {
    use crate::cli::pick::pick_fs_command::visit_fs_entries;
    use std::fs;
    use std::path::MAIN_SEPARATOR;
    use tempfile::tempdir;

    #[tokio::test]
    async fn visits_fs_entries_breadth_first() -> eyre::Result<()> {
        let dir = tempdir()?;
        let nested_file_path = format!("nested{MAIN_SEPARATOR}beta.txt");
        fs::write(dir.path().join("alpha.txt"), "alpha")?;
        fs::create_dir(dir.path().join("nested"))?;
        fs::write(dir.path().join("nested").join("beta.txt"), "beta")?;

        let mut top_level = Vec::new();
        visit_fs_entries(dir.path(), false, |batch| {
            top_level.extend(batch);
            Ok(())
        })
        .await?;
        let mut recursive = Vec::new();
        visit_fs_entries(dir.path(), true, |batch| {
            recursive.extend(batch);
            Ok(())
        })
        .await?;

        let top_level_paths: Vec<_> = top_level.iter().map(|entry| entry.path.as_str()).collect();
        assert_eq!(top_level_paths, vec!["alpha.txt", "nested"]);

        let recursive_paths: Vec<_> = recursive.iter().map(|entry| entry.path.as_str()).collect();
        assert_eq!(
            recursive_paths,
            vec!["alpha.txt", "nested", nested_file_path.as_str()]
        );

        let nested_dir = recursive
            .iter()
            .find(|entry| entry.path == "nested")
            .unwrap();
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
