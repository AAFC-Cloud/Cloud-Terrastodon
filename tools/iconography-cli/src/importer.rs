use crate::IconCatalogue;
use crate::IconFamily;
use crate::OperationReport;
use crate::generated_file::GeneratedFile;
use crate::generated_file::write_files;
use crate::generation;
use crate::public_asset_download::download;
use crate::source_discovery::resolve_sources;
use crate::validation::validate_svg;
use chrono::Utc;
use cloud_terrastodon_app::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use reqwest::Client;
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Refresh existing entries from one public source and regenerate local outputs.
///
/// Downloads and SVG checks complete before any checked-in input changes. The
/// final file replacements share one cancellation deferral with generated Rust,
/// Cargo features, and the provenance catalogue. Individual replacements are
/// atomic; filesystem failures can still interrupt a multi-file update. After
/// repairing such an error, rerun update for the same source to reconcile asset
/// bytes and catalogue hashes. Offline generation repairs output drift only
/// after the local assets and their recorded hashes agree.
///
/// This operates on public artwork sources without credentials. The catalogue
/// records terms references without asserting an artwork redistribution license.
pub async fn update(
    workspace: &Path,
    family: IconFamily,
    cancellation: &CancellationScope,
) -> Result<OperationReport> {
    let workspace = fs::canonicalize(workspace)
        .wrap_err_with(|| format!("Resolving iconography workspace {}", workspace.display()))?;
    let mut catalogue = IconCatalogue::read(&workspace)?;
    catalogue.validate_metadata()?;
    let selected: Vec<_> = catalogue
        .icons
        .iter()
        .filter(|entry| entry.family == family)
        .cloned()
        .collect();
    ensure!(
        !selected.is_empty(),
        "The catalogue has no {} icons to update",
        family.as_str()
    );
    let client = Client::builder()
        .user_agent(concat!(
            "cloud-terrastodon-iconography/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(30))
        .build()
        .wrap_err("Creating the public icon asset downloader")?;
    cancellation.bail_if_cancelled()?;
    let sources = resolve_sources(family, &selected, &client, cancellation).await?;
    let mut files = Vec::new();
    let retrieved_at = Utc::now();

    for entry in &mut catalogue.icons {
        cancellation.bail_if_cancelled()?;
        let path = workspace
            .join("crates")
            .join(entry.family.crate_directory())
            .join("svg")
            .join(&entry.file);
        if entry.family == family {
            let source_url = sources.get(&entry.logical_name).ok_or_else(|| {
                eyre::eyre!(
                    "Discovery did not resolve {} icon {}",
                    family.as_str(),
                    entry.logical_name
                )
            })?;
            let bytes = download(&client, source_url, cancellation).await?;
            validate_svg(&bytes, &entry.logical_name)?;
            let digest = format!("{:x}", Sha256::digest(&bytes));
            // A repeated update with identical source bytes and identity is a
            // no-op, including its provenance timestamp.
            if entry.source_url != *source_url || entry.sha256 != digest {
                entry.source_url = source_url.clone();
                entry.sha256 = digest;
                entry.retrieved_at = retrieved_at;
            }
            files.push(GeneratedFile {
                path,
                contents: bytes,
            });
        } else {
            // Unselected local inputs must remain sound before their generated
            // API is included in the same publication unit.
            let bytes = fs::read(&path)
                .wrap_err_with(|| format!("Reading unselected icon {}", path.display()))?;
            validate_svg(&bytes, &entry.logical_name)?;
            ensure!(
                format!("{:x}", Sha256::digest(&bytes)) == entry.sha256,
                "Unselected icon {} differs from its recorded SHA-256; update its source first",
                path.display()
            );
        }
    }
    files.extend(generation::planned_files(&workspace, &catalogue)?);
    let mut contents =
        facet_json::to_vec_pretty(&catalogue).wrap_err("Serializing the updated icon catalogue")?;
    contents.push(b'\n');
    files.push(GeneratedFile {
        path: workspace.join("tools/iconography-cli/catalogue.json"),
        contents,
    });
    let obsolete = generation::obsolete_files(&workspace, &files)?;
    let mut paths = files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    paths.extend(obsolete.iter().cloned());
    let changed_files = write_files(files, obsolete, cancellation)?;
    Ok(OperationReport {
        operation: "update".into(),
        icon_count: selected.len(),
        files: paths,
        changed_files,
    })
}
