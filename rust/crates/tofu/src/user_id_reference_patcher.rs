use anyhow::Result;
use azure::prelude::UserId;
use azure::prelude::Uuid;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use std::collections::HashMap;
use std::collections::HashSet;
use tofu_types::prelude::AsTofuString;
use tofu_types::prelude::TofuDataBlock;
use tofu_types::prelude::TryAsTofuBlocks;

pub struct UserIdReferencePatcher {
    pub user_principal_name_by_user_id: HashMap<UserId, String>,
    pub used: HashSet<UserId>,
}
impl UserIdReferencePatcher {
    /// Returns None if no user references were transformed.
    pub fn build_lookup_blocks(&mut self) -> Result<Option<Body>> {
        if self.used.is_empty() {
            return Ok(None);
        }

        let mut body = Body::with_capacity(2);

        let data_block = TofuDataBlock::UserLookup {
            label: "users".to_string(),
            user_principal_names: self
                .used
                .iter()
                .filter_map(|x| self.user_principal_name_by_user_id.get(x))
                .map(|x| x.to_string())
                .collect(),
        }
        .as_tofu_string();

        // No need to use indoc to strip indent because this gets parsed into body
        let local_block = r#"
            locals {
                user_id_by_mail = {
                    for x in data.azuread_users.users.users :
                    x.user_principal_name => x.object_id
                }
            }
        "#;

        [data_block.as_str(), local_block]
            .into_iter()
            .filter_map(|x| x.try_as_tofu_blocks().ok())
            .flatten()
            .for_each(|x| body.push(x));

        Ok(Some(body))
    }
}
impl VisitMut for UserIdReferencePatcher {
    fn visit_attr_mut(&mut self, mut node: AttributeMut) {
        let key = node.key.to_string();
        let key = key.trim();
        if (key == "owners" || key == "members")
            && let Some(array) = node.value_mut().as_array_mut()
        {
            for entry in array.iter_mut() {
                if let Some(value) = entry.as_str()
                    && let Ok(id) = Uuid::parse_str(value)
                    && let Some(mail) = self.user_principal_name_by_user_id.get(&UserId(id))
                    && let Ok(expr) = format!("local.users[\"{}\"]", mail).parse::<Expression>()
                {
                    *entry = expr;
                    let decor = entry.decor_mut();
                    decor.set_prefix("\n");
                    self.used.insert(UserId(id));
                }
            }
            array.set_trailing_comma(true);
            array.set_trailing("\n");
        } else {
            visit_attr_mut(self, node);
        };
    }
}
