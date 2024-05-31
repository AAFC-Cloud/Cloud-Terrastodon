use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;
use azure::prelude::fetch_all_resource_groups;
use azure::prelude::ResourceGroup;
use azure::prelude::Subscription;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use tofu::prelude::Sanitizable;
use tofu::prelude::TofuImportBlock;
use tofu::prelude::TofuImportWriter;
use tofu::prelude::TofuProviderKind;
use tofu::prelude::TofuProviderReference;
use tracing::info;

pub struct SubRGPair {
    subscription: Rc<Subscription>,
    resource_group: ResourceGroup,
}

impl From<SubRGPair> for Choice<SubRGPair> {
    fn from(value: SubRGPair) -> Self {
        let display = format!("{}", value.resource_group.name);
        Choice {
            inner: value,
            display,
        }
    }
}

pub async fn build_resource_group_imports() -> Result<()> {
    info!("Fetching resource groups");
    let resource_groups = fetch_all_resource_groups()
        .await?
        .into_iter()
        .flat_map(|(sub, rgs)| {
            let subscription = Rc::new(sub);
            rgs.into_iter().map(move |rg| SubRGPair {
                subscription: subscription.clone(),
                resource_group: rg,
            })
        })
        .collect_vec();

    let chosen = pick_many(FzfArgs {
        choices: resource_groups,
        prompt: Some("Groups to import: ".to_string()),
        header: None,
    })?;

    let imports = chosen
        .into_iter()
        .map(|pair| {
            let block: TofuImportBlock = pair.resource_group.into();
            block.using_provider_alias(TofuProviderReference::Alias {
                kind: TofuProviderKind::AzureRM,
                name: pair.subscription.name.sanitize(),
            })
        })
        .collect_vec();

    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    TofuImportWriter::new("resource_group_imports.tf")
        .overwrite(imports)
        .await?;

    Ok(())
}
