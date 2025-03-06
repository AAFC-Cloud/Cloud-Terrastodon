mod apply_processed;
mod azure_devops_project_import_wizard_menu;
mod browse_oauth2_permission_grants;
mod browse_policy_assignments;
mod browse_policy_definitions;
mod browse_resource_groups;
mod browse_resources;
mod browse_role_assignments;
mod browse_security_groups;
mod browse_users;
mod build_group_imports;
mod build_imports_from_existing;
mod build_policy_imports;
mod build_resource_group_imports;
mod build_role_assignment_imports;
mod bulk_user_id_lookup;
mod clean_all;
mod clean_imports;
mod clean_processed;
mod copy_azurerm_backend_menu;
mod create_new_action_variant;
mod create_oauth2_permission_grants;
mod create_role_assignment_menu;
mod dump_tags;
mod dump_work_items;
mod find_resource_owners;
mod init_processed;
mod jump_to_block;
mod list_imports;
mod open_dir;
mod pim_activate;
mod plan_processed;
mod populate_cache;
mod remove_oauth2_permission_grants;
mod resource_group_import_wizard_menu;
mod run_query;
mod tag_empty_resource_groups;
mod tag_resources_menu;
pub const THIS_FILE: &str = file!();
pub mod prelude {
    pub use crate::interactive::apply_processed::*;
    pub use crate::interactive::azure_devops_project_import_wizard_menu::*;
    pub use crate::interactive::browse_oauth2_permission_grants::*;
    pub use crate::interactive::browse_policy_assignments::*;
    pub use crate::interactive::browse_policy_definitions::*;
    pub use crate::interactive::browse_resource_groups::*;
    pub use crate::interactive::browse_resources::*;
    pub use crate::interactive::browse_role_assignments::*;
    pub use crate::interactive::browse_security_groups::*;
    pub use crate::interactive::browse_users::*;
    pub use crate::interactive::build_group_imports::*;
    pub use crate::interactive::build_imports_from_existing::*;
    pub use crate::interactive::build_policy_imports::*;
    pub use crate::interactive::build_resource_group_imports::*;
    pub use crate::interactive::build_role_assignment_imports::*;
    pub use crate::interactive::bulk_user_id_lookup::*;
    pub use crate::interactive::clean_all::*;
    pub use crate::interactive::clean_imports::*;
    pub use crate::interactive::clean_processed::*;
    pub use crate::interactive::copy_azurerm_backend_menu::*;
    pub use crate::interactive::create_new_action_variant::*;
    pub use crate::interactive::create_oauth2_permission_grants::*;
    pub use crate::interactive::create_role_assignment_menu::*;
    pub use crate::interactive::dump_tags::*;
    pub use crate::interactive::dump_work_items::*;
    pub use crate::interactive::find_resource_owners::*;
    pub use crate::interactive::init_processed::*;
    pub use crate::interactive::jump_to_block::*;
    pub use crate::interactive::list_imports::*;
    pub use crate::interactive::open_dir::*;
    pub use crate::interactive::pim_activate::*;
    pub use crate::interactive::plan_processed::*;
    pub use crate::interactive::populate_cache::*;
    pub use crate::interactive::remove_oauth2_permission_grants::*;
    pub use crate::interactive::resource_group_import_wizard_menu::*;
    pub use crate::interactive::run_query::*;
    pub use crate::interactive::tag_empty_resource_groups::*;
    pub use crate::interactive::tag_resources_menu::*;
}
