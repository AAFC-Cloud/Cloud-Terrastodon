use crate::IconCatalogue;
use crate::IconCatalogueEntry;
use crate::IconFamily;
use crate::OperationReport;
use crate::generated_file::GeneratedFile;
use crate::generated_file::is_generated_source;
use crate::generated_file::write_files;
use cloud_terrastodon_app::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

/// Render the complete generated surface from validated offline metadata.
/// No assets are read here, allowing updates to stage newly downloaded bytes.
pub fn planned_files(workspace: &Path, catalogue: &IconCatalogue) -> Result<Vec<GeneratedFile>> {
    catalogue.validate_metadata()?;
    let mut files = Vec::new();
    for family in IconFamily::ALL {
        let directory = workspace.join("crates").join(family.crate_directory());
        let mut entries = catalogue
            .icons
            .iter()
            .filter(|entry| entry.family == family)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.variant.cmp(&right.variant));
        for entry in &entries {
            files.push(GeneratedFile {
                path: directory
                    .join("src")
                    .join(format!("{}.rs", family.icon_module_name(&entry.variant))),
                contents: render_icon(workspace, entry)?.into_bytes(),
            });
        }
        files.push(GeneratedFile {
            path: directory.join("src/lib.rs"),
            contents: render_library(workspace, family, &entries, &directory.join("src/lib.rs"))?
                .into_bytes(),
        });
        if family == IconFamily::Azure {
            files.push(GeneratedFile {
                path: directory.join("src/azure_icon_for_resource_type.rs"),
                contents: render_resource_lookup(workspace, &entries)?.into_bytes(),
            });
        }
        let path = directory.join("Cargo.toml");
        let manifest =
            fs::read_to_string(&path).wrap_err_with(|| format!("reading {}", path.display()))?;
        files.push(GeneratedFile {
            path,
            contents: replace_features(&manifest, family, &entries)?.into_bytes(),
        });
        files.push(GeneratedFile {
            path: directory.join("ASSETS.md"),
            contents: render_provenance(family, &entries).into_bytes(),
        });
    }
    Ok(files)
}

/// Regenerate checked-in code and Cargo category features using local SVGs only.
pub fn generate(
    workspace: &Path,
    catalogue: &IconCatalogue,
    scope: &CancellationScope,
) -> Result<OperationReport> {
    catalogue.validate(workspace, scope)?;
    let files = planned_files(workspace, catalogue)?;
    let obsolete = obsolete_files(workspace, &files)?;
    let mut paths = files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    paths.extend(obsolete.iter().cloned());
    let changed_files = write_files(files, obsolete, scope)?;
    Ok(OperationReport {
        operation: "generate".into(),
        icon_count: catalogue.icons.len(),
        files: paths,
        changed_files,
    })
}

/// Verify local SVG integrity and report generated-file drift without writing.
pub fn check(
    workspace: &Path,
    catalogue: &IconCatalogue,
    scope: &CancellationScope,
) -> Result<OperationReport> {
    catalogue.validate(workspace, scope)?;
    let files = planned_files(workspace, catalogue)?;
    let obsolete = obsolete_files(workspace, &files)?;
    let mut changed_files = obsolete.clone();
    for file in &files {
        scope.bail_if_cancelled()?;
        match fs::read(&file.path) {
            Ok(existing) if existing == file.contents => {}
            Ok(_) => changed_files.push(file.path.clone()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                changed_files.push(file.path.clone())
            }
            Err(error) => {
                return Err(error).wrap_err_with(|| format!("reading {}", file.path.display()));
            }
        }
    }
    ensure!(
        changed_files.is_empty(),
        "generated iconography files are stale; run generate:\n{}",
        changed_files
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(OperationReport {
        operation: "check".into(),
        icon_count: catalogue.icons.len(),
        files: files
            .into_iter()
            .map(|file| file.path)
            .chain(obsolete)
            .collect(),
        changed_files,
    })
}

/// Find only obsolete generated Rust files directly inside the owned source directories.
/// Files without the exact generated marker, directories, and symlinks are preserved.
pub fn obsolete_files(workspace: &Path, planned: &[GeneratedFile]) -> Result<Vec<PathBuf>> {
    let root = fs::canonicalize(workspace).wrap_err("resolving iconography workspace")?;
    let paths = planned
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let mut obsolete = Vec::new();
    for family in IconFamily::ALL {
        let directory = workspace
            .join("crates")
            .join(family.crate_directory())
            .join("src");
        match fs::symlink_metadata(&directory) {
            Ok(metadata) => ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "icon source directory must be a regular directory: {}",
                directory.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).wrap_err_with(|| format!("examining {}", directory.display()));
            }
        }
        ensure!(
            fs::canonicalize(&directory)?.starts_with(&root),
            "icon source directory leaves workspace: {}",
            directory.display()
        );
        for entry in fs::read_dir(&directory)
            .wrap_err_with(|| format!("scanning {}", directory.display()))?
        {
            let entry = entry
                .wrap_err_with(|| format!("reading source entry in {}", directory.display()))?;
            let path = entry.path();
            if !entry.file_type()?.is_file()
                || path.extension().is_none_or(|extension| extension != "rs")
                || paths.contains(&path)
            {
                continue;
            }
            ensure!(
                path.parent() == Some(directory.as_path()),
                "generated source is not an immediate child: {}",
                path.display()
            );
            let contents = fs::read(&path)
                .wrap_err_with(|| format!("reading possible obsolete source {}", path.display()))?;
            if is_generated_source(&contents) {
                obsolete.push(path);
            }
        }
    }
    obsolete.sort();
    Ok(obsolete)
}

fn render_icon(workspace: &Path, entry: &IconCatalogueEntry) -> Result<String> {
    let type_name = format_ident!("{}", entry.family.icon_type_name(&entry.variant));
    let source = format!("../svg/{}", entry.file);
    let name = &entry.name;
    let category = &entry.category;
    let description = format!(
        " {} artwork. Requires the `{category}` category feature.",
        entry.name
    );
    let provenance = format!(
        " Source: <{}>. See the packaged ASSETS.md for artwork provenance and terms.",
        entry.source_url
    );
    render_source(
        workspace,
        quote! {
            use cloud_terrastodon_iconography::SvgIcon;
            use std::borrow::Cow;

            #[doc = #description]
            #[doc = ""]
            #[doc = #provenance]
            #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
            #[cfg_attr(feature = "registry", derive(facet::Facet, arbitrary::Arbitrary))]
            pub struct #type_name;

            impl SvgIcon for #type_name {
                fn svg(&self) -> Cow<'static, str> { Cow::Borrowed(include_str!(#source)) }
                fn name(&self) -> Cow<'static, str> { Cow::Borrowed(#name) }
                fn category(&self) -> Cow<'static, str> { Cow::Borrowed(#category) }
            }

            #[cfg(feature = "registry")]
            cloud_terrastodon_iconography::register_svg_icon!(#type_name);
        },
    )
}

fn render_library(
    workspace: &Path,
    family: IconFamily,
    entries: &[&IconCatalogueEntry],
    path: &Path,
) -> Result<String> {
    let mut sorted = entries.to_vec();
    sorted.sort_by_key(|entry| family.icon_module_name(&entry.variant));
    let declarations = sorted.iter().map(|entry| {
        let module = format_ident!("{}", family.icon_module_name(&entry.variant));
        let category = &entry.category;
        quote! { #[cfg(feature = #category)] mod #module; }
    });
    let exports = sorted.iter().map(|entry| {
        let module = format_ident!("{}", family.icon_module_name(&entry.variant));
        let type_name = format_ident!("{}", family.icon_type_name(&entry.variant));
        let category = &entry.category;
        quote! { #[cfg(feature = #category)] pub use #module::#type_name; }
    });
    let (lookup_module, lookup_export) = if family == IconFamily::Azure {
        (
            quote! { #[cfg(feature = "resource-types")] mod azure_icon_for_resource_type; },
            quote! { #[cfg(feature = "resource-types")] pub use azure_icon_for_resource_type::icon_for_resource_type; },
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };
    let tests = match fs::read_to_string(path) {
        Ok(source) => {
            let syntax = syn::parse_file(&source).wrap_err_with(|| {
                format!("parsing existing module declarations {}", path.display())
            })?;
            let generated = is_generated_source(source.as_bytes());
            ensure!(
                generated
                    || (syntax.shebang.is_none()
                        && syntax
                            .attrs
                            .iter()
                            .all(|attribute| attribute.path().is_ident("doc"))),
                "unmarked library has custom crate attributes; preserving {}",
                path.display()
            );
            let mut tests = Vec::new();
            for item in syntax.items {
                if matches!(&item, syn::Item::Mod(module) if test_only_module(module)) {
                    tests.push(item);
                } else {
                    ensure!(
                        generated || legacy_library_item(&item, family),
                        "unmarked library contains a custom production item; preserving {}. Only the legacy icon module and glob re-export can be migrated automatically",
                        path.display()
                    );
                }
            }
            tests
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            return Err(error).wrap_err_with(|| {
                format!("reading existing module declarations {}", path.display())
            });
        }
    };
    let introduction = match family {
        IconFamily::Azure => {
            " Embedded Azure Portal SVG icons selected through Cargo category features.\n\n No category is enabled by default. Each enabled icon is an independent unit type.\n Enable `registry` for reflected pure projections and offline arbitrary constructors.\n See the packaged ASSETS.md for artwork sources and terms."
        }
        IconFamily::AzureDevOps => {
            " Embedded Azure DevOps SVG icons selected through Cargo category features.\n\n No category is enabled by default. Each enabled icon is an independent unit type.\n Enable `registry` for reflected pure projections and offline arbitrary constructors.\n See the packaged ASSETS.md for artwork sources and terms."
        }
    };
    let documentation = introduction.lines().map(|line| quote! { #![doc = #line] });
    render_source(
        workspace,
        quote! {
            #(#documentation)*
            #lookup_module
            #(#declarations)*

            #lookup_export
            #(#exports)*

            #(#tests)*
        },
    )
}

fn test_only_module(module: &syn::ItemMod) -> bool {
    module.attrs.iter().any(|attribute| match &attribute.meta {
        syn::Meta::List(meta) if meta.path.is_ident("cfg") => meta
            .parse_args::<syn::Meta>()
            .is_ok_and(|condition| requires_test(&condition)),
        _ => false,
    })
}

fn requires_test(condition: &syn::Meta) -> bool {
    match condition {
        syn::Meta::Path(path) => path.is_ident("test"),
        syn::Meta::List(list) if list.path.is_ident("all") || list.path.is_ident("any") => {
            let Ok(conditions) = list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            ) else {
                return false;
            };
            if list.path.is_ident("all") {
                conditions.iter().any(requires_test)
            } else {
                !conditions.is_empty() && conditions.iter().all(requires_test)
            }
        }
        _ => false,
    }
}

fn legacy_library_item(item: &syn::Item, family: IconFamily) -> bool {
    let legacy = format!("{}_icon", family.module_prefix());
    match item {
        syn::Item::Mod(module) => {
            module.ident == legacy
                && module.content.is_none()
                && module.attrs.is_empty()
                && matches!(module.vis, syn::Visibility::Inherited)
        }
        syn::Item::Use(import) => {
            import.attrs.is_empty()
                && import.leading_colon.is_none()
                && matches!(import.vis, syn::Visibility::Public(_))
                && matches!(&import.tree, syn::UseTree::Path(path) if path.ident == legacy && matches!(&*path.tree, syn::UseTree::Glob(_)))
        }
        _ => false,
    }
}

fn render_resource_lookup(workspace: &Path, entries: &[&IconCatalogueEntry]) -> Result<String> {
    let imports = entries
        .iter()
        .filter(|entry| !entry.resource_types.is_empty())
        .map(|entry| {
            let type_name = format_ident!("{}", entry.family.icon_type_name(&entry.variant));
            let category = &entry.category;
            quote! { #[cfg(feature = #category)] use crate::#type_name; }
        });
    let arms = entries
        .iter()
        .filter(|entry| !entry.resource_types.is_empty())
        .map(|entry| {
            let type_name = format_ident!("{}", entry.family.icon_type_name(&entry.variant));
            let category = &entry.category;
            let comparisons = entry
                .resource_types
                .iter()
                .map(|alias| quote! { value.eq_ignore_ascii_case(#alias) })
                .reduce(|left, right| quote! { #left || #right })
                .expect("filtered nonempty aliases");
            quote! { #[cfg(feature = #category)] value if #comparisons => Some(&#type_name), }
        });
    render_source(
        workspace,
        quote! {
            #(#imports)*
            use cloud_terrastodon_azure_resource_types::ResourceType;
            use cloud_terrastodon_iconography::SvgIcon;

            /// Find an enabled icon for a resource type, including case-insensitive aliases.
            ///
            /// Unmapped types and categories disabled at compile time return `None`.
            pub fn icon_for_resource_type(resource_type: &ResourceType) -> Option<&'static dyn SvgIcon> {
                match resource_type.as_ref() { #(#arms)* _ => None, }
            }
        },
    )
}

fn render_source(workspace: &Path, tokens: TokenStream) -> Result<String> {
    let syntax = syn::parse2(tokens).wrap_err("parsing generated iconography source")?;
    let formatted = prettyplease::unparse(&syntax);
    format_rust(
        workspace,
        &format!("// @generated by iconography-cli generate; do not edit.\n\n{formatted}"),
    )
}

fn format_rust(workspace: &Path, source: &str) -> Result<String> {
    let mut input = NamedTempFile::new().wrap_err("creating temporary generated Rust input")?;
    input
        .write_all(source.as_bytes())
        .wrap_err("writing temporary generated Rust input")?;
    input
        .flush()
        .wrap_err("flushing temporary generated Rust input")?;
    let mut formatter = Command::new("rustfmt");
    formatter.args([
        "--quiet",
        "--emit",
        "stdout",
        "--edition",
        "2024",
        "--config",
        "style_edition=2024,skip_children=true",
    ]);
    let configuration = workspace.join("rustfmt.toml");
    if configuration.is_file() {
        formatter.arg("--config-path").arg(&configuration);
    }
    let output = formatter
        .arg(input.path())
        .output()
        .wrap_err("running rustfmt for generated icons; install the rustfmt toolchain component")?;
    ensure!(
        output.status.success(),
        "rustfmt failed while formatting generated icons ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).wrap_err("decoding rustfmt output as UTF-8")
}

fn replace_features(
    manifest: &str,
    family: IconFamily,
    entries: &[&IconCatalogueEntry],
) -> Result<String> {
    let categories = entries
        .iter()
        .map(|entry| entry.category.as_str())
        .collect::<BTreeSet<_>>();
    let mut features = String::from(
        "[features]\n# @generated by iconography-cli generate; categories come from catalogue.json.\ndefault = []\nall = [",
    );
    for (index, category) in categories.iter().enumerate() {
        if index > 0 {
            features.push_str(", ");
        }
        features.push('"');
        features.push_str(category);
        features.push('"');
    }
    features.push_str("]\n");
    for category in categories {
        features.push_str(&format!("{category} = []\n"));
    }
    features.push_str("registry = [\"cloud_terrastodon_iconography/registry\", \"dep:facet\", \"dep:arbitrary\", \"dep:eyre\", \"dep:linkme\"]\n");
    if family == IconFamily::Azure {
        features.push_str("resource-types = [\"dep:cloud_terrastodon_azure_resource_types\"]\n");
    }
    let normalized = manifest.replace("\r\n", "\n");
    let mut start = None;
    let mut end = normalized.len();
    let mut offset = 0;
    for line in normalized.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed == "[features]" {
            ensure!(
                start.is_none(),
                "manifest contains multiple [features] sections"
            );
            start = Some(offset);
        } else if start.is_some() && trimmed.starts_with('[') {
            end = offset;
            break;
        }
        offset += line.len();
    }
    match start {
        Some(start) => {
            if end < normalized.len() {
                features.push('\n');
            }
            Ok(format!(
                "{}{}{}",
                &normalized[..start],
                features,
                &normalized[end..]
            ))
        }
        None => Ok(format!("{}\n{}", normalized.trim_end(), features)),
    }
}

fn render_provenance(family: IconFamily, entries: &[&IconCatalogueEntry]) -> String {
    let mut output = String::from(
        "<!-- @generated by iconography-cli generate; do not edit. -->\n\n# Artwork provenance\n\nThe Cargo package's LiLiQ-R-1.1 license applies to its Rust code. Microsoft artwork remains subject to separate artwork terms; recording and packaging these assets does not grant artwork or trademark rights. Review the terms linked for each icon before use.\n\n",
    );
    match family {
        IconFamily::Azure => output.push_str("The SVGs are extracted from the Azure Portal by [maskati/azure-icons](https://github.com/maskati/azure-icons). These are Portal assets, not imports from the separate Azure Architecture icon collection. Original accepted SVG bytes are preserved.\n\n"),
        IconFamily::AzureDevOps => output.push_str("The SVGs are Microsoft Azure DevOps assets served by the public CDN. Original accepted SVG bytes are preserved. Assets whose discovery URL is their source URL are explicitly pinned; updates re-fetch that URL until a new discovery source is recorded.\n\n"),
    }
    for entry in entries {
        output.push_str(&format!(
            "## {} (`{}`)\n\n",
            markdown(&entry.name),
            markdown(&family.icon_type_name(&entry.variant))
        ));
        output.push_str(&format!("- Category feature: `{}`\n- Packaged file: `svg/{}`\n- Upstream logical name: `{}`\n- Source URL: <{}>\n- Discovery URL: <{}>\n- SHA-256: `{}`\n- Retrieved at (UTC): `{}`\n- Artwork terms: <{}>\n",
            markdown(&entry.category), markdown(&entry.file), markdown(&entry.logical_name), entry.source_url,
            entry.discovery_url, entry.sha256, entry.retrieved_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), entry.terms_url));
        if !entry.resource_types.is_empty() {
            let mut aliases = entry
                .resource_types
                .iter()
                .map(|alias| format!("`{}`", markdown(alias)))
                .collect::<Vec<_>>();
            aliases.sort();
            output.push_str(&format!(
                "- Azure resource-type aliases: {}\n",
                aliases.join(", ")
            ));
        }
        output.push('\n');
    }
    output.trim_end().to_owned() + "\n"
}

fn markdown(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::check;
    use super::generate;
    use super::planned_files;
    use super::replace_features;
    use crate::IconCatalogue;
    use crate::IconCatalogueEntry;
    use crate::IconFamily;
    use chrono::DateTime;
    use eyre::Result;
    use sha2::Digest;
    use sha2::Sha256;
    use std::fs;
    use teamy_cancellation::CancellationToken;
    use tempfile::TempDir;

    fn fixture() -> Result<(TempDir, IconCatalogue)> {
        let directory = tempfile::tempdir()?;
        for family in IconFamily::ALL {
            let crate_directory = directory
                .path()
                .join("crates")
                .join(family.crate_directory());
            fs::create_dir_all(crate_directory.join("svg"))?;
            fs::write(
                crate_directory.join("Cargo.toml"),
                "[package]\nname = \"fixture\"\n\n[features]\ndefault = []\n\n[dependencies.some_dependency]\npath = \"../dependency\"\n",
            )?;
        }
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><path fill="red"/></svg>"#;
        let mut icons = Vec::new();
        for variant in ["Subscription", "ResourceGroup"] {
            let file = format!("{}.svg", variant.to_ascii_lowercase());
            fs::write(
                directory
                    .path()
                    .join("crates/iconography_azure/svg")
                    .join(&file),
                svg,
            )?;
            icons.push(IconCatalogueEntry {
                family: IconFamily::Azure,
                variant: variant.into(),
                name: variant.into(),
                category: "resources".into(),
                file,
                source_url: "https://example.com/icon.svg".parse()?,
                logical_name: variant.into(),
                discovery_url: "https://example.com/manifest".parse()?,
                sha256: format!("{:x}", Sha256::digest(svg)),
                retrieved_at: DateTime::from_timestamp(0, 0).unwrap(),
                terms_url: "https://example.com/terms".parse()?,
                resource_types: Vec::new(),
            });
        }
        Ok((
            directory,
            IconCatalogue {
                schema_version: 1,
                icons,
            },
        ))
    }

    #[test]
    fn catalogue_order_does_not_change_generated_artifacts() -> Result<()> {
        let (directory, mut catalogue) = fixture()?;
        let first = planned_files(directory.path(), &catalogue)?;
        catalogue.icons.reverse();
        let reordered = planned_files(directory.path(), &catalogue)?;
        assert_eq!(first.len(), reordered.len());
        for (first, reordered) in first.iter().zip(&reordered) {
            assert_eq!(first.path, reordered.path);
            assert_eq!(first.contents, reordered.contents);
        }
        Ok(())
    }

    #[test]
    fn feature_updates_preserve_other_manifest_sections_and_are_idempotent() -> Result<()> {
        let (_, catalogue) = fixture()?;
        let entries = catalogue.icons.iter().collect::<Vec<_>>();
        let before = "[package]\nname = \"fixture\"\n\n[features]\nold = []\n\n[dependencies.some_dependency]\npath = \"../dependency\"\n";
        let after = replace_features(before, IconFamily::Azure, &entries)?;
        assert!(after.starts_with("[package]\nname = \"fixture\"\n\n"));
        assert!(after.ends_with("[dependencies.some_dependency]\npath = \"../dependency\"\n"));
        assert_eq!(
            after,
            replace_features(&after, IconFamily::Azure, &entries)?
        );
        Ok(())
    }

    #[test]
    fn repeated_generation_is_clean_and_check_detects_drift_without_repairing_it() -> Result<()> {
        let (directory, catalogue) = fixture()?;
        CancellationToken::new().with_scope(|scope| -> Result<()> {
            let first = generate(directory.path(), &catalogue, &scope)?;
            assert!(!first.changed_files.is_empty());
            assert!(
                generate(directory.path(), &catalogue, &scope)?
                    .changed_files
                    .is_empty()
            );
            assert!(check(directory.path(), &catalogue, &scope).is_ok());
            let path = directory
                .path()
                .join("crates/iconography_azure/src/azure_resource_group_icon.rs");
            fs::write(&path, "stale")?;
            assert!(
                check(directory.path(), &catalogue, &scope)
                    .unwrap_err()
                    .to_string()
                    .contains("stale")
            );
            assert_eq!(fs::read_to_string(path)?, "stale");
            Ok(())
        })
    }

    #[test]
    fn unmarked_libraries_with_custom_production_code_are_preserved_without_writes() -> Result<()> {
        let (directory, catalogue) = fixture()?;
        let source = directory.path().join("crates/iconography_azure/src");
        fs::create_dir_all(&source)?;
        let path = source.join("lib.rs");
        for original in [
            "mod custom;\n",
            "struct UserType;\n",
            "#[cfg(not(test))]\nmod custom;\n",
            "#[cfg(any(test, feature = \"production\"))]\nmod custom;\n",
        ] {
            fs::write(&path, original)?;
            CancellationToken::new().with_scope(|scope| -> Result<()> {
                assert!(
                    generate(directory.path(), &catalogue, &scope)
                        .unwrap_err()
                        .to_string()
                        .contains("unmarked library")
                );
                assert_eq!(fs::read_to_string(&path)?, original);
                assert!(!source.join("azure_resource_group_icon.rs").exists());
                Ok(())
            })?;
        }
        Ok(())
    }

    #[test]
    fn regeneration_removes_only_obsolete_generated_sources_and_preserves_test_modules()
    -> Result<()> {
        let (directory, mut catalogue) = fixture()?;
        let source = directory.path().join("crates/iconography_azure/src");
        fs::create_dir_all(&source)?;
        fs::write(source.join("lib.rs"), "#[cfg(test)]\nmod custom_tests;\n")?;
        fs::write(
            source.join("custom_tests.rs"),
            "// user-owned test module\n",
        )?;
        CancellationToken::new().with_scope(|scope| -> Result<()> {
            generate(directory.path(), &catalogue, &scope)?;
            let old_enum = source.join("azure_icon.rs");
            fs::write(&old_enum, "// @generated by iconography-cli generate; do not edit.\n\npub enum LegacyIcon {}\n")?;
            assert!(check(directory.path(), &catalogue, &scope).unwrap_err().to_string().contains("azure_icon.rs"));
            catalogue.icons.retain(|entry| entry.variant != "Subscription");
            let report = generate(directory.path(), &catalogue, &scope)?;
            assert!(report.changed_files.contains(&old_enum));
            assert!(!old_enum.exists());
            assert!(!source.join("azure_subscription_icon.rs").exists());
            assert_eq!(fs::read_to_string(source.join("custom_tests.rs"))?, "// user-owned test module\n");
            assert!(fs::read_to_string(source.join("lib.rs"))?.contains("mod custom_tests;"));
            assert!(check(directory.path(), &catalogue, &scope).is_ok());
            Ok(())
        })
    }
}
