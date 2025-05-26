use crate::data_lookup_holder::DataLookupHolder;
use cloud_terrastodon_azure::prelude::NameLookupHelper;
use cloud_terrastodon_azure::prelude::ScopeImpl;
use cloud_terrastodon_hcl_types::prelude::AzureRMProviderDataBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLDataBlock;
use cloud_terrastodon_hcl_types::prelude::HCLDataBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_hcl_types::prelude::TryAsHCLBlocks;
use eyre::Result;
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
            ScopeImpl::PolicyDefinition(_) => HCLDataBlockReference::AzureRM {
                kind: AzureRMProviderDataBlockKind::PolicyDefinition,
                name: name.to_owned().sanitize(),
            },
            ScopeImpl::PolicySetDefinition(_) => HCLDataBlockReference::AzureRM {
                kind: AzureRMProviderDataBlockKind::PolicySetDefinition,
                name: name.to_owned().sanitize(),
            },
            ScopeImpl::ResourceGroup(_) => HCLDataBlockReference::AzureRM {
                kind: AzureRMProviderDataBlockKind::ResourceGroup,
                name: name.to_owned().sanitize(),
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
        let data_block = HCLDataBlock::LookupByName {
            reference,
            name: name.to_owned(),
        };

        // Add the data block to the body
        data_block.try_as_hcl_blocks()?.for_each(|b| body.push(b));
    }
    Ok((body, lookup_holder))
}
