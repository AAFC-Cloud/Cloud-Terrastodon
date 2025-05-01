use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::bstr::ByteSlice;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use eyre::Result;
use hcl::edit::Decorate;
use hcl::edit::structure::Body;
use std::path::Path;

pub async fn get_imports_from_existing(path: impl AsRef<Path>) -> Result<Vec<HCLImportBlock>> {
    let body = CommandBuilder::new(CommandKind::Terraform)
        .should_announce(true)
        .arg("show")
        .use_run_dir(path)
        .run_raw()
        .await?
        .stdout
        .to_str()?
        .parse::<Body>()?;

    let mut imports = Vec::<HCLImportBlock>::new();

    for block in body.into_blocks() {
        if block.ident.to_string() != "resource" {
            continue;
        }
        let provider: HCLProviderReference = block.clone().try_into()?;

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
        let to = ResourceBlockReference::Raw(to);
        imports.push(HCLImportBlock { provider, id, to })
    }

    Ok(imports)
}
