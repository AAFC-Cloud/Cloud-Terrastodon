use azure::prelude::UserId;
use azure::prelude::Uuid;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use std::collections::HashMap;
use std::collections::HashSet;
use tofu_types::prelude::TofuDataBlock;

pub struct UserIdReferencePatcher {
    pub user_principal_name_by_user_id: HashMap<UserId, String>,
    pub used: HashSet<UserId>,
}
impl UserIdReferencePatcher {
    pub fn build_lookup_block(&mut self) -> TofuDataBlock {
        TofuDataBlock::UserLookup {
            label: "users".to_string(),
            user_principal_names: self
                .used
                .iter()
                .filter_map(|x| self.user_principal_name_by_user_id.get(x))
                .map(|x| x.to_string())
                .collect(),
        }
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
            return;
        };
    }
}
