use bstr::BString;
use cloud_terrastodon_cache::ArtifactFilePath;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::ensure;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;
use tracing::debug;

/// Adjacent inputs owned by one subprocess attempt, independent of cache storage.
///
/// The guard keeps staged files alive until the caller finishes using them.
/// Files use the artifact store's portable relative-path contract; exclusive
/// creation rejects filesystem aliases instead of overwriting another input.
#[derive(Debug)]
pub struct CommandInputWorkspace {
    // The cleanup guard and canonical lookup must stay together. Exposing mutable
    // access would permit deleting or replacing files still used by a child.
    _directory: TempDir,
    paths: HashMap<PathBuf, PathBuf>,
}

impl CommandInputWorkspace {
    pub(crate) async fn stage(inputs: &HashMap<PathBuf, BString>) -> Result<Option<Self>> {
        if inputs.is_empty() {
            return Ok(None);
        }
        let inputs = inputs.clone();
        // The worker owns the directory even when its awaiting future is dropped.
        // Cleanup cannot race a still-running write through a borrowed guard.
        tokio::task::spawn_blocking(move || {
            let inputs = inputs
                .into_iter()
                .map(|(path, contents)| {
                    let portable = path
                        .to_str()
                        .ok_or_eyre("adjacent input paths must be Unicode")?;
                    let portable = if cfg!(windows) {
                        portable.replace('\\', "/")
                    } else {
                        portable.to_owned()
                    };
                    let validated = ArtifactFilePath::new(portable)
                        .wrap_err_with(|| format!("invalid adjacent input path {path:?}"))?;
                    Ok((path, validated, contents))
                })
                .collect::<Result<Vec<_>>>()?;
            let directory = tempfile::Builder::new()
                .prefix("cloud-terrastodon-command-")
                .tempdir()
                .wrap_err("creating isolated command input workspace")?;
            let root = directory
                .path()
                .canonicalize()
                .wrap_err("resolving command input workspace")?;
            let mut paths = HashMap::with_capacity(inputs.len());
            for (path, validated, contents) in inputs {
                let mut staged = root.clone();
                for component in validated.components() {
                    staged.push(component);
                }
                let parent = staged
                    .parent()
                    .ok_or_eyre("adjacent input has no workspace parent")?;
                std::fs::create_dir_all(parent)
                    .wrap_err_with(|| format!("creating adjacent input directory {parent:?}"))?;
                debug!("Writing arg {}", staged.display());
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&staged)
                    .wrap_err_with(|| format!("creating adjacent input {staged:?}"))?;
                file.write_all(&contents)
                    .wrap_err_with(|| format!("writing adjacent input {staged:?}"))?;
                drop(file);
                let canonical = staged
                    .canonicalize()
                    .wrap_err_with(|| format!("resolving adjacent input {staged:?}"))?;
                ensure!(
                    canonical.starts_with(&root),
                    "adjacent input escaped its command workspace"
                );
                paths.insert(path, canonical);
            }
            Ok(Some(Self {
                _directory: directory,
                paths,
            }))
        })
        .await
        .wrap_err("command input staging worker failed")?
    }

    pub(crate) fn input_path(&self, key: &Path) -> Result<&Path> {
        self.paths
            .get(key)
            .map(PathBuf::as_path)
            .ok_or_eyre("adjacent input was not staged by this command workspace")
    }
}

#[cfg(test)]
mod tests {
    use super::CommandInputWorkspace;
    use bstr::BString;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[tokio::test]
    async fn rejects_inputs_outside_the_attempt_workspace_without_changing_them() -> eyre::Result<()>
    {
        let fixture = tempfile::tempdir()?;
        let sentinel = fixture.path().join("sentinel.txt");
        std::fs::write(&sentinel, b"original")?;
        let traversal = PathBuf::from("..")
            .join(
                fixture
                    .path()
                    .file_name()
                    .expect("temporary directory name"),
            )
            .join("sentinel.txt");

        for invalid in [traversal, sentinel.clone()] {
            let inputs = HashMap::from([(invalid.clone(), BString::from("replacement"))]);
            assert!(
                CommandInputWorkspace::stage(&inputs).await.is_err(),
                "accepted input outside the workspace: {invalid:?}"
            );
            assert_eq!(std::fs::read(&sentinel)?, b"original");
        }
        Ok(())
    }
}
