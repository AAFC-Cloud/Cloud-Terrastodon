use anyhow::Result;
use command::prelude::CommandBuilder;
use hcl::edit::structure::Body;
use hcl::edit::Decorate;
use std::path::Path;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuResourceReference;

pub async fn get_imports_from_existing(path: impl AsRef<Path>) -> Result<Vec<TofuImportBlock>> {
    let body = CommandBuilder::new(command::prelude::CommandKind::Tofu)
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
        let id = block
            .body
            .get_attribute("id")
            .and_then(|x| x.value.as_str())
            .unwrap_or_else(|| "\"unknown id\"")
            .to_owned();

        let to = block
            .decor()
            .prefix()
            .and_then(|x| x.trim().strip_prefix("# "))
            .and_then(|x| x.strip_suffix(":"))
            .unwrap_or_else(|| "unknown")
            .to_owned();
        let to = TofuResourceReference::Raw(to);
        imports.push(TofuImportBlock { id, to })
    }

    Ok(imports)
}
