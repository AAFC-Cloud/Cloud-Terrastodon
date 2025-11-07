use cloud_terrastodon_azure::prelude::UserId;
use cloud_terrastodon_hcl_types::prelude::AzureAdResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockResourceKind;
use cloud_terrastodon_hcl_types::prelude::UsersLookupBody;
use eyre::Result;
use hcl::edit::Decorate;
use hcl::edit::expr::Array;
use hcl::edit::expr::Expression;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_block_mut;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::debug;

pub struct UserIdReferencePatcher {
    pub user_principal_name_by_user_id: HashMap<UserId, String>,
    pub used: HashSet<UserId>,
}
impl UserIdReferencePatcher {
    /// Returns None if no user references were transformed.
    pub fn build_lookup_blocks(&mut self) -> Result<Option<UsersLookupBody>> {
        if self.used.is_empty() {
            return Ok(None);
        }
        Ok(Some(UsersLookupBody {
            user_principal_names: self
                .used
                .iter()
                .filter_map(|x| self.user_principal_name_by_user_id.get(x))
                .map(|x| x.to_string())
                .collect(),
        }))
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
            && let Ok(new_expr) = format!("local.users[\"{mail}\"]").parse::<Expression>()
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
        let resource_block_kind = node
            .labels
            .first()
            .map(|x| x.as_str())
            .and_then(|x| x.parse::<ResourceBlockResourceKind>().ok());

        // Comment out empty owners/members attributes to satisfy terraform validate
        match resource_block_kind {
            Some(ResourceBlockResourceKind::AzureAD(AzureAdResourceBlockKind::Group)) => {
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
            Some(ResourceBlockResourceKind::AzureRM(AzureRmResourceBlockKind::RoleAssignment)) => {
                debug!("Converting principal_id for azurerm_role_assignment");
                if node
                    .body
                    .get_attribute("principal_type")
                    .and_then(|x| x.value.as_str())
                    == Some("User")
                    && let Some(mut attr) = node.body.get_attribute_mut("principal_id")
                {
                    self.convert_expr_if_user_id(attr.value_mut());
                }
            }
            _ => {}
        }

        visit_block_mut(self, node);
    }
}
