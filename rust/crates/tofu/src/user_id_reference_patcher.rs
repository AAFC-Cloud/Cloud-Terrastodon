use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::UserId;
use cloud_terrastodon_core_tofu_types::prelude::AsTofuString;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureADResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureRMResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuDataBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TryAsTofuBlocks;
use hcl::edit::expr::Array;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::visit_block_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::debug;

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

    fn convert_array(&mut self, array: &mut Array) {
        for entry in array.iter_mut() {
            if self.convert_expr_if_user_id(entry) {
                let decor = entry.decor_mut();
                decor.set_prefix("\n");
            }
        }
        array.set_trailing_comma(true);
        array.set_trailing("\n");
    }

    fn convert_expr_if_user_id(&mut self, expr: &mut Expression) -> bool {
        if let Some(value) = expr.as_str()
            && let Ok(id) = value.parse::<UserId>()
            && let Some(mail) = self.user_principal_name_by_user_id.get(&id)
            && let Ok(new_expr) = format!("local.users[\"{}\"]", mail).parse::<Expression>()
        {
            *expr = new_expr;
            self.used.insert(id);
            true
        } else {
            false
        }
    }
}
impl VisitMut for UserIdReferencePatcher {
    fn visit_block_mut(&mut self, node: &mut hcl::edit::structure::Block) {
        let tofu_resource_kind = node
            .labels
            .first()
            .map(|x| x.as_str())
            .and_then(|x| x.parse::<TofuResourceKind>().ok());

        // Comment out empty owners/members attributes to satisfy terraform validate
        match tofu_resource_kind {
            Some(TofuResourceKind::AzureAD(TofuAzureADResourceKind::Group)) => {
                debug!("Converting owners and members for azuread_security_group");
                for key in ["owners", "members"] {
                    if let Some(ref mut attr) = node.body.get_attribute_mut(key)
                        && let Some(ref mut array) = attr.value_mut().as_array_mut()
                    {
                        if array.is_empty() {
                            array.set_trailing("");
                            attr.decor_mut().set_prefix("#");
                        } else {
                            self.convert_array(array);
                        }
                    }
                }
            }
            Some(TofuResourceKind::AzureRM(TofuAzureRMResourceKind::RoleAssignment)) => {
                debug!("Converting principal_id for azurerm_role_assignment");
                if node
                    .body
                    .get_attribute("principal_type")
                    .and_then(|x| x.value.as_str())
                    == Some("User")
                {
                    if let Some(mut attr) = node.body.get_attribute_mut("principal_id") {
                        self.convert_expr_if_user_id(attr.value_mut());
                    }
                }
            }
            _ => {}
        }

        visit_block_mut(self, node);
    }
}
