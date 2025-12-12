use crate::reflow::HclReflower;
use cloud_terrastodon_azure::prelude::PrincipalCollection;
use cloud_terrastodon_azure::prelude::PrincipalId;
use hcl::edit::Decorate;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_expr_mut;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
pub struct ReflowPrincipalIdComments {
    principals: PrincipalCollection,
}
impl ReflowPrincipalIdComments {
    pub fn new(principals: PrincipalCollection) -> Self {
        Self { principals }
    }
}
#[async_trait::async_trait]
impl HclReflower for ReflowPrincipalIdComments {
    async fn reflow(
        &mut self,
        hcl: HashMap<PathBuf, Body>,
    ) -> eyre::Result<HashMap<PathBuf, Body>> {
        let mut reflowed = HashMap::new();
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

        // Update the comment
        let comment = format!("({}) {}", principal.kind(), principal.display_name(),);
        let decor = node.decor_mut();
        let existing_suffix = decor.suffix();
        decor.set_suffix(if let Some(existing_suffix) = existing_suffix {
            if existing_suffix.is_empty() {
                format!(" // {comment}")
            } else {
                format!(" // {comment}\n{}", existing_suffix.deref())
            }
        } else {
            format!(" // {comment}")
        });
    }
}

#[cfg(test)]
mod test {
    use crate::reflow::HclReflower;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use cloud_terrastodon_azure::prelude::Principal;
    use cloud_terrastodon_azure::prelude::PrincipalCollection;
    use cloud_terrastodon_azure::prelude::User;
    use hcl::edit::structure::Body;
    use indoc::formatdoc;
    use rand::Rng;
    use std::path::PathBuf;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        // Create random user principal
        let mut raw = [0u8; 128];
        rand::rng().fill(&mut raw);
        let mut noise = Unstructured::new(&raw);
        let mut user = User::arbitrary(&mut noise)?;
        user.user_principal_name = "first.last@agr.gc.ca".to_string();
        let user_id = user.id.clone();

        // Create principal collection
        let principal_collection = PrincipalCollection::new(vec![Principal::User(user)]);

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
            .ok_or_else(|| eyre::eyre!("Missing body"))?;

        let expected = formatdoc! {
        r#"
            resource "role_assignment" "bruh" {{
                principal_id = "{user_id}" // (User) first.last@agr.gc.ca
            }}
        "#
        };

        println!("reflowed body:\n{}", body.to_string());
        assert_eq!(body.to_string(), expected);


        Ok(())
    }
}
