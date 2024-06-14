use crate::data_lookup_holder::DataLookupHolder;
use crate::import_lookup_holder::ResourceId;
use anyhow::bail;
use anyhow::Result;
use azure::prelude::NameLookupHelper;
use azure::prelude::ScopeImpl;
use hcl::edit::structure::Body;
use std::collections::HashSet;
use std::str::FromStr;
use tofu_types::prelude::TofuAzureRMDataKind;
use tofu_types::prelude::TofuDataBlock;
use tofu_types::prelude::TofuDataReference;
use tofu_types::prelude::TryAsTofuBlocks;

pub async fn create_data_blocks_for_ids(
    ids: &HashSet<ResourceId>,
) -> Result<(Body, DataLookupHolder)> {

    let mut body = Body::new();
    let mut name_helper = NameLookupHelper::default();
    let mut lookup_holder = DataLookupHolder::default();

    for id in ids {
        // Convert ID string to scope
        let scope = ScopeImpl::from_str(id)?;

        // Look up the name for the scope
        let Some(name) = name_helper.get_name_for_scope(&scope).await? else {
            bail!("Failed to find name for {id}");
        };

        // Create the data reference
        let reference = match &scope {
            ScopeImpl::PolicyDefinition(_) => TofuDataReference::AzureRM {
                kind: TofuAzureRMDataKind::PolicyDefinition,
                name: name.to_owned(),
            },
            ScopeImpl::PolicySetDefinition(_) => TofuDataReference::AzureRM {
                kind: TofuAzureRMDataKind::PolicySetDefinition,
                name: name.to_owned(),
            },
            _ => todo!(),
        };

        // Add the reference to the lookup
        lookup_holder
            .data_references_by_id
            .insert(scope, reference.clone());

        // Create the data block
        let data_block = TofuDataBlock::LookupByName {
            reference,
            name: name.to_owned(),
        };

        // Add the data block to the body
        data_block.try_as_tofu_blocks()?.for_each(|b| body.push(b));
    }
    Ok((body, lookup_holder))
}
