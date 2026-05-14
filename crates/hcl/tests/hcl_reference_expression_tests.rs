use eyre::bail;
use hcl::edit::{
    expr::{Expression, TraversalOperator},
    structure::Body,
};
use indoc::indoc;

#[test]
fn hcl_import() -> eyre::Result<()> {
    let text = indoc! {r#"
            import {
                id = "omitted"
                to = azuread_group.mygroup
            }
        "#};
    let body: Body = text.parse().unwrap();
    println!("{:#?}", body);
    let to_attr = body
        .iter()
        .next()
        .unwrap()
        .as_block()
        .unwrap()
        .body
        .attributes()
        .skip(1)
        .next()
        .unwrap();
    assert_eq!(to_attr.key.as_str(), "to");
    let to_value = to_attr.value.as_traversal().unwrap();
    assert_eq!(
        to_value.expr.as_variable().unwrap().as_str(),
        "azuread_group"
    );
    let [op] = to_value.operators.as_slice() else {
        bail!("expected exactly one operator in to attribute value traversal");
    };
    let TraversalOperator::GetAttr(op) = op.value() else {
        bail!("expected to attribute value traversal operator to be GetAttr");
    };
    assert_eq!(op.as_str(), "mygroup");
    Ok(())
}

#[test]
fn hcl_import_for_each() -> eyre::Result<()> {
    let text = indoc! {r#"
            import {
                id = "omitted"
                to = azuread_group.mygroup["group1"]
            }
        "#};
    let body: Body = text.parse().unwrap();
    let to_attr = body
        .iter()
        .next()
        .unwrap()
        .as_block()
        .unwrap()
        .body
        .attributes()
        .skip(1)
        .next()
        .unwrap();
    assert_eq!(to_attr.key.as_str(), "to");
    let to_value = to_attr.value.as_traversal().unwrap();
    assert_eq!(
        to_value.expr.as_variable().unwrap().as_str(),
        "azuread_group"
    );
    let [op, op2] = to_value.operators.as_slice() else {
        bail!("expected exactly two operators in to attribute value traversal");
    };
    let TraversalOperator::GetAttr(op) = op.value() else {
        bail!("expected to attribute value traversal operator to be GetAttr");
    };
    assert_eq!(op.as_str(), "mygroup");
    let TraversalOperator::Index(op2) = op2.value() else {
        bail!("expected to attribute value traversal operator to be Index");
    };
    let Expression::String(op2) = op2 else {
        bail!("expected to attribute value traversal index operator operand to be a string");
    };
    assert_eq!(op2.as_str(), "group1");
    println!("{:#?}", body);
    Ok(())
}
