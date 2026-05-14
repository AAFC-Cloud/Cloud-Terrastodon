use cloud_terrastodon_hcl::reflow::HclReflower;
use cloud_terrastodon_hcl::reflow::ReflowAzureDevOpsGitRepositoryInitializationAttributes;
use cloud_terrastodon_hcl::reflow::ReflowByBlockIdentifier;
use cloud_terrastodon_hcl::reflow::ReflowExpressionsUseImportedResourceBlocks;
use cloud_terrastodon_hcl::reflow::ReflowRemoveDefaultAttributes;
use hcl::edit::structure::Body;
use indoc::indoc;
use std::collections::HashMap;
use std::path::PathBuf;

fn parse_body(input: &str) -> eyre::Result<Body> {
    Ok(input.parse()?)
}

async fn apply_reflower<'a, R>(
    mut reflower: R,
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> eyre::Result<HashMap<PathBuf, Body>>
where
    R: HclReflower,
{
    let hcl = files
        .into_iter()
        .map(|(path, body)| Ok((PathBuf::from(path), parse_body(body)?)))
        .collect::<eyre::Result<HashMap<_, _>>>()?;
    reflower.reflow(hcl).await
}

#[tokio::test]
async fn reflow_by_block_identifier_preserves_comment_only_body() -> eyre::Result<()> {
    let path = PathBuf::from("comments.tf");
    let original = parse_body(indoc! {r#"
        # this is a comment
    "#})?;
    let expected = original.to_string();

    let reflowed = ReflowByBlockIdentifier
        .reflow(HashMap::from([(path.clone(), original)]))
        .await?;

    assert_eq!(reflowed.get(&path).unwrap().to_string(), expected);
    Ok(())
}

#[tokio::test]
async fn reflow_by_block_identifier_co_locates_import_and_moved_above_resource() -> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier,
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
async fn reflow_by_block_identifier_co_locates_import_for_for_each_resource() -> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowByBlockIdentifier,
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
    assert!(output.contains("to = azuread_application_owner.prod[\"141e1371-9290-4d81-9e3e-1b1658b265f4\"]"));
    Ok(())
}

#[tokio::test]
async fn reflow_azure_devops_git_repository_adds_initialization_and_lifecycle() -> eyre::Result<()> {
    let reflowed = apply_reflower(
        ReflowAzureDevOpsGitRepositoryInitializationAttributes,
        [("repo.tf", indoc! {r#"
            resource "azuredevops_git_repository" "repo" {
              name = "demo"
            }
        "#})],
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
async fn reflow_remove_default_attributes_prunes_default_role_assignment_fields() -> eyre::Result<()> {
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
async fn reflow_expressions_use_imported_resource_blocks_rewrites_matching_ids() -> eyre::Result<()> {
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

    let output = reflowed.get(&PathBuf::from("imports.tf")).unwrap().to_string();
    assert!(output.contains("scope = azurerm_resource_group.main.id"));
    Ok(())
}