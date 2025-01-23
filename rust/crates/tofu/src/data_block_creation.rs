use crate::data_lookup_holder::DataLookupHolder;
use eyre::Result;
use cloud_terrastodon_core_azure::prelude::NameLookupHelper;
use cloud_terrastodon_core_azure::prelude::ScopeImpl;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureRMDataKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuDataBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuDataReference;
use cloud_terrastodon_core_tofu_types::prelude::TryAsTofuBlocks;
use hcl::edit::structure::Body;
use std::collections::HashSet;
use tracing::warn;

pub async fn create_data_blocks_for_ids(
    ids: &HashSet<ScopeImpl>,
) -> Result<(Body, DataLookupHolder)> {
    let mut body = Body::new();
    let mut name_helper = NameLookupHelper::default();
    let mut lookup_holder = DataLookupHolder::default();

    for scope in ids {
        // Look up the name for the scope
        let Some(name) = name_helper.get_name_for_scope(scope).await? else {
            warn!("Failed to find name for {scope}");
            continue;
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
            ScopeImpl::ResourceGroup(_) => TofuDataReference::AzureRM {
                kind: TofuAzureRMDataKind::ResourceGroup,
                name: name.to_owned(),
            },
            x => {
                warn!(
                    "Data reference block creation missing impl for {x:?} in {} {}:{}",
                    file!(),
                    line!(),
                    column!()
                );
                continue;
            }
        };

        // Add the reference to the lookup
        lookup_holder
            .data_references_by_id
            .insert(scope.to_owned(), reference.clone());

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
