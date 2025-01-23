use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceReference;
use eyre::Result;
use hcl::edit::structure::Body;
use hcl::edit::Decorate;
use std::path::Path;

pub async fn get_imports_from_existing(path: impl AsRef<Path>) -> Result<Vec<TofuImportBlock>> {
    let body = CommandBuilder::new(CommandKind::Tofu)
        .should_announce(true)
        .arg("show")
        .use_run_dir(path)
        .run_raw()
        .await?
        .stdout
        .parse::<Body>()?;

    let mut imports = Vec::<TofuImportBlock>::new();

    for block in body.into_blocks() {
        if block.ident.to_string() != "resource" {
            continue;
        }
        let provider: TofuProviderReference = block.clone().try_into()?;

        let id = block
            .body
            .get_attribute("id")
            .and_then(|x| x.value.as_str())
            .unwrap_or("\"unknown id\"")
            .to_owned();

        let to = block
            .decor()
            .prefix()
            .and_then(|x| x.trim().strip_prefix("# "))
            .and_then(|x| x.strip_suffix(":"))
            .unwrap_or("unknown")
            .to_owned();
        let to = TofuResourceReference::Raw(to);
        imports.push(TofuImportBlock { provider, id, to })
    }

    Ok(imports)
}
