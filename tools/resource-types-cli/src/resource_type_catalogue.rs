use crate::ResourceTypeGenerationReport;
use cloud_terrastodon_app::CancellationScope;
use cloud_terrastodon_azure_types::AzureResourceProvider;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;
use quote::quote;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use syn::Ident;
use tempfile::NamedTempFile;

/// Validated input for the generated resource-type enum.
///
/// Names retain the supplied service spelling. Exact duplicates are removed and
/// output order is deterministic. Conflicting case-insensitive names, invalid
/// generated identifiers, and identifier collisions are errors, not renamings.
/// An empty catalogue is rejected to avoid accidentally deleting every known type.
pub struct ResourceTypeCatalogue {
    // Private so rendering cannot bypass the validation and ordering contract.
    names: Vec<String>,
}

impl ResourceTypeCatalogue {
    /// The maintenance catalogue beside this tool's Cargo manifest.
    ///
    /// This unpublished tool uses its build checkout, regardless of the process's
    /// working directory. Callers can supply another path explicitly.
    pub fn default_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("catalogue.json")
    }

    /// The generated enum in the reusable crate within this tool's build checkout.
    /// Resolving this path does not read or create files.
    pub fn default_output_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/azure_resource_types/src/resource_types.rs")
    }

    /// Build generation input from SDK metadata, retaining the service's spelling.
    /// Missing names or inventories are errors rather than silent omissions from
    /// the generated enum. Do not round-trip through `ResourceType`: known enum
    /// variants would replace new service spelling with the old catalogue spelling.
    pub fn from_providers(
        providers: impl IntoIterator<Item = impl Borrow<AzureResourceProvider>>,
        cancellation: &CancellationScope,
    ) -> Result<Self> {
        let mut names = Vec::new();
        for provider in providers {
            let provider = provider.borrow();
            cancellation.bail_if_cancelled()?;
            let namespace = provider
                .namespace
                .as_ref()
                .ok_or_else(|| eyre!("provider metadata omitted a provider namespace"))?;
            for resource_type in provider
                .resource_types
                .as_ref()
                .ok_or_else(|| eyre!("provider metadata omitted a provider's resource-type list"))?
            {
                names.push(format!(
                    "{namespace}/{}",
                    resource_type
                        .resource_type
                        .as_ref()
                        .ok_or_else(|| eyre!("provider metadata omitted a resource-type name"))?
                ));
            }
        }
        Self::new(names)
    }

    pub fn new(names: Vec<String>) -> Result<Self> {
        ensure!(
            !names.is_empty(),
            "resource-type catalogue must not be empty"
        );
        let mut canonical = BTreeMap::new();
        let mut variants = BTreeMap::new();
        for name in names {
            let (namespace, resource_type) = name.split_once('/').ok_or_else(|| {
                eyre!("resource type must contain its provider namespace: {name:?}")
            })?;
            ensure!(
                !namespace.is_empty() && !resource_type.split('/').any(str::is_empty),
                "resource type contains an empty namespace or path segment: {name:?}"
            );
            let variant = Self::variant(&name)?;
            let variant_name = variant.to_string();
            if let Some(existing) = variants.insert(variant_name.clone(), name.clone()) {
                ensure!(
                    existing == name,
                    "resource types {existing:?} and {name:?} generate the same Rust variant {variant_name}"
                );
            }
            if let Some(existing) = canonical.insert(name.to_lowercase(), name.clone()) {
                ensure!(
                    existing == name,
                    "resource-type catalogue has conflicting case-insensitive spellings: {existing:?} and {name:?}"
                );
            }
        }
        Ok(Self {
            names: canonical.into_values().collect(),
        })
    }

    /// Read only the supplied JSON array; this operation never queries Azure.
    pub fn read(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)
            .wrap_err_with(|| format!("reading resource-type catalogue {}", path.display()))?;
        let names = facet_json::from_str::<Vec<String>>(&json)
            .wrap_err_with(|| format!("decoding resource-type catalogue {}", path.display()))?;
        Self::new(names)
    }

    /// Save only the validated, normalized resource-type names as a JSON array.
    /// This is the offline input consumed by subsequent generation and checks.
    pub fn save(
        &self,
        output: &Path,
        cancellation: &CancellationScope,
    ) -> Result<ResourceTypeGenerationReport> {
        let json = facet_json::to_string_pretty(&self.names)
            .wrap_err("encoding the resource-type catalogue")?;
        cancellation.bail_if_cancelled()?;
        self.publish(output, &format!("{json}\n"), cancellation)
    }

    fn variant(name: &str) -> Result<Ident> {
        let variant = name
            .replace('.', "_DOT_")
            .replace('/', "_SLASH_")
            .to_uppercase();
        syn::parse_str(&variant).wrap_err_with(|| {
            format!("resource type {name:?} does not produce a valid Rust variant: {variant:?}")
        })
    }

    /// Render the existing public enum contract without consulting service state.
    ///
    /// Known types parse case-insensitively and display the catalogue spelling.
    /// Unknown values retain their input spelling in `Other(String)` in all builds.
    pub fn render(&self) -> Result<String> {
        let variants = self
            .names
            .iter()
            .map(|name| Self::variant(name))
            .collect::<Result<Vec<_>>>()?;
        let lowercase = self.names.iter().map(|name| name.to_lowercase());
        let names = &self.names;
        let source = quote! {
            use arbitrary::Arbitrary;
            use facet::Facet;

            #[rustfmt::skip]
            #[derive(Debug, Clone, Eq, PartialEq, Hash, Facet, Arbitrary)]
            #[facet(proxy = String)]
            #[repr(C)]
            #[allow(non_camel_case_types)]
            pub enum ResourceType {
                #(#variants,)*
                #[arbitrary(skip)]
                Other(String),
            }

            #[rustfmt::skip]
            impl std::str::FromStr for ResourceType {
                type Err = std::convert::Infallible;
                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value.to_lowercase().as_str() {
                        #(#lowercase => return Ok(ResourceType::#variants),)*
                        _ => {}
                    }
                    Ok(ResourceType::Other(value.to_string()))
                }
            }

            #[rustfmt::skip]
            impl AsRef<str> for ResourceType {
                fn as_ref(&self) -> &str {
                    match self {
                        #(ResourceType::#variants => #names,)*
                        ResourceType::Other(value) => value.as_str(),
                    }
                }
            }
        };
        let syntax = syn::parse2(source).wrap_err("parsing generated resource-type source")?;
        Ok(format!(
            "// @generated by resource-types generate; do not edit.\n// Generator: tools/resource-types-cli\n{}",
            prettyplease::unparse(&syntax)
        ))
    }

    /// Replace the selected output atomically after all generation succeeds.
    /// Existing identical contents are left untouched.
    pub fn generate(
        &self,
        output: &Path,
        cancellation: &CancellationScope,
    ) -> Result<ResourceTypeGenerationReport> {
        let source = self.render()?;
        cancellation.bail_if_cancelled()?;
        self.publish(output, &source, cancellation)
    }

    fn publish(
        &self,
        output: &Path,
        source: &str,
        cancellation: &CancellationScope,
    ) -> Result<ResourceTypeGenerationReport> {
        // Validate the presentation boundary before any output mutation. A
        // successful write must not later become an error while emitting JSON.
        let mut report = ResourceTypeGenerationReport::new(self.names.len(), output, false)?;
        let changed = !Self::matches(output, source)?;
        if changed {
            let parent = output
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let mut temporary = NamedTempFile::new_in(parent).wrap_err_with(|| {
                format!("staging resource-type output beside {}", output.display())
            })?;
            temporary
                .write_all(source.as_bytes())
                .wrap_err("writing staged resource-type output")?;
            temporary
                .flush()
                .wrap_err("flushing staged resource-type output")?;
            cancellation.with_deferral("publishing resource-type output", |_| {
                temporary.persist(output).wrap_err_with(|| {
                    format!("publishing resource-type output {}", output.display())
                })?;
                eyre::Ok(())
            })?;
        }
        report.changed = changed;
        Ok(report)
    }

    /// Fail on missing or stale generated output without changing any file.
    pub fn check(
        &self,
        output: &Path,
        cancellation: &CancellationScope,
    ) -> Result<ResourceTypeGenerationReport> {
        let source = self.render()?;
        cancellation.bail_if_cancelled()?;
        ensure!(
            Self::matches(output, &source)?,
            "generated resource types are stale or missing: {}; run resource-types generate with this catalogue and output",
            output.display()
        );
        ResourceTypeGenerationReport::new(self.names.len(), output, false)
    }

    fn matches(output: &Path, source: &str) -> Result<bool> {
        match fs::read(output) {
            Ok(existing) => Ok(existing == source.as_bytes()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error)
                .wrap_err_with(|| format!("reading resource-type output {}", output.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceTypeCatalogue;
    use cloud_terrastodon_app::CliOutput;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_azure_types::AzureResourceProvider;
    use cloud_terrastodon_azure_types::AzureResourceProviderResourceType;
    use eyre::Result;
    use std::fs;
    use teamy_cancellation::CancellationToken;

    #[test]
    fn sdk_metadata_preserves_new_types_and_service_spelling() -> Result<()> {
        let providers = vec![AzureResourceProvider {
            namespace: Some("microsoft.compute".parse()?),
            resource_types: Some(vec![
                AzureResourceProviderResourceType {
                    resource_type: Some("virtualmachines".parse()?),
                    ..Default::default()
                },
                AzureResourceProviderResourceType {
                    resource_type: Some("syntheticNewKind/children".parse()?),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }];
        CancellationToken::new().with_scope(|scope| -> Result<()> {
            let catalogue = ResourceTypeCatalogue::from_providers(providers, &scope)?;
            assert_eq!(
                catalogue.names,
                [
                    "microsoft.compute/syntheticNewKind/children",
                    "microsoft.compute/virtualmachines"
                ]
            );
            Ok(())
        })
    }

    #[test]
    fn incomplete_sdk_metadata_cannot_silently_shrink_the_catalogue() -> Result<()> {
        for provider in [
            AzureResourceProvider::default(),
            AzureResourceProvider {
                namespace: Some("Example.Compute".parse()?),
                ..Default::default()
            },
            AzureResourceProvider {
                namespace: Some("Example.Compute".parse()?),
                resource_types: Some(vec![AzureResourceProviderResourceType::default()]),
                ..Default::default()
            },
        ] {
            CancellationToken::new().with_scope(|scope| -> Result<()> {
                assert!(ResourceTypeCatalogue::from_providers(vec![provider], &scope).is_err());
                Ok(())
            })?;
        }
        Ok(())
    }

    #[test]
    fn input_order_and_exact_duplicates_do_not_change_generation() -> Result<()> {
        let catalogue = ResourceTypeCatalogue::new(vec![
            "Example.Compute/servers".into(),
            "Example.Compute/servers/extensions".into(),
        ])?;
        let reordered = ResourceTypeCatalogue::new(vec![
            "Example.Compute/servers/extensions".into(),
            "Example.Compute/servers".into(),
            "Example.Compute/servers/extensions".into(),
        ])?;
        assert_eq!(catalogue.render()?, reordered.render()?);
        Ok(())
    }

    #[test]
    fn ambiguous_spellings_and_variant_collisions_are_errors() {
        for names in [
            vec!["Example.Compute/servers", "example.compute/servers"],
            vec!["Example.Compute/servers", "Example_DOT_Compute/servers"],
            vec!["Example/servers/children", "Example/servers_SLASH_children"],
        ] {
            assert!(
                ResourceTypeCatalogue::new(names.into_iter().map(str::to_owned).collect()).is_err()
            );
        }
    }

    #[test]
    fn invalid_catalogues_fail_before_rendering() {
        assert!(ResourceTypeCatalogue::new(Vec::new()).is_err());
        for name in [
            "Example",
            "/servers",
            "Example/",
            "Example/servers//children",
            "Example/invalid-name",
            "1Example/servers",
        ] {
            assert!(
                ResourceTypeCatalogue::new(vec![name.into()]).is_err(),
                "{name}"
            );
        }
    }

    #[test]
    fn repeated_generation_and_check_preserve_files_until_an_explicit_refresh() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let output = directory.path().join("resource_types.rs");
        let catalogue = ResourceTypeCatalogue::new(vec!["Example.Compute/servers".into()])?;
        CancellationToken::new().with_scope(|scope| -> Result<()> {
            assert!(catalogue.check(&output, &scope).is_err());
            assert!(!output.exists());
            let first = catalogue.generate(&output, &scope)?;
            assert!(first.changed);
            // Default redirected output is JSON. A committed source file must
            // have a report the CLI can actually emit before returning success.
            assert!(
                CliOutput::facet(first)
                    .render(Some(OutputFormat::Json), false, &scope)?
                    .is_some()
            );
            assert!(!catalogue.generate(&output, &scope)?.changed);
            assert!(!catalogue.check(&output, &scope)?.changed);
            fs::write(&output, "stale")?;
            assert!(catalogue.check(&output, &scope).is_err());
            assert_eq!(fs::read_to_string(&output)?, "stale");
            assert!(catalogue.generate(&output, &scope)?.changed);
            assert_eq!(fs::read_to_string(&output)?, catalogue.render()?);
            Ok(())
        })
    }

    #[test]
    fn saved_catalogue_is_deterministic_input_for_offline_generation() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let input = directory.path().join("catalogue.json");
        let output = directory.path().join("resource_types.rs");
        let catalogue = ResourceTypeCatalogue::new(vec![
            "Example.Compute/servers/extensions".into(),
            "Example.Compute/servers".into(),
            "Example.Compute/servers".into(),
        ])?;
        CancellationToken::new().with_scope(|scope| -> Result<()> {
            assert!(catalogue.save(&input, &scope)?.changed);
            assert!(!catalogue.save(&input, &scope)?.changed);
            let read = ResourceTypeCatalogue::read(&input)?;
            assert_eq!(catalogue.render()?, read.render()?);
            assert_eq!(read.generate(&output, &scope)?.resource_type_count, 2);
            assert_eq!(fs::read_to_string(&output)?, catalogue.render()?);
            Ok(())
        })
    }
}
