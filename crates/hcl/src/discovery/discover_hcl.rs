use crate::discovery::try_read_hcl_file;
use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::task::JoinSet;
use tracing::debug;

#[derive(Debug, Clone, Copy)]
pub enum DiscoveryDepth {
    Shallow,
    Recursive,
}

#[derive(Debug)]
enum ActionResponse {
    DiscoveredChildFile(PathBuf),
    DiscoverDirChildren(PathBuf),
    DiscoveredBody(PathBuf, Body),
    None,
}

pub async fn discover_hcl(
    directory: impl AsRef<Path>,
    depth: DiscoveryDepth,
) -> eyre::Result<HashMap<PathBuf, Body>> {
    let mut join_set: JoinSet<eyre::Result<ActionResponse>> = JoinSet::new();
    let dir = directory.as_ref().to_path_buf();
    join_set.spawn(async move { Ok(ActionResponse::DiscoverDirChildren(dir)) });
    let mut hcl_bodies = HashMap::new();
    while let Some(res) = join_set.join_next().await {
        let res = res??;
        debug!(?res, "Handling discovery action response");
        match res {
            ActionResponse::None => {}
            ActionResponse::DiscoveredChildFile(path) => {
                join_set.spawn(async move {
                    if let Some(body) = try_read_hcl_file(&path).await? {
                        Ok(ActionResponse::DiscoveredBody(path, body))
                    } else {
                        Ok(ActionResponse::None)
                    }
                });
            }
            ActionResponse::DiscoveredBody(path, body) => {
                hcl_bodies.insert(path, body);
            }
            ActionResponse::DiscoverDirChildren(dir) => {
                let mut entries = match tokio::fs::read_dir(&dir).await {
                    Ok(entries) => entries,
                    Err(_) => continue,
                };

                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    let metadata = entry.metadata().await?;

                    if metadata.is_file() {
                        join_set
                            .spawn(async move { Ok(ActionResponse::DiscoveredChildFile(path)) });
                    } else if metadata.is_dir() && matches!(depth, DiscoveryDepth::Recursive) {
                        let path_clone = path.clone();
                        join_set.spawn(async move {
                            Ok(ActionResponse::DiscoverDirChildren(path_clone))
                        });
                    }
                }
            }
        }
    }
    Ok(hcl_bodies)
}

#[cfg(test)]
mod test {
    use crate::discovery::DiscoveryDepth;
    use crate::discovery::discover_hcl;
    use std::path::PathBuf;
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    fn init_logging() -> eyre::Result<()> {
        let env_filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy();
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_file(true)
            .with_line_number(true)
            .without_time();
        if let Err(error) = subscriber.try_init() {
            eprintln!(
                "Failed to initialize tracing subscriber - are you running `cargo test`? If so, multiple test entrypoints may be running from the same process. https://github.com/tokio-rs/console/issues/505 : {error}"
            );
            return Ok(());
        }
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        init_logging()?;

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PathBuf::from_iter([
            "..",
            "..",
            "test_data",
            "terraform_discovery",
        ]));
        assert!(dir.exists());

        let discovered_flat = discover_hcl(&dir, DiscoveryDepth::Shallow).await?;
        assert_eq!(discovered_flat.len(), 2);

        let discovered_recursive = discover_hcl(&dir, DiscoveryDepth::Recursive).await?;
        assert_eq!(discovered_recursive.len(), 6);
        Ok(())
    }
}
