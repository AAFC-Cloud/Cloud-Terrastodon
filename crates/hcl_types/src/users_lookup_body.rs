use crate::data_block::HclDataBlock;
use crate::prelude::AzureAdDataBlockKind;
use crate::strings::TryAsHclBlocks;
use eyre::Context;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Body;
use hcl::edit::structure::IntoBlocks;
use hcl_primitives::Ident;

#[derive(Debug, Default)]
pub struct UsersLookupBody {
    pub user_principal_names: Vec<String>,
}
impl UsersLookupBody {
    pub fn is_empty(&self) -> bool {
        self.user_principal_names.is_empty()
    }
}
impl From<UsersLookupBody> for Body {
    fn from(value: UsersLookupBody) -> Self {
        let mut body = Body::with_capacity(2);

        let data_block = HclDataBlock::AzureAD {
            kind: AzureAdDataBlockKind::Users,
            name: "users".to_string(),
            body: Body::builder()
                .attribute(Attribute::new(
                    Ident::new("user_principal_names"),
                    value.user_principal_names,
                ))
                .build(),
        };
        body.push(data_block);

        // No need to use indoc to strip indent because this gets parsed into body
        let local_block: IntoBlocks = r#"
            locals {
                users = {
                    for user in data.azuread_users.users.users :
                    user.user_principal_name => user.object_id
                }
            }
        "#
        .try_as_hcl_blocks()
        .wrap_err("Failed to parse body as HCL, this shouldn't happen".to_string())
        .unwrap();
        for block in local_block {
            body.push(block);
        }

        body
    }
}
