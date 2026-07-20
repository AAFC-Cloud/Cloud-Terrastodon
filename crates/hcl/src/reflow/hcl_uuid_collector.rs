use crate::HclProject;
use cloud_terrastodon_azure::uuid::Uuid;
use hcl::edit::expr::Expression;
use hcl::edit::visit::Visit;
use hcl::edit::visit::visit_expr;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Default)]
pub struct HclUuidCollector {
    ids: HashSet<Uuid>,
}

impl HclUuidCollector {
    pub fn collect(hcl: &HclProject) -> Vec<Uuid> {
        let mut collector = Self::default();
        for body in hcl.values() {
            collector.visit_body(body);
        }
        collector.ids.into_iter().collect()
    }
}

impl Visit for HclUuidCollector {
    fn visit_expr(&mut self, node: &Expression) {
        if let Some(id) = node.as_str().and_then(|value| Uuid::from_str(value).ok()) {
            self.ids.insert(id);
            return;
        }

        visit_expr(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hcl::edit::structure::Body;
    use std::path::PathBuf;

    #[test]
    fn collects_distinct_uuid_string_literals() -> eyre::Result<()> {
        let body = r#"
            resource "example" "one" {
                principal_id = "11111111-1111-1111-1111-111111111111"
                unrelated = "not an object id"
            }
            resource "example" "two" {
                principal_ids = [
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                ]
            }
        "#
        .parse::<Body>()?;
        let hcl = [(PathBuf::from("main.tf"), body)].into();

        let mut ids = HclUuidCollector::collect(&hcl);
        ids.sort_unstable();

        assert_eq!(
            ids,
            vec![
                Uuid::from_u128(0x11111111111111111111111111111111),
                Uuid::from_u128(0x22222222222222222222222222222222),
            ]
        );
        Ok(())
    }
}
