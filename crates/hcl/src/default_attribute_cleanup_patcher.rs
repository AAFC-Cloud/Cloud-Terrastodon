use cloud_terrastodon_azure::prelude::uuid::Uuid;
use cloud_terrastodon_hcl_types::prelude::AzureADResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockKind;
use hcl::edit::Decorate;
use hcl::edit::Decorated;
use hcl::edit::expr::Array;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_block_mut;
use tracing::warn;

fn is_null_or_empty(attrib: &Attribute) -> bool {
    attrib.value.is_null() || attrib.value.as_str().filter(|x| x.is_empty()).is_some()
}

fn remove_if_null_or_empty(body: &mut Body, key: &str) {
    if let Some(attrib) = body.get_attribute(key) {
        if is_null_or_empty(attrib) {
            body.remove_attribute(key).unwrap();
        }
    }
}
fn replace_if_null_or_empty(body: &mut Body, key: &str, default: impl Into<String>) {
    if let Some(mut attrib) = body.get_attribute_mut(key) {
        if is_null_or_empty(&attrib) {
            *attrib.value_mut() = Expression::String(Decorated::new(default.into()));
        }
    }
}
fn remove_if_false(body: &mut Body, key: &str) {
    if let Some(attrib) = body.get_attribute(key) {
        if let Some(false) = attrib.value.as_bool() {
            body.remove_attribute(key).unwrap();
        }
    }
}

fn remove_if_empty_array(body: &mut Body, key: &str) {
    if let Some(attrib) = body.get_attribute(key) {
        if let Some(x) = attrib.value.as_array() {
            if x.is_empty() {
                body.remove_attribute(key).unwrap();
            }
        }
    }
}

fn remove_second_if_both_present(body: &mut Body, keep: &str, remove: &str) {
    if body.has_attribute(keep) && body.has_attribute(remove) {
        body.remove_attribute(remove).unwrap();
    }
}

pub struct DefaultAttributeCleanupPatcher;
impl VisitMut for DefaultAttributeCleanupPatcher {
    fn visit_block_mut(&mut self, node: &mut hcl::edit::structure::Block) {
        if node.ident.as_str() != "resource" {
            return;
        }
        visit_block_mut(self, node);
        let [resource_kind, name] = node.labels.as_slice() else {
            return;
        };
        let Ok(resource_kind) = resource_kind.parse() else {
            warn!("Failed to identify resource kind for {resource_kind:?}");
            return;
        };

        let name = name.as_str();
        let body = &mut node.body;
        match resource_kind {
            ResourceBlockKind::AzureRM(AzureRMResourceBlockKind::RoleAssignment) => {
                // Use role name instead of ID for readability
                remove_second_if_both_present(body, "role_definition_name", "role_definition_id");

                remove_if_null_or_empty(body, "condition");
                remove_if_null_or_empty(body, "condition_version");
                remove_if_null_or_empty(body, "delegated_managed_identity_resource_id");
                remove_if_null_or_empty(body, "description");
                remove_if_null_or_empty(body, "skip_service_principal_aad_check");

                // Don't both repeating the name, which is part of the ID
                let _ = body.remove_attribute("name");
            }
            ResourceBlockKind::AzureAD(AzureADResourceBlockKind::Group) => {
                // Remove mail_enabled when security_enabled specified
                remove_second_if_both_present(body, "security_enabled", "mail_enabled");

                // Remove members (empty and comment) when dynamic_membership specified
                if body.has_attribute("members") && body.has_blocks("dynamic_membership") {
                    let mut members = body.get_attribute_mut("members").unwrap();
                    let mut array = Array::new();
                    array.set_trailing("");
                    members.decor_mut().set_prefix("#");
                    *members.value_mut() = Expression::Array(array);
                }

                remove_if_null_or_empty(body, "description");
                remove_if_null_or_empty(body, "theme");
                remove_if_null_or_empty(body, "visibility");
                remove_if_null_or_empty(body, "onpremises_group_type");

                remove_if_false(body, "assignable_to_role");
                remove_if_false(body, "auto_subscribe_new_members");
                remove_if_false(body, "external_senders_allowed");
                remove_if_false(body, "hide_from_address_lists");
                remove_if_false(body, "hide_from_outlook_clients");
                remove_if_false(body, "prevent_duplicate_names");
                remove_if_false(body, "writeback_enabled");

                remove_if_empty_array(body, "administrative_unit_ids");
                remove_if_empty_array(body, "behaviors");
                remove_if_empty_array(body, "provisioning_options");
                remove_if_empty_array(body, "types");

                // Remove default mail nicknames
                fn is_default_nick(s: &str) -> bool {
                    s.parse::<Uuid>().is_ok()
                }
                if let Some(attrib) = body.get_attribute("mail_nickname") {
                    if let Some(nick) = attrib.value.as_str() {
                        if is_default_nick(nick) {
                            body.remove_attribute("mail_nickname").unwrap();
                        }
                    }
                }
            }
            ResourceBlockKind::AzureRM(AzureRMResourceBlockKind::ResourceGroup) => {
                remove_if_null_or_empty(body, "managed_by");
            }
            ResourceBlockKind::AzureDevOps(AzureDevOpsResourceBlockKind::Project) => {
                let features = body.get_attribute("features");
                if let Some(features) = features {
                    if features
                        .value
                        .as_object()
                        .map(|x| x.is_empty())
                        .unwrap_or(false)
                    {
                        body.remove_attribute("features");
                    }
                }
            }
            ResourceBlockKind::AzureRM(AzureRMResourceBlockKind::PolicyDefinition)
            | ResourceBlockKind::AzureRM(AzureRMResourceBlockKind::PolicySetDefinition) => {
                replace_if_null_or_empty(
                    body,
                    "display_name",
                    body.get_attribute("name").unwrap().value.as_str().unwrap().to_owned(),
                );
            }
            _ => {}
        }
    }
}
