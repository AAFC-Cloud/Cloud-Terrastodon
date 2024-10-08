use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_tofu_types::prelude::AsTofuString;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureADResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuDataBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TryAsTofuBlocks;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::visit_block_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct UserIdReferencePatcher {
    pub user_principal_name_by_user_id: HashMap<Uuid, String>,
    pub used: HashSet<Uuid>,
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
                users = {
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
    fn visit_block_mut(&mut self, node: &mut hcl::edit::structure::Block) {
        visit_block_mut(self, node);
        // Comment out empty owners/members attributes to satisfy terraform validate
        if node
            .labels
            .first()
            .map(|x| x.as_str())
            .and_then(|x| x.parse::<TofuResourceKind>().ok())
            == Some(TofuResourceKind::AzureAD(TofuAzureADResourceKind::Group))
        {
            for key in ["owners", "members"] {
                if let Some(ref mut attr) = node.body.get_attribute_mut(key)
                    && let Some(ref mut array) = attr.value_mut().as_array_mut()
                    && array.is_empty()
                {
                    array.set_trailing("");
                    attr.decor_mut().set_prefix("#");
                }
            }
        }
    }
    fn visit_attr_mut(&mut self, mut node: AttributeMut) {
        let key = node.key.to_string();
        let key = key.trim();
        if (key == "owners" || key == "members")
            && let Some(array) = node.value_mut().as_array_mut()
        {
            for entry in array.iter_mut() {
                if let Some(value) = entry.as_str()
                    && let Ok(id) = Uuid::parse_str(value)
                    && let Some(mail) = self.user_principal_name_by_user_id.get(&id)
                    && let Ok(expr) = format!("local.users[\"{}\"]", mail).parse::<Expression>()
                {
                    *entry = expr;
                    let decor = entry.decor_mut();
                    decor.set_prefix("\n");
                    self.used.insert(id);
                }
            }
            array.set_trailing_comma(true);
            array.set_trailing("\n");
        } else {
            visit_attr_mut(self, node);
        };
    }
}
