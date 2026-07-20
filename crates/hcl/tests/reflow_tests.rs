use cloud_terrastodon_hcl::HclProject;
use cloud_terrastodon_hcl::reflow::HclReflower;
use cloud_terrastodon_hcl::reflow::ReflowAzureDevOpsGitRepositoryInitializationAttributes;
use cloud_terrastodon_hcl::reflow::ReflowBlockDecorations;
use cloud_terrastodon_hcl::reflow::ReflowByBlockIdentifier;
use cloud_terrastodon_hcl::reflow::ReflowExpressionsUseImportedResourceBlocks;
use cloud_terrastodon_hcl::reflow::ReflowRemoveDefaultAttributes;
use hcl::edit::structure::Body;
use indoc::indoc;
use std::path::PathBuf;

fn parse_body(input: &str) -> eyre::Result<Body> {
    Ok(input.parse()?)
}

async fn apply_reflower<'a, R>(
    mut reflower: R,
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> eyre::Result<HclProject>
where
    R: HclReflower,
{
    let hcl = files
        .into_iter()
        .map(|(path, body)| Ok((PathBuf::from(path), parse_body(body)?)))
        .collect::<eyre::Result<HclProject>>()?;
    reflower.reflow(hcl).await
}

async fn apply_reflowers(
    mut hcl: HclProject,
    reflowers: &mut [Box<dyn HclReflower>],
) -> eyre::Result<HclProject> {
    for reflower in reflowers {
        hcl = reflower.reflow(hcl).await?;
    }
    Ok(hcl)
}

#[tokio::test]
async fn reflow_by_block_identifier_preserves_comment_only_body() -> eyre::Result<()> {
    let path = PathBuf::from("comments.tf");
    let original = parse_body(indoc! {r#"
        # this is a comment
    "#})?;
    let expected = original.to_string();

    let reflowed = ReflowByBlockIdentifier::default()
        .reflow(HclProject::from([(path.clone(), original)]))
        .await?;

    assert_eq!(reflowed.get(&path).unwrap().to_string(), expected);
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_co_locates_import_and_moved_above_resource() -> eyre::Result<()>
{
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier::default(),
        [
            (
                "resource.azuread_service_principal.dev.tf",
                indoc! {r#"
                    moved {
                      from = azuread_service_principal.dev
                      to = azuread_service_principal.main
                    }
                "#},
            ),
            (
                "imports.tf",
                indoc! {r#"
                    import {
                      id = "/applications/00000000-0000-0000-0000-000000000000/servicePrincipals/11111111-1111-1111-1111-111111111111"
                      to = azuread_service_principal.main
                    }
                "#},
            ),
            (
                "legacy.tf",
                indoc! {r#"
                    resource "azuread_service_principal" "main" {
                      client_id = azuread_application_registration.main.client_id
                      owners = local.app_owner_object_ids
                    }
                "#},
            ),
        ],
    )
    .await?;

    let expected_path = PathBuf::from("resource.azuread_service_principal.main.tf");
    assert_eq!(reflowed.len(), 1);
    let output = reflowed.get(&expected_path).unwrap().to_string();

    let import_pos = output.find("import {").unwrap();
    let moved_pos = output.find("moved {").unwrap();
    let resource_pos = output
        .find("resource \"azuread_service_principal\" \"main\" {")
        .unwrap();

    assert!(import_pos < moved_pos);
    assert!(moved_pos < resource_pos);
    Ok(())
}

#[tokio::test]
async fn reflow_import_merge_is_idempotent() -> eyre::Result<()> {
    let input = HclProject::from([
        (
            PathBuf::from("imports.tf"),
            parse_body(indoc! {r#"
                import {
                  id = "/applications/00000000-0000-0000-0000-000000000000/servicePrincipals/11111111-1111-1111-1111-111111111111"
                  to = azuread_service_principal.main
                }
            "#})?,
        ),
        (
            PathBuf::from("legacy.tf"),
            parse_body(indoc! {r#"
                resource "azuread_service_principal" "main" {
                  client_id = azuread_application_registration.main.client_id
                }
            "#})?,
        ),
    ]);

    let first_pass = apply_reflowers(
        input,
        &mut [
            Box::new(ReflowByBlockIdentifier::default()),
            Box::new(ReflowBlockDecorations),
        ],
    )
    .await?;
    let first_output = first_pass
        .get(&PathBuf::from("resource.azuread_service_principal.main.tf"))
        .unwrap()
        .to_string();

    let second_input = HclProject::from([(
        PathBuf::from("resource.azuread_service_principal.main.tf"),
        parse_body(&first_output)?,
    )]);
    let second_pass = apply_reflowers(
        second_input,
        &mut [
            Box::new(ReflowByBlockIdentifier::default()),
            Box::new(ReflowBlockDecorations),
        ],
    )
    .await?;
    let second_output = second_pass
        .get(&PathBuf::from("resource.azuread_service_principal.main.tf"))
        .unwrap()
        .to_string();

    assert_eq!(second_output, first_output);
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_co_locates_import_for_for_each_resource() -> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier::default(),
        [
            (
                "imports.tf",
                indoc! {r#"
                    import {
                      to = azuread_application_owner.prod["141e1371-9290-4d81-9e3e-1b1658b265f4"]
                      id = "/applications/75b6598e-3faf-4303-8984-f88b625346e7/owners/3962c322-04a3-4880-8570-c1190db26423"
                    }
                "#},
            ),
            (
                "owners.tf",
                indoc! {r#"
                    resource "azuread_application_owner" "prod" {
                      for_each = toset(["141e1371-9290-4d81-9e3e-1b1658b265f4"])
                      application_id = azuread_application_registration.prod.id
                      owner_object_id = each.value
                    }
                "#},
            ),
        ],
    )
    .await?;

    let expected_path = PathBuf::from("resource.azuread_application_owner.prod.tf");
    assert_eq!(reflowed.len(), 1);
    let output = reflowed.get(&expected_path).unwrap().to_string();

    let import_pos = output.find("import {").unwrap();
    let resource_pos = output
        .find("resource \"azuread_application_owner\" \"prod\" {")
        .unwrap();

    assert!(import_pos < resource_pos);
    assert!(
        output.contains(
            "to = azuread_application_owner.prod[\"141e1371-9290-4d81-9e3e-1b1658b265f4\"]"
        )
    );
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_bails_on_duplicate_block_identity() -> eyre::Result<()> {
    let error = apply_reflower(
        ReflowByBlockIdentifier::default(),
        [(
            "main.tf",
            indoc! {r#"
                data "azuread_service_principal" "sharepoint" {
                  client_id = data.azuread_application_published_app_ids.well_known.result["Office365SharePointOnline"]
                }

                data "azuread_service_principal" "sharepoint" {
                  client_id = data.azuread_application_published_app_ids.well_known.result["MicrosoftGraph"]
                }
            "#},
        )],
    )
    .await
    .unwrap_err();

    let message = error.to_string();
    assert!(message.contains("Duplicate Terraform block identity"));
    assert!(message.contains("sharepoint"));
    assert!(message.contains("main.tf:1:1"));
    assert!(message.contains("main.tf:5:1"));
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_single_file_orders_block_categories() -> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier::new(Some(PathBuf::from("main.tf")), true),
        [
            (
                "z-output.tf",
                indoc! {r#"
                    output "app_id" {
                      value = azurerm_resource_group.main.id
                    }
                "#},
            ),
            (
                "terraform.tf",
                indoc! {r#"
                    terraform {
                      required_version = ">= 1.9.0"
                    }
                "#},
            ),
            (
                "provider.tf",
                indoc! {r#"
                    provider "azurerm" {
                      features {}
                    }
                "#},
            ),
            (
                "variables.tf",
                indoc! {r#"
                    variable "location" {
                      type = string
                    }
                "#},
            ),
            (
                "imports.tf",
                indoc! {r#"
                    import {
                      id = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/demo"
                      to = azurerm_resource_group.main
                    }

                    moved {
                      from = azurerm_resource_group.legacy
                      to = azurerm_resource_group.main
                    }
                "#},
            ),
            (
                "data.tf",
                indoc! {r#"
                    data "azurerm_client_config" "current" {}
                "#},
            ),
            (
                "resource.tf",
                indoc! {r#"
                    resource "azurerm_resource_group" "main" {
                      location = var.location
                      name     = "demo"
                    }
                "#},
            ),
        ],
    )
    .await?;

    assert_eq!(reflowed.len(), 1);
    let output = reflowed.get(&PathBuf::from("main.tf")).unwrap().to_string();

    let terraform_pos = output.find("terraform {").unwrap();
    let provider_pos = output.find("provider \"azurerm\" {").unwrap();
    let variable_pos = output.find("variable \"location\" {").unwrap();
    let data_pos = output
        .find("data \"azurerm_client_config\" \"current\" {")
        .unwrap();
    let import_pos = output.find("import {").unwrap();
    let moved_pos = output.find("moved {").unwrap();
    let resource_pos = output
        .find("resource \"azurerm_resource_group\" \"main\" {")
        .unwrap();
    let output_pos = output.find("output \"app_id\" {").unwrap();

    assert!(terraform_pos < provider_pos);
    assert!(provider_pos < data_pos);
    assert!(data_pos < variable_pos);
    assert!(data_pos < import_pos);
    assert!(import_pos < moved_pos);
    assert!(moved_pos < resource_pos);
    assert!(resource_pos < output_pos);
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_hybrid_merge_keeps_shared_data_separate() -> eyre::Result<()> {
    let reflowed = apply_reflower(
            ReflowByBlockIdentifier::new(None, true),
                [
                        (
                                "main.tf",
                                indoc! {r#"
                                        data "azuredevops_project" "main" {
                                            name = "CKAN_DC"
                                        }

                                        data "azuredevops_users" "ckan" {
                                            for_each = toset(["dominic.phillips@agr.gc.ca"])
                                            principal_name = each.value
                                        }

                                        resource "azuredevops_check_approval" "ckan" {
                                            project_id           = data.azuredevops_project.main.id
                                            target_resource_id   = azuredevops_environment.ckan.id
                                            target_resource_type = "environment"
                                            approvers = [for approver in data.azuredevops_users.ckan : one(approver.users).id]
                                        }

                                        resource "azuredevops_environment" "ckan" {
                                            name       = "CKAN"
                                            project_id = data.azuredevops_project.main.id
                                        }

                                        data "azuredevops_users" "pipelines" {
                                            for_each = toset(["dominic.phillips@agr.gc.ca"])
                                            principal_name = each.value
                                        }

                                        resource "azuredevops_check_approval" "pipelines" {
                                            project_id           = data.azuredevops_project.main.id
                                            target_resource_id   = azuredevops_environment.pipelines.id
                                            target_resource_type = "environment"
                                            approvers = [for approver in data.azuredevops_users.pipelines : one(approver.users).id]
                                        }

                                        resource "azuredevops_environment" "pipelines" {
                                            name       = "pipelines"
                                            project_id = data.azuredevops_project.main.id
                                        }
                                "#},
                        ),
                ],
        )
        .await?;

    assert!(reflowed.contains_key(&PathBuf::from("data.azuredevops_project.main.tf")));
    assert!(reflowed.contains_key(&PathBuf::from("resource.azuredevops_environment.ckan.tf")));
    assert!(reflowed.contains_key(&PathBuf::from(
        "resource.azuredevops_environment.pipelines.tf"
    )));
    assert!(!reflowed.contains_key(&PathBuf::from("data.azuredevops_users.ckan.tf")));
    assert!(!reflowed.contains_key(&PathBuf::from("data.azuredevops_users.pipelines.tf")));

    let pipelines = reflowed
        .get(&PathBuf::from(
            "resource.azuredevops_environment.pipelines.tf",
        ))
        .unwrap()
        .to_string();
    let env_pos = pipelines
        .find("resource \"azuredevops_environment\" \"pipelines\" {")
        .unwrap();
    let users_pos = pipelines
        .find("data \"azuredevops_users\" \"pipelines\" {")
        .unwrap();
    let approval_pos = pipelines
        .find("resource \"azuredevops_check_approval\" \"pipelines\" {")
        .unwrap();

    assert!(env_pos < users_pos);
    assert!(users_pos < approval_pos);
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_merges_single_use_variable_into_resource_file()
-> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier::new(None, true),
        [(
            "main.tf",
            indoc! {r#"
                                variable "location" {
                                    type = string
                                }

                                resource "azurerm_resource_group" "main" {
                                    location = var.location
                                    name     = "demo"
                                }
                        "#},
        )],
    )
    .await?;

    assert_eq!(reflowed.len(), 1);
    let output = reflowed
        .get(&PathBuf::from("resource.azurerm_resource_group.main.tf"))
        .unwrap()
        .to_string();
    assert!(
        output.find("variable \"location\" {").unwrap()
            < output
                .find("resource \"azurerm_resource_group\" \"main\" {")
                .unwrap()
    );
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_splits_and_preserves_locals_comments() -> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier::new(None, true),
        [(
            "main.tf",
            indoc! {r#"
                                # keep this comment
                                locals {
                                    alpha = "demo"
                                    beta  = local.alpha
                                }

                                resource "azurerm_resource_group" "main" {
                                    location = "canadacentral"
                                    name     = local.beta
                                }
                        "#},
        )],
    )
    .await?;

    assert_eq!(reflowed.len(), 1);
    let output = reflowed
        .get(&PathBuf::from("resource.azurerm_resource_group.main.tf"))
        .unwrap()
        .to_string();

    assert!(output.contains("# keep this comment"));
    assert!(output.contains("alpha = \"demo\""));
    assert!(output.contains("beta  = local.alpha") || output.contains("beta = local.alpha"));
    assert!(
        output.find("locals {").unwrap()
            < output
                .find("resource \"azurerm_resource_group\" \"main\" {")
                .unwrap()
    );
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_default_flat_keeps_support_blocks_separate() -> eyre::Result<()>
{
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier::default(),
                [(
                        "main.tf",
                        indoc! {r#"
                                data "azuredevops_project" "main" {
                                    name = "CKAN_DC"
                                }

                                data "azuredevops_users" "pipelines" {
                                    for_each = toset(["dominic.phillips@agr.gc.ca"])
                                    principal_name = each.value
                                }

                                resource "azuredevops_check_approval" "pipelines" {
                                    project_id           = data.azuredevops_project.main.id
                                    target_resource_id   = azuredevops_environment.pipelines.id
                                    target_resource_type = "environment"
                                    approvers = [for approver in data.azuredevops_users.pipelines : one(approver.users).id]
                                }

                                resource "azuredevops_environment" "pipelines" {
                                    name       = "pipelines"
                                    project_id = data.azuredevops_project.main.id
                                }
                        "#},
                )],
        )
        .await?;

    assert!(reflowed.contains_key(&PathBuf::from("data.azuredevops_project.main.tf")));
    assert!(reflowed.contains_key(&PathBuf::from("data.azuredevops_users.pipelines.tf")));
    assert!(reflowed.contains_key(&PathBuf::from(
        "resource.azuredevops_check_approval.pipelines.tf"
    )));
    assert!(reflowed.contains_key(&PathBuf::from(
        "resource.azuredevops_environment.pipelines.tf"
    )));
    Ok(())
}

#[tokio::test]
async fn reflow_block_decorations_adds_blank_line_between_blocks() -> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowBlockDecorations,
        [(
            "main.tf",
            "resource \"a\" \"one\" {}\nresource \"a\" \"two\" {}\n",
        )],
    )
    .await?;

    let output = reflowed.get(&PathBuf::from("main.tf")).unwrap().to_string();
    assert!(output.contains("resource \"a\" \"one\" {}\n\nresource \"a\" \"two\" {}"));
    Ok(())
}

#[tokio::test]
async fn reflow_azure_devops_git_repository_adds_initialization_and_lifecycle() -> eyre::Result<()>
{
    let reflowed = apply_reflower(
        ReflowAzureDevOpsGitRepositoryInitializationAttributes,
        [(
            "repo.tf",
            indoc! {r#"
            resource "azuredevops_git_repository" "repo" {
              name = "demo"
            }
        "#},
        )],
    )
    .await?;

    let output = reflowed.get(&PathBuf::from("repo.tf")).unwrap().to_string();
    assert!(output.contains("initialization {"));
    assert!(output.contains("init_type = \"Clean\""));
    assert!(output.contains("lifecycle {"));
    assert!(output.contains("ignore_changes = [initialization]"));
    Ok(())
}

#[tokio::test]
async fn reflow_remove_default_attributes_prunes_default_role_assignment_fields() -> eyre::Result<()>
{
    let reflowed = apply_reflower(
        ReflowRemoveDefaultAttributes,
        [("role.tf", indoc! {r#"
            resource "azurerm_role_assignment" "main" {
              scope                = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg"
              principal_id         = "11111111-1111-1111-1111-111111111111"
              role_definition_name = "Reader"
              role_definition_id   = "/providers/Microsoft.Authorization/roleDefinitions/abcd"
              condition            = null
              description          = ""
            }
        "#})],
    )
    .await?;

    let output = reflowed.get(&PathBuf::from("role.tf")).unwrap().to_string();
    assert!(output.contains("role_definition_name = \"Reader\""));
    assert!(!output.contains("role_definition_id"));
    assert!(!output.contains("condition = null"));
    assert!(!output.contains("description = \"\""));
    Ok(())
}

#[tokio::test]
async fn reflow_expressions_use_imported_resource_blocks_rewrites_matching_ids() -> eyre::Result<()>
{
    let resource_id = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg";
    let input = format!(
        "import {{\n  id = \"{}\"\n  to = azurerm_resource_group.main\n}}\n\nresource \"azurerm_role_assignment\" \"main\" {{\n  scope = \"{}\"\n}}\n",
        resource_id, resource_id
    );
    let reflowed = apply_reflower(
        ReflowExpressionsUseImportedResourceBlocks::default(),
        [("imports.tf", input.as_str())],
    )
    .await?;

    let output = reflowed
        .get(&PathBuf::from("imports.tf"))
        .unwrap()
        .to_string();
    assert!(output.contains("scope = azurerm_resource_group.main.id"));
    Ok(())
}
