use crate::data_lookup_holder::DataLookupHolder;
use cloud_terrastodon_azure::prelude::NameLookupHelper;
use cloud_terrastodon_azure::prelude::ScopeImpl;
use cloud_terrastodon_hcl_types::prelude::AzureRmDataBlockKind;
use cloud_terrastodon_hcl_types::prelude::HclDataBlock;
use cloud_terrastodon_hcl_types::prelude::DataBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use eyre::Result;
use hcl::edit::Decorated;
use hcl::edit::Ident;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
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
            ScopeImpl::PolicyDefinition(_) => DataBlockReference::AzureRM {
                kind: AzureRmDataBlockKind::PolicyDefinition,
                name: name.to_owned().sanitize(),
            },
            ScopeImpl::PolicySetDefinition(_) => DataBlockReference::AzureRM {
                kind: AzureRmDataBlockKind::PolicySetDefinition,
                name: name.to_owned().sanitize(),
            },
            ScopeImpl::ResourceGroup(_) => DataBlockReference::AzureRM {
                kind: AzureRmDataBlockKind::ResourceGroup,
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
        let data_block = HclDataBlock::Other {
            provider: reference.provider_kind(),
            kind: reference.kind().to_owned(),
            name: name.to_owned(),
            body: Body::builder()
                .attribute(Attribute::new(
                    Ident::new("name"),
                    Expression::String(Decorated::new(name.to_owned())),
                ))
                .build(),
        };

        // Add the data block to the body
        body.push(data_block);
    }
    Ok((body, lookup_holder))
}
