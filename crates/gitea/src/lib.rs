mod default_tenant;
mod gitea_api_support;
mod gitea_instance_url;
mod gitea_login;
mod gitea_logins_list_request;
mod gitea_organization;
mod gitea_organization_argument;
mod gitea_organization_get_request;
mod gitea_organization_id;
mod gitea_organization_list_request;
mod gitea_organization_name;
mod gitea_organization_repo_list_request;
mod gitea_owner_name;
mod gitea_repo;
mod gitea_repo_argument;
mod gitea_repo_enumeration_analysis_request;
mod gitea_repo_enumeration_method;
mod gitea_repo_enumeration_report;
mod gitea_repo_full_name;
mod gitea_repo_get_by_id_request;
mod gitea_repo_get_request;
mod gitea_repo_id;
mod gitea_repo_list_request;
mod gitea_repo_name;
mod gitea_repo_scan_by_id_request;
mod gitea_repo_search_request;
mod gitea_search_results;
mod gitea_tenant_alias;
mod gitea_tenant_argument;
mod gitea_tracked_tenants;
mod gitea_user;
mod gitea_user_argument;
mod gitea_user_current_get_request;
mod gitea_user_current_repo_list_request;
mod gitea_user_get_request;
mod gitea_user_id;
mod gitea_user_list_request;
mod gitea_user_repo_list_request;
mod gitea_username;

pub use crate::default_tenant::*;
pub use crate::gitea_api_support::*;
pub use crate::gitea_instance_url::*;
pub use crate::gitea_login::*;
pub use crate::gitea_logins_list_request::*;
pub use crate::gitea_organization::*;
pub use crate::gitea_organization_argument::*;
pub use crate::gitea_organization_get_request::*;
pub use crate::gitea_organization_id::*;
pub use crate::gitea_organization_list_request::*;
pub use crate::gitea_organization_name::*;
pub use crate::gitea_organization_repo_list_request::*;
pub use crate::gitea_owner_name::*;
pub use crate::gitea_repo::*;
pub use crate::gitea_repo_argument::*;
pub use crate::gitea_repo_enumeration_analysis_request::*;
pub use crate::gitea_repo_enumeration_method::*;
pub use crate::gitea_repo_enumeration_report::*;
pub use crate::gitea_repo_full_name::*;
pub use crate::gitea_repo_get_by_id_request::*;
pub use crate::gitea_repo_get_request::*;
pub use crate::gitea_repo_id::*;
pub use crate::gitea_repo_list_request::*;
pub use crate::gitea_repo_name::*;
pub use crate::gitea_repo_scan_by_id_request::*;
pub use crate::gitea_repo_search_request::*;
pub use crate::gitea_search_results::*;
pub use crate::gitea_tenant_alias::*;
pub use crate::gitea_tenant_argument::*;
pub use crate::gitea_tracked_tenants::*;
pub use crate::gitea_user::*;
pub use crate::gitea_user_argument::*;
pub use crate::gitea_user_current_get_request::*;
pub use crate::gitea_user_current_repo_list_request::*;
pub use crate::gitea_user_get_request::*;
pub use crate::gitea_user_id::*;
pub use crate::gitea_user_list_request::*;
pub use crate::gitea_user_repo_list_request::*;
pub use crate::gitea_username::*;

#[cfg(test)]
mod tests {
    use crate::analyze_gitea_repo_enumeration;
    use crate::fetch_all_gitea_repositories;
    use crate::get_default_gitea_instance_url;

    #[test_log::test(tokio::test)]
    async fn it_fetches_repositories_without_empty_ids() -> eyre::Result<()> {
        let tenant = get_default_gitea_instance_url().await?;
        let repositories = fetch_all_gitea_repositories(&tenant).await?;
        assert!(!repositories.is_empty());
        assert!(repositories.iter().all(|repo| *repo.id > 0));
        Ok(())
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_analyzes_repo_enumeration_methods() -> eyre::Result<()> {
        let tenant = get_default_gitea_instance_url().await?;
        let report = analyze_gitea_repo_enumeration(&tenant, 1000).await?;
        assert!(report.combined.repo_count > 0);
        assert!(report.search.repo_count > 0);
        assert!(report.search_missing_from_combined.is_empty());
        Ok(())
    }
}
