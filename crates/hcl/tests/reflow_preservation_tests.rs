#![cfg(test)]

use cloud_terrastodon_hcl::HclProject;
use cloud_terrastodon_hcl::reflow::HclReflower;
use cloud_terrastodon_hcl::reflow::ReflowByBlockIdentifier;
use hcl::edit::structure::Body;
use indoc::indoc;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

fn project<'a>(files: impl IntoIterator<Item = (&'a str, &'a str)>) -> eyre::Result<HclProject> {
    files
        .into_iter()
        .map(|(path, text)| Ok((PathBuf::from(path), text.parse::<Body>()?)))
        .collect()
}

fn snapshot(project: &HclProject) -> BTreeMap<PathBuf, String> {
    project
        .iter()
        .map(|(path, body)| (path.clone(), body.to_string()))
        .collect()
}

fn reparse(project: &HclProject) -> eyre::Result<HclProject> {
    project
        .iter()
        .map(|(path, body)| Ok((path.clone(), body.to_string().parse::<Body>()?)))
        .collect()
}

fn combined_text(project: &HclProject) -> String {
    snapshot(project)
        .into_values()
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
async fn missing_import_and_moved_targets_preserve_each_original_source() -> eyre::Result<()> {
    let first = indoc! {r#"
        import {
          id = "first-import"
          to = example_resource.destination
        }

        moved {
          from = example_resource.first_legacy
          to   = example_resource.destination
        }
    "#};
    let second = indoc! {r#"
        import {
          id = "second-import"
          to = example_resource.destination
        }

        moved {
          from = example_resource.second_legacy
          to   = example_resource.destination
        }
    "#};

    for mixed in [false, true] {
        let reflowed = ReflowByBlockIdentifier::new(None, mixed)
            .reflow(project([("first.tf", first), ("second.tf", second)])?)
            .await?;

        assert_eq!(reflowed.len(), 2);
        for (path, id, from) in [
            ("first.tf", "first-import", "example_resource.first_legacy"),
            (
                "second.tf",
                "second-import",
                "example_resource.second_legacy",
            ),
        ] {
            let body = reflowed
                .get(Path::new(path))
                .expect("a missing target must not relocate or discard its source");
            assert_eq!(body.len(), 2);
            assert_eq!(body.get_blocks("import").count(), 1);
            assert_eq!(body.get_blocks("moved").count(), 1);
            let import = body.get_blocks("import").next().unwrap();
            assert_eq!(
                import.body.get_attribute("id").unwrap().value.as_str(),
                Some(id)
            );
            let moved = body.get_blocks("moved").next().unwrap();
            assert_eq!(
                moved
                    .body
                    .get_attribute("from")
                    .unwrap()
                    .value
                    .to_string()
                    .trim(),
                from
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn a_comment_only_source_cannot_replace_a_generated_resource() -> eyre::Result<()> {
    for mixed in [false, true] {
        let reflowed = ReflowByBlockIdentifier::new(None, mixed)
            .reflow(project([
                (
                    "resource.example_resource.main.tf",
                    "# preserve this standalone note\n",
                ),
                (
                    "original.tf",
                    indoc! {r#"
                        resource "example_resource" "main" {
                          name = "preserve this resource"
                        }
                    "#},
                ),
            ])?)
            .await?;

        let output = combined_text(&reflowed);
        assert_eq!(output.matches("# preserve this standalone note").count(), 1);
        assert_eq!(
            output
                .matches("resource \"example_resource\" \"main\"")
                .count(),
            1
        );
        assert_eq!(output.matches("preserve this resource").count(), 1);
    }
    Ok(())
}

#[tokio::test]
async fn single_file_reflow_preserves_every_trailing_comment_without_repeating_it()
-> eyre::Result<()> {
    let mut reflower = ReflowByBlockIdentifier::new(Some(PathBuf::from("merged.tf")), false);
    let reflowed = reflower
        .reflow(project([
            (
                "first.tf",
                indoc! {r#"
                    resource "example_resource" "first" {}

                    # trailing note from the first file
                "#},
            ),
            (
                "second.tf",
                indoc! {r#"
                    resource "example_resource" "second" {}

                    # trailing note from the second file
                "#},
            ),
        ])?)
        .await?;

    assert_eq!(reflowed.len(), 1);
    let output = combined_text(&reflowed);
    for marker in [
        "resource \"example_resource\" \"first\"",
        "resource \"example_resource\" \"second\"",
        "# trailing note from the first file",
        "# trailing note from the second file",
    ] {
        assert_eq!(output.matches(marker).count(), 1, "{marker}");
    }
    let expected = snapshot(&reflowed);
    assert_eq!(
        snapshot(&reflower.reflow(reparse(&reflowed)?).await?),
        expected
    );
    Ok(())
}

#[tokio::test]
async fn splitting_a_source_emits_its_trailing_comment_once_and_stays_idempotent()
-> eyre::Result<()> {
    let mut reflower = ReflowByBlockIdentifier::default();
    let reflowed = reflower
        .reflow(project([(
            "source.tf",
            indoc! {r#"
                resource "example_resource" "first" {}
                resource "example_resource" "second" {}

                # one trailing source note
            "#},
        )])?)
        .await?;

    let output = combined_text(&reflowed);
    for marker in [
        "resource \"example_resource\" \"first\"",
        "resource \"example_resource\" \"second\"",
        "# one trailing source note",
    ] {
        assert_eq!(output.matches(marker).count(), 1, "{marker}");
    }
    let expected = snapshot(&reflowed);
    assert_eq!(
        snapshot(&reflower.reflow(reparse(&reflowed)?).await?),
        expected
    );
    Ok(())
}

#[tokio::test]
async fn cyclic_resources_fail_instead_of_producing_an_incomplete_single_file() -> eyre::Result<()>
{
    let result = ReflowByBlockIdentifier::new(Some(PathBuf::from("merged.tf")), false)
        .reflow(project([(
            "source.tf",
            indoc! {r#"
                locals {
                  shared_name = "shared by both cyclic resources"
                }

                resource "example_resource" "first" {
                  name = local.shared_name
                  depends_on = [example_resource.second]
                }

                resource "example_resource" "second" {
                  name = local.shared_name
                  depends_on = [example_resource.first]
                }
            "#},
        )])?)
        .await;

    assert!(
        result.is_err(),
        "an unrenderable dependency cycle must abort reflow before any source can be replaced"
    );
    Ok(())
}

#[tokio::test]
async fn merging_conflicting_top_level_attributes_fails_instead_of_discarding_one()
-> eyre::Result<()> {
    let result = ReflowByBlockIdentifier::new(Some(PathBuf::from("merged.tf")), false)
        .reflow(project([
            ("first.tf", "setting = \"first value\"\n"),
            ("second.tf", "setting = \"second value\"\n"),
        ])?)
        .await;

    assert!(
        result.is_err(),
        "conflicting attributes must not be silently dropped"
    );
    Ok(())
}
