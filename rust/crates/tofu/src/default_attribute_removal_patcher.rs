use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureADResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureRMResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceKind;
use hcl::edit::expr::Array;
use hcl::edit::expr::Expression;
use hcl::edit::visit_mut::visit_block_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use tracing::warn;

pub struct DefaultAttributeRemovalPatcher;
impl VisitMut for DefaultAttributeRemovalPatcher {
    fn visit_block_mut(&mut self, node: &mut hcl::edit::structure::Block) {
        if node.ident.as_str() != "resource" {
            return;
        }
        visit_block_mut(self, node);
        let Some(resource_kind_str) = node.labels.first().map(|x| x.as_str()) else {
            return;
        };
        // let Some(name) = node.labels.get(1).map(|x| x.as_str()) else {
        //     return;
        // };
        let Ok(resource_kind) = resource_kind_str.parse() else {
            warn!("Failed to identify resource kind for {resource_kind_str:?}");
            return;
        };

        match resource_kind {
            TofuResourceKind::AzureRM(TofuAzureRMResourceKind::RoleAssignment) => {
                // Use role name instead of ID for readability
                if node.body.has_attribute("role_definition_name")
                    && node.body.has_attribute("role_definition_id")
                {
                    node.body.remove_attribute("role_definition_id").unwrap();
                }

                // Remove null attributes
                for key in [
                    "condition",
                    "condition_version",
                    "delegated_managed_identity_resource_id",
                    "description",
                    "skip_service_principal_aad_check",
                ] {
                    if let Some(attrib) = node.body.get_attribute(key) {
                        if attrib.value.is_null() {
                            node.body.remove_attribute(key).unwrap();
                        }
                    }
                }

                // Don't both repeating the name, which is part of the ID
                let _ = node.body.remove_attribute("name");
            }
            TofuResourceKind::AzureAD(TofuAzureADResourceKind::Group) => {
                // Remove mail_enabled when security_enabled specified
                if node.body.has_attribute("security_enabled")
                    && node.body.has_attribute("mail_enabled")
                {
                    node.body.remove_attribute("mail_enabled").unwrap();
                }

                // Remove members (empty and comment) when dynamic_membership specified
                if node.body.has_attribute("members") && node.body.has_blocks("dynamic_membership")
                {
                    let mut members = node.body.get_attribute_mut("members").unwrap();
                    let mut array = Array::new();
                    array.set_trailing("");
                    members.decor_mut().set_prefix("#");
                    *members.value_mut() = Expression::Array(array);
                }

                // Remove null attributes
                for key in [
                    "description",
                    "theme",
                    "visibility",
                    "onpremises_group_type",
                ] {
                    if let Some(attrib) = node.body.get_attribute(key) {
                        if attrib.value.is_null() {
                            node.body.remove_attribute(key).unwrap();
                        }
                    }
                }

                // Remove false attributes
                for key in [
                    "assignable_to_role",
                    "auto_subscribe_new_members",
                    "external_senders_allowed",
                    "hide_from_address_lists",
                    "hide_from_outlook_clients",
                    "prevent_duplicate_names",
                    "writeback_enabled",
                ] {
                    if let Some(attrib) = node.body.get_attribute(key) {
                        if let Some(false) = attrib.value.as_bool() {
                            node.body.remove_attribute(key).unwrap();
                        }
                    }
                }

                // Remove empty list attributes
                for key in [
                    "administrative_unit_ids",
                    "behaviors",
                    "provisioning_options",
                    "types",
                ] {
                    if let Some(attrib) = node.body.get_attribute(key) {
                        if let Some(x) = attrib.value.as_array() {
                            if x.is_empty() {
                                node.body.remove_attribute(key).unwrap();
                            }
                        }
                    }
                }

                // Remove default mail nicknames
                fn is_default_nick(s: &str) -> bool {
                    // is UUID?
                    if s.parse::<Uuid>().is_ok() {
                        return true;
                    }
                    // is 8 hex chars, a dash, and a final hex char?
                    let parts: Vec<&str> = s.split('-').collect();
                    if parts.len() != 2 {
                        return false;
                    }
                    let first_part = parts[0];
                    let second_part = parts[1];
                    first_part.len() == 8
                        && first_part.chars().all(|c| c.is_ascii_hexdigit())
                        && second_part.len() == 1
                        && second_part.chars().all(|c| c.is_ascii_hexdigit())
                }
                if let Some(attrib) = node.body.get_attribute("mail_nickname") {
                    if let Some(nick) = attrib.value.as_str() {
                        if is_default_nick(nick) {
                            node.body.remove_attribute("mail_nickname").unwrap();
                        }
                    }
                }
            }
            TofuResourceKind::AzureRM(TofuAzureRMResourceKind::ResourceGroup) => {
                // Remove null attributes
                for key in ["managed_by"] {
                    if let Some(attrib) = node.body.get_attribute(key) {
                        if attrib.value.is_null() {
                            node.body.remove_attribute(key).unwrap();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
