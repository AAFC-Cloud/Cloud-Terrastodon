use hcl::edit::structure::Body;

#[test]
fn utf8_problem() {
    let text = r#"
            import {
                id = "omitted"
                to = azuread_group.écurité
            }
        "#;
    let _body: Body = text.parse().unwrap();
}
#[test]
fn utf8_problem2() {
    let text = r#"
            locals {
                é = 4
            }
            output "ééé" {
            value = local.é
            }
        "#;
    let _body: Body = text.parse().unwrap();
}
