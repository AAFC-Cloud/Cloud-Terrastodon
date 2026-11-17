use crate::IconCatalogueEntry;
use crate::validation;
use cloud_terrastodon_app::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use std::fs;
use std::path::Path;

/// The checked-in input contract for offline generation and explicit asset updates.
#[derive(Clone, Debug, Facet)]
#[facet(deny_unknown_fields)]
pub struct IconCatalogue {
    pub schema_version: u32,
    pub icons: Vec<IconCatalogueEntry>,
}

impl IconCatalogue {
    pub fn read(workspace: &Path) -> Result<Self> {
        let path = workspace.join("tools/iconography-cli/catalogue.json");
        let text = fs::read_to_string(&path)
            .wrap_err_with(|| format!("reading icon catalogue {}", path.display()))?;
        facet_json::from_str(&text)
            .wrap_err_with(|| format!("decoding icon catalogue {}", path.display()))
    }

    pub fn validate(&self, workspace: &Path, scope: &CancellationScope) -> Result<()> {
        self.validate_metadata()?;
        for entry in &self.icons {
            scope.bail_if_cancelled()?;
            let path = workspace
                .join("crates")
                .join(entry.family.crate_directory())
                .join("svg")
                .join(&entry.file);
            let bytes =
                fs::read(&path).wrap_err_with(|| format!("reading SVG {}", path.display()))?;
            scope.bail_if_cancelled()?;
            validation::validate_asset(entry, &bytes)
                .wrap_err_with(|| format!("validating {}", path.display()))?;
        }
        Ok(())
    }

    pub fn validate_metadata(&self) -> Result<()> {
        validation::validate_catalogue(self)
    }
}
