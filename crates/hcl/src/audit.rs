use crate::HclAuditProblem;
use crate::HclAuditor;
use crate::HclProject;
use crate::HclWriter;
use crate::TerraformBlockExtracterPatcher;
use crate::discovery::DiscoveryDepth;
use crate::discovery::discover_hcl;
use cloud_terrastodon_hcl_types::ProviderKind;
use cloud_terrastodon_hcl_types::ProviderVersionConstraint;
use cloud_terrastodon_hcl_types::ProviderVersionConstraintClause;
use cloud_terrastodon_hcl_types::SemVer;
use cloud_terrastodon_hcl_types::TerraformBlock;
use cloud_terrastodon_hcl_types::TerraformProviderInfo;
use hcl::edit::expr::Expression;
use hcl::edit::expr::ObjectKey;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::VisitMut;
use std::collections::HashSet;
use std::path::Path;
use tracing::debug;
use tracing::info;
use tracing::warn;

const AZURE_ID_ATTRIBUTES: [&str; 2] = ["tenant_id", "subscription_id"];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct MissingAzureIdentityAttribute {
    block_kind: &'static str,
    attribute: &'static str,
}

#[derive(Debug, Default)]
pub struct TerraformAuditor;

#[async_trait::async_trait]
impl HclAuditor for TerraformAuditor {
    async fn audit(
        &mut self,
        mut hcl: HclProject,
    ) -> eyre::Result<(HclProject, Vec<HclAuditProblem>)> {
        let main_body = project_body(&hcl);
        let missing_azure_identity_attributes = find_missing_azure_identity_attributes(&main_body);
        let providers_being_used = providers_being_used(&main_body);
        let terraform_block = extract_terraform_block(main_body);

        debug!("Extracted Terraform configuration: {terraform_block:#?}");

        let providers_with_version_specified: HashSet<String> =
            match &terraform_block.required_providers {
                Some(required_providers) => required_providers.0.keys().cloned().collect(),
                None => Default::default(),
            };

        let mut problems = Vec::new();

        for missing in missing_azure_identity_attributes {
            problems.push(HclAuditProblem::new(format!(
                "{} is missing required {} attribute.",
                missing.block_kind, missing.attribute
            )));
        }

        if terraform_block.backend.is_none() {
            problems.push(HclAuditProblem::new(
                "No backend is specified in the Terraform configuration. If you lose your state file, you're pooched .",
            ));
        }

        for provider in providers_with_version_specified.difference(&providers_being_used) {
            problems.push(HclAuditProblem::new(format!(
                "Provider `{provider}` is specified as required but is not being used in the configuration."
            )));
        }

        for provider in providers_being_used.difference(&providers_with_version_specified) {
            problems.push(HclAuditProblem::new(format!(
                "Provider `{provider}` is being used but does not have a version specified. This can lead to unexpected behavior."
            )));
        }

        audit_extraneous_azurerm_registration_attribute(&mut hcl, &terraform_block, &mut problems);

        Ok((hcl, problems))
    }
}

fn project_body(hcl: &HclProject) -> Body {
    hcl.values().cloned().flatten().collect()
}

fn extract_terraform_block(mut body: Body) -> TerraformBlock {
    let mut patcher = TerraformBlockExtracterPatcher::default();
    patcher.visit_body_mut(&mut body);
    patcher.terraform_block
}

fn providers_being_used(body: &Body) -> HashSet<String> {
    let mut providers = HashSet::new();

    for structure in body {
        let Some(block) = structure.as_block() else {
            continue;
        };
        if block.ident.as_str() != "resource" && block.ident.as_str() != "data" {
            continue;
        }
        let [kind, _name] = block.labels.as_slice() else {
            debug!(?block, "Block does not have exactly two labels, skipping");
            continue;
        };
        let Some((before, _after)) = kind.as_str().split_once('_') else {
            debug!(?kind, "Block kind does not have an underscore, skipping");
            continue;
        };
        providers.insert(before.to_string());
    }

    providers
}

fn audit_extraneous_azurerm_registration_attribute(
    hcl: &mut HclProject,
    terraform_block: &TerraformBlock,
    problems: &mut Vec<HclAuditProblem>,
) {
    let Some(azurerm_provider) = terraform_block
        .required_providers
        .as_ref()
        .and_then(|required_providers| required_providers.0.get("azurerm"))
    else {
        return;
    };

    if azurerm_provider.source.kind != ProviderKind::AzureRM
        || !provider_constraint_is_greater_than_five(&azurerm_provider.version)
    {
        return;
    }

    let provider_version = azurerm_provider.version.to_string();
    for body in hcl.values_mut() {
        for provider_block in body.get_blocks_mut("provider") {
            if !provider_block.has_exact_labels(&["azurerm"]) {
                continue;
            }

            let Some(attribute) = provider_block
                .body
                .get_attribute("resource_provider_registrations")
            else {
                continue;
            };
            if attribute.value.as_str() != Some("none") {
                continue;
            }

            provider_block
                .body
                .remove_attribute("resource_provider_registrations");
            problems.push(HclAuditProblem::new(format!(
                "The azurerm provider constraint `{provider_version}` is newer than 5.0.0, so `resource_provider_registrations = \"none\"` is no longer necessary."
            )));
        }
    }
}

fn provider_constraint_is_greater_than_five(constraint: &ProviderVersionConstraint) -> bool {
    let threshold = SemVer {
        major: 5,
        minor: Some(0),
        patch: Some(0),
        pre_release: None,
    };

    let has_strict_lower_bound = constraint.clauses.iter().any(|clause| match clause {
        ProviderVersionConstraintClause::Equals(version) => version > &threshold,
        ProviderVersionConstraintClause::Greater(version) => version >= &threshold,
        ProviderVersionConstraintClause::GreaterOrEqual(version) => version > &threshold,
        ProviderVersionConstraintClause::PatchIncrement(version) => version > &threshold,
        ProviderVersionConstraintClause::NotEquals(_)
        | ProviderVersionConstraintClause::Lesser(_)
        | ProviderVersionConstraintClause::LesserOrEqual(_) => false,
    });

    let has_disqualifying_upper_bound = constraint.clauses.iter().any(|clause| {
        matches!(
            clause,
            ProviderVersionConstraintClause::Lesser(version)
                | ProviderVersionConstraintClause::LesserOrEqual(version)
                if version <= &threshold
        )
    });

    has_strict_lower_bound && !has_disqualifying_upper_bound
}
fn find_missing_azure_identity_attributes(body: &Body) -> Vec<MissingAzureIdentityAttribute> {
    let mut missing = Vec::new();

    for terraform_block in body.get_blocks("terraform") {
        for backend_block in terraform_block.body.get_blocks("backend") {
            if backend_block.has_exact_labels(&["azurerm"]) {
                add_missing_attributes(
                    &mut missing,
                    &backend_block.body,
                    "Terraform azurerm backend",
                );
            }
        }
    }

    for data_block in body.get_blocks("data") {
        let is_terraform_remote_state = data_block
            .labels
            .first()
            .is_some_and(|label| label.as_str() == "terraform_remote_state");
        let uses_azurerm_backend = data_block
            .body
            .get_attribute("backend")
            .and_then(|attribute| attribute.value.as_str())
            == Some("azurerm");

        if !is_terraform_remote_state || !uses_azurerm_backend {
            continue;
        }

        let config = data_block
            .body
            .get_attribute("config")
            .map(|attribute| &attribute.value);
        for attribute in AZURE_ID_ATTRIBUTES {
            if !config.is_some_and(|config| expression_has_object_key(config, attribute)) {
                missing.push(MissingAzureIdentityAttribute {
                    block_kind: "terraform_remote_state data block with azurerm backend",
                    attribute,
                });
            }
        }
    }

    for provider_block in body.get_blocks("provider") {
        let Some(provider_kind) = provider_block.labels.first() else {
            continue;
        };
        let block_kind = match provider_kind.as_str() {
            "azurerm" => "azurerm provider",
            "azuread" => "azuread provider",
            "azuredevops" => "azuredevops provider",
            _ => continue,
        };
        add_missing_attributes(&mut missing, &provider_block.body, block_kind);
    }

    missing
}

fn add_missing_attributes(
    missing: &mut Vec<MissingAzureIdentityAttribute>,
    body: &Body,
    block_kind: &'static str,
) {
    for attribute in AZURE_ID_ATTRIBUTES {
        if body.get_attribute(attribute).is_none() {
            missing.push(MissingAzureIdentityAttribute {
                block_kind,
                attribute,
            });
        }
    }
}

fn expression_has_object_key(expression: &Expression, expected_key: &str) -> bool {
    match expression {
        Expression::Object(object) => object.iter().any(|(key, _)| match key {
            ObjectKey::Ident(key) => key.as_str() == expected_key,
            ObjectKey::Expression(key) => key.as_str() == Some(expected_key),
        }),
        Expression::Parenthesis(parenthesis) => {
            expression_has_object_key(parenthesis.inner(), expected_key)
        }
        _ => false,
    }
}

async fn audit_latest_provider_versions(hcl: &HclProject) -> eyre::Result<Vec<HclAuditProblem>> {
    let terraform_block = extract_terraform_block(project_body(hcl));
    let mut problems = Vec::new();

    if let Some(required_providers) = &terraform_block.required_providers {
        for (key, provider) in &required_providers.0 {
            let url = format!(
                "https://{registry_url}/v1/providers/{namespace}/{provider}",
                registry_url = provider.source.hostname.0,
                namespace = provider.source.namespace.0,
                provider = provider.source.kind.provider_prefix()
            );
            let json = reqwest::Client::new()
                .get(&url)
                .send()
                .await?
                .text()
                .await?;
            let json = facet_json::from_str::<TerraformProviderInfo>(&json)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            let latest_version = json
                .versions
                .last()
                .ok_or_else(|| eyre::eyre!("Provider registry returned no versions for `{key}`"))?;
            if provider.version.is_satisfied_by(latest_version) {
                info!(
                    "Provider `{key}` version \"{}\" satisfies the latest version \"{}\".",
                    provider.version, latest_version
                );
            } else {
                problems.push(HclAuditProblem::new(format!(
                    "Provider `{key}` version \"{}\" does not satisfy the latest version \"{}\". Please update your configuration.",
                    provider.version, latest_version
                )));
            }
        }
    }

    Ok(problems)
}

pub async fn audit(source_dir: &Path) -> eyre::Result<()> {
    audit_with_fix(source_dir, false).await
}

pub async fn audit_with_fix(source_dir: &Path, fix: bool) -> eyre::Result<()> {
    info!(?source_dir, fix, "Auditing");

    let project = discover_hcl(source_dir, DiscoveryDepth::Shallow).await?;
    let original_contents = project
        .iter()
        .map(|(path, body)| (path.clone(), body.to_string()))
        .collect::<std::collections::HashMap<_, _>>();

    let mut auditor = TerraformAuditor;
    let (audited_project, mut problems) = auditor.audit(project).await?;
    problems.extend(audit_latest_provider_versions(&audited_project).await?);

    for problem in &problems {
        warn!(
            location = %problem.location,
            "{}",
            problem.message
        );
    }

    if fix {
        for (path, body) in &audited_project {
            if original_contents.get(path) == Some(&body.to_string()) {
                continue;
            }
            info!(path = %path.display(), "Applying Terraform audit fix");
            HclWriter::new(path).overwrite(body.clone()).await?;
        }
    }

    if problems.is_empty() {
        info!("Epic config win! You're doing it awesome style! 🔥🔥🔥");
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn finds_missing_azure_identity_attributes() -> eyre::Result<()> {
        let body: Body = indoc! {
            r#"
            terraform {
                backend "azurerm" {
                    subscription_id = "subscription"
                }
            }

            data "terraform_remote_state" "missing_subscription" {
                backend = "azurerm"
                config = {
                    tenant_id = "tenant"
                }
            }

            data "terraform_remote_state" "local" {
                backend = "local"
                config = {}
            }

            provider "azurerm" {
                subscription_id = "subscription"
            }

            provider "azuread" {
                tenant_id = "tenant"
            }

            provider "azuredevops" {
                subscription_id = "subscription"
            }

            provider "random" {}
            "#
        }
        .parse()?;

        assert_eq!(
            find_missing_azure_identity_attributes(&body),
            vec![
                MissingAzureIdentityAttribute {
                    block_kind: "Terraform azurerm backend",
                    attribute: "tenant_id",
                },
                MissingAzureIdentityAttribute {
                    block_kind: "terraform_remote_state data block with azurerm backend",
                    attribute: "subscription_id",
                },
                MissingAzureIdentityAttribute {
                    block_kind: "azurerm provider",
                    attribute: "tenant_id",
                },
                MissingAzureIdentityAttribute {
                    block_kind: "azuread provider",
                    attribute: "subscription_id",
                },
                MissingAzureIdentityAttribute {
                    block_kind: "azuredevops provider",
                    attribute: "tenant_id",
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn accepts_identity_attributes_in_remote_state_config() -> eyre::Result<()> {
        let body: Body = indoc! {
            r#"
            terraform {
                backend "azurerm" {
                    tenant_id = "tenant"
                    subscription_id = "subscription"
                }
            }

            data "terraform_remote_state" "complete" {
                backend = "azurerm"
                config = ({
                    "tenant_id" = "tenant"
                    "subscription_id" = "subscription"
                })
            }

            provider "azurerm" {
                tenant_id = "tenant"
                subscription_id = "subscription"
            }

            provider "azuread" {
                tenant_id = "tenant"
                subscription_id = "subscription"
            }

            provider "azuredevops" {
                tenant_id = "tenant"
                subscription_id = "subscription"
            }
            "#
        }
        .parse()?;

        assert!(find_missing_azure_identity_attributes(&body).is_empty());
        Ok(())
    }

    #[test]
    fn only_strictly_newer_azurerm_constraints_trigger_the_registration_rule() -> eyre::Result<()> {
        let at_five: ProviderVersionConstraint = ">=5.0.0".parse()?;
        let above_five: ProviderVersionConstraint = ">5.0.0".parse()?;
        let newer_minimum: ProviderVersionConstraint = ">=5.1.0".parse()?;

        assert!(!provider_constraint_is_greater_than_five(&at_five));
        assert!(provider_constraint_is_greater_than_five(&above_five));
        assert!(provider_constraint_is_greater_than_five(&newer_minimum));
        Ok(())
    }

    #[tokio::test]
    async fn removes_extraneous_registration_attribute_from_new_azurerm_provider()
    -> eyre::Result<()> {
        let body: Body = indoc! {
            r#"
            terraform {
                backend "local" {}

                required_providers {
                    azurerm = {
                        source  = "hashicorp/azurerm"
                        version = ">5.0.0"
                    }
                }
            }

            provider "azurerm" {
                tenant_id = "tenant"
                subscription_id = "subscription"
                resource_provider_registrations = "none"
                features {}
            }

            resource "azurerm_resource_group" "example" {}
            "#
        }
        .parse()?;

        let project = HclProject::from([(std::path::PathBuf::from("main.tf"), body)]);
        let (fixed_project, problems) = TerraformAuditor.audit(project).await?;

        let fixed_body = fixed_project
            .as_map()
            .get(&std::path::PathBuf::from("main.tf"))
            .expect("test project should retain its path");
        let provider_block = fixed_body
            .get_blocks("provider")
            .next()
            .expect("test provider should remain");
        assert!(
            provider_block
                .body
                .get_attribute("resource_provider_registrations")
                .is_none()
        );
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0]
                .message
                .contains("resource_provider_registrations")
        );
        assert!(problems[0].location.inner.file().ends_with("audit.rs"));
        Ok(())
    }

    #[tokio::test]
    async fn keeps_registration_attribute_for_at_least_five_azurerm_constraint() -> eyre::Result<()>
    {
        let body: Body = indoc! {
            r#"
            terraform {
                backend "local" {}

                required_providers {
                    azurerm = {
                        source  = "hashicorp/azurerm"
                        version = ">=5.0.0"
                    }
                }
            }

            provider "azurerm" {
                tenant_id = "tenant"
                subscription_id = "subscription"
                resource_provider_registrations = "none"
                features {}
            }

            resource "azurerm_resource_group" "example" {}
            "#
        }
        .parse()?;

        let project = HclProject::from([(std::path::PathBuf::from("main.tf"), body)]);
        let (audited_project, problems) = TerraformAuditor.audit(project).await?;

        let provider_block = audited_project
            .as_map()
            .get(&std::path::PathBuf::from("main.tf"))
            .expect("test project should retain its path")
            .get_blocks("provider")
            .next()
            .expect("test provider should remain");
        assert!(
            provider_block
                .body
                .get_attribute("resource_provider_registrations")
                .is_some()
        );
        assert!(problems.is_empty());
        Ok(())
    }
}
