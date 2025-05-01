use eyre::Context;
use hcl::edit::structure::Body;

use crate::data::HCLDataBlock;
use crate::strings::AsHCLString;
use crate::strings::TryAsHCLBlocks;

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

        let data_block = HCLDataBlock::UserLookup {
            label: "users".to_string(),
            user_principal_names: value.user_principal_names,
        }
        .as_hcl_string();

        // No need to use indoc to strip indent because this gets parsed into body
        let local_block = r#"
            locals {
                users = {
                    for user in data.azuread_users.users.users :
                    user.user_principal_name => user.object_id
                }
            }
        "#;

        for hcl in [data_block.as_str(), local_block] {
            let blocks = hcl
                .try_as_hcl_blocks()
                .wrap_err(format!(
                    "Failed to parse body as HCL, this shouldn't happen:\n```\n{hcl}\n```"
                ))
                .unwrap();
            for block in blocks {
                body.push(block);
            }
        }

        body
    }
}
