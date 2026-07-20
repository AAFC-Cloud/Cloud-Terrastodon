use crate::HclProject;
use crate::reflow::HclReflower;
use cloud_terrastodon_azure::EntraDirectoryObject;
use cloud_terrastodon_azure::Principal;
use cloud_terrastodon_azure::PrincipalCollection;
use cloud_terrastodon_azure::PrincipalId;
use hcl::edit::Decorate;
use hcl::edit::RawString;
use hcl::edit::expr::Expression;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_expr_mut;
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

/// Prefix any hardcoded principal IDs with comments indicating the principal type and name.
/// Prefix instead of suffix to enable sorting by principal name in editors.
pub struct ReflowPrincipalIdComments {
    principals: HashMap<PrincipalId, PrincipalComment>,
}

struct PrincipalComment {
    kind: String,
    display_name: String,
}
impl ReflowPrincipalIdComments {
    pub fn new(principals: PrincipalCollection) -> Self {
        let principals = principals
            .values()
            .map(|principal| (principal.id(), PrincipalComment::from(principal)))
            .collect();
        Self { principals }
    }

    pub fn from_directory_objects(objects: impl IntoIterator<Item = EntraDirectoryObject>) -> Self {
        let principals = objects
            .into_iter()
            .filter_map(|object| {
                let display_name = object.display_name()?.to_string();
                Some((
                    object.id(),
                    PrincipalComment {
                        kind: object.kind().to_string(),
                        display_name,
                    },
                ))
            })
            .collect();
        Self { principals }
    }
}

impl From<&Principal> for PrincipalComment {
    fn from(principal: &Principal) -> Self {
        Self {
            kind: principal.kind().to_string(),
            display_name: principal.display_name().to_string(),
        }
    }
}
#[async_trait::async_trait]
impl HclReflower for ReflowPrincipalIdComments {
    async fn reflow(&mut self, hcl: HclProject) -> eyre::Result<HclProject> {
        let mut reflowed = HclProject::new();
        for (path, mut body) in hcl {
            self.visit_body_mut(&mut body);
            reflowed.insert(path, body);
        }
        Ok(reflowed)
    }
}
impl VisitMut for ReflowPrincipalIdComments {
    fn visit_expr_mut(&mut self, node: &mut Expression) {
        // Must be a principal id
        let Some(Ok(principal_id)) = node.as_str().map(PrincipalId::from_str) else {
            return visit_expr_mut(self, node);
        };

        // Must have a matching principal
        let Some(principal) = self.principals.get(&principal_id) else {
            return visit_expr_mut(self, node);
        };

        // Update the comment, preserve existing leading whitespace while ensuring idempotent behaviour
        let mut new_prefix = format!("/* ({}) {} */", principal.kind, principal.display_name,);
        let decor = node.decor_mut();
        if let Some(existing_prefix) = decor.prefix().map(RawString::deref) {
            if existing_prefix.contains(&new_prefix) {
                // do nothing, already present
            } else {
                new_prefix = format!("{}{}", existing_prefix, new_prefix);
                decor.set_prefix(new_prefix);
            }
        } else {
            decor.set_prefix(new_prefix);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::reflow::HclReflower;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use cloud_terrastodon_azure::EntraDirectoryObject;
    use cloud_terrastodon_azure::EntraUser;
    use cloud_terrastodon_azure::Principal;
    use cloud_terrastodon_azure::PrincipalCollection;
    use eyre::ContextCompat;
    use hcl::edit::structure::Body;
    use indoc::formatdoc;
    use rand::RngExt;
    use std::path::PathBuf;

    #[tokio::test]
    pub async fn it_works_with_directory_objects() -> eyre::Result<()> {
        let user = facet_json::from_str::<EntraDirectoryObject>(
            r##"{
                "@odata.type": "#microsoft.graph.user",
                "id": "11111111-1111-1111-1111-111111111111",
                "displayName": "Example User",
                "userPrincipalName": "example.user@example.com"
            }"##,
        )?;
        let user_id = user.id();
        let mut reflower = super::ReflowPrincipalIdComments::from_directory_objects([user]);
        let body = formatdoc! {
            r#"
                resource "role_assignment" "example" {{
                    principal_id = "{user_id}"
                }}
            "#
        }
        .parse::<Body>()?;

        let hcl = [(PathBuf::from("a.tf"), body)].into();
        let mut hcl = reflower.reflow(hcl).await?;
        let body = hcl
            .remove(&PathBuf::from("a.tf"))
            .wrap_err("Missing body")?;

        assert!(
            body.to_string()
                .contains("/* (User) example.user@example.com */")
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        // Create random user principal
        let mut raw = [0u8; 128];
        rand::rng().fill(&mut raw);
        let mut noise = Unstructured::new(&raw);
        let mut user = EntraUser::arbitrary(&mut noise)?;
        user.user_principal_name = "first.last@agr.gc.ca".to_string();
        let user_id = user.id;

        // Create principal collection
        let principal_collection = PrincipalCollection::new(vec![Principal::User(Box::new(user))]);

        // Create reflower
        let mut reflower = super::ReflowPrincipalIdComments::new(principal_collection);

        // Create body
        let body = formatdoc! {
        r#"
            resource "role_assignment" "bruh" {{
                principal_id = "{user_id}"
            }}
        "#}
        .parse::<Body>()?;

        let hcl = [(PathBuf::from("a.tf"), body)].into();

        // Reflow body
        let mut hcl = reflower.reflow(hcl).await?;
        assert!(hcl.len() == 1);
        let body = hcl
            .remove(&PathBuf::from("a.tf"))
            .wrap_err("Missing body")?;

        let expected = formatdoc! {
        r#"
            resource "role_assignment" "bruh" {{
                principal_id = /* (User) first.last@agr.gc.ca */"{user_id}"
            }}
        "#
        };

        // println!("reflowed body:\n{}", body.to_string());
        assert_eq!(body.to_string(), expected);

        Ok(())
    }

    #[tokio::test]
    pub async fn it_works_array1() -> eyre::Result<()> {
        // Create random user principal
        let mut raw = [0u8; 128];
        rand::rng().fill(&mut raw);
        let mut noise = Unstructured::new(&raw);
        let mut user1 = EntraUser::arbitrary(&mut noise)?;
        user1.user_principal_name = "first.last@agr.gc.ca".to_string();
        let user_id1 = user1.id;

        let mut user2 = EntraUser::arbitrary(&mut noise)?;
        user2.user_principal_name = "hot.rod@agr.gc.ca".to_string();
        let user_id2 = user2.id;

        // Create principal collection
        let principal_collection = PrincipalCollection::new(vec![
            Principal::User(Box::new(user1)),
            Principal::User(Box::new(user2)),
        ]);

        // Create reflower
        let mut reflower = super::ReflowPrincipalIdComments::new(principal_collection);

        // Create body
        let body = formatdoc! {
        r#"
            resource "azuread_group" "bruh" {{
                members = ["{user_id1}", "{user_id2}"]
            }}
        "#}
        .parse::<Body>()?;

        let hcl = [(PathBuf::from("a.tf"), body)].into();

        // Reflow body
        let mut hcl = reflower.reflow(hcl).await?;
        assert!(hcl.len() == 1);
        let body = hcl
            .remove(&PathBuf::from("a.tf"))
            .wrap_err("Missing body")?;

        let expected = formatdoc! {
        r#"
            resource "azuread_group" "bruh" {{
                members = [/* (User) first.last@agr.gc.ca */"{user_id1}", /* (User) hot.rod@agr.gc.ca */"{user_id2}"]
            }}
        "#
        };

        // println!("reflowed body:\n{}", body.to_string());
        assert_eq!(body.to_string(), expected);

        Ok(())
    }

    #[tokio::test]
    pub async fn it_works_array2() -> eyre::Result<()> {
        // Create random user principal
        let mut raw = [0u8; 128];
        rand::rng().fill(&mut raw);
        let mut noise = Unstructured::new(&raw);
        let mut user1 = EntraUser::arbitrary(&mut noise)?;
        user1.user_principal_name = "first.last@agr.gc.ca".to_string();
        let user_id1 = user1.id;

        let mut user2 = EntraUser::arbitrary(&mut noise)?;
        user2.user_principal_name = "hot.rod@agr.gc.ca".to_string();
        let user_id2 = user2.id;

        // Create principal collection
        let principal_collection = PrincipalCollection::new(vec![
            Principal::User(Box::new(user1)),
            Principal::User(Box::new(user2)),
        ]);

        // Create reflower
        let mut reflower = super::ReflowPrincipalIdComments::new(principal_collection);

        // Create body
        let body = formatdoc! {
        r#"
            resource "azuread_group" "bruh" {{
                members = [
                    "{user_id1}",
                    "{user_id2}",
                ]
            }}
        "#}
        .parse::<Body>()?;

        let hcl = [(PathBuf::from("a.tf"), body)].into();

        // Reflow body
        let mut hcl = reflower.reflow(hcl).await?;
        assert!(hcl.len() == 1);
        let body = hcl
            .remove(&PathBuf::from("a.tf"))
            .wrap_err("Missing body")?;

        let expected = formatdoc! {
        r#"
            resource "azuread_group" "bruh" {{
                members = [
                    /* (User) first.last@agr.gc.ca */"{user_id1}",
                    /* (User) hot.rod@agr.gc.ca */"{user_id2}",
                ]
            }}
        "#
        };

        // println!("reflowed body:\n{}", body.to_string());
        assert_eq!(body.to_string(), expected);

        Ok(())
    }

    #[tokio::test]
    pub async fn it_works_idempotent() -> eyre::Result<()> {
        // Create random user principal
        let mut raw = [0u8; 128];
        rand::rng().fill(&mut raw);
        let mut noise = Unstructured::new(&raw);
        let mut user = EntraUser::arbitrary(&mut noise)?;
        user.user_principal_name = "first.last@agr.gc.ca".to_string();
        let user_id = user.id;

        // Create principal collection
        let principal_collection = PrincipalCollection::new(vec![Principal::User(Box::new(user))]);

        // Create reflower
        let mut reflower = super::ReflowPrincipalIdComments::new(principal_collection);

        // Create body
        let body = formatdoc! {
        r#"
            resource "role_assignment" "bruh" {{
                principal_id = /* (User) first.last@agr.gc.ca */ "{user_id}"
            }}
        "#}
        .parse::<Body>()?;

        let hcl = [(PathBuf::from("a.tf"), body)].into();

        // Reflow body
        let mut hcl = reflower.reflow(hcl).await?;
        assert!(hcl.len() == 1);
        let body = hcl
            .remove(&PathBuf::from("a.tf"))
            .wrap_err("Missing body")?;

        let expected = formatdoc! {
        r#"
            resource "role_assignment" "bruh" {{
                principal_id = /* (User) first.last@agr.gc.ca */ "{user_id}"
            }}
        "#
        };

        // println!("reflowed body:\n{}", body.to_string());
        assert_eq!(body.to_string(), expected);

        Ok(())
    }

    #[tokio::test]
    pub async fn it_works_idempotent2() -> eyre::Result<()> {
        // Create random user principal
        let mut raw = [0u8; 128];
        rand::rng().fill(&mut raw);
        let mut noise = Unstructured::new(&raw);
        let mut user1 = EntraUser::arbitrary(&mut noise)?;
        user1.user_principal_name = "first.last@agr.gc.ca".to_string();
        let user_id1 = user1.id;

        let mut user2 = EntraUser::arbitrary(&mut noise)?;
        user2.user_principal_name = "hot.rod@agr.gc.ca".to_string();
        let user_id2 = user2.id;

        // Create principal collection
        let principal_collection = PrincipalCollection::new(vec![
            Principal::User(Box::new(user1)),
            Principal::User(Box::new(user2)),
        ]);

        // Create reflower
        let mut reflower = super::ReflowPrincipalIdComments::new(principal_collection);

        // Create body
        let body = formatdoc! {
        r#"
            resource "azuread_group" "bruh" {{
                members = [
                    /* (User) first.last@agr.gc.ca */
                    "{user_id1}",
                    /* (User) hot.rod@agr.gc.ca */
                    "{user_id2}",
                ]
            }}
        "#}
        .parse::<Body>()?;

        let hcl = [(PathBuf::from("a.tf"), body)].into();

        // Reflow body
        let mut hcl = reflower.reflow(hcl).await?;
        assert!(hcl.len() == 1);
        let body = hcl
            .remove(&PathBuf::from("a.tf"))
            .wrap_err("Missing body")?;

        let expected = formatdoc! {
        r#"
            resource "azuread_group" "bruh" {{
                members = [
                    /* (User) first.last@agr.gc.ca */
                    "{user_id1}",
                    /* (User) hot.rod@agr.gc.ca */
                    "{user_id2}",
                ]
            }}
        "#
        };

        // println!("reflowed body:\n{}", body.to_string());
        assert_eq!(body.to_string(), expected);

        Ok(())
    }
}
