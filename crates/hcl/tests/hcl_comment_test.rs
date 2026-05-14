use hcl::edit::{Decor, Decorate, structure::Body};
use indoc::indoc;

#[test]
fn hcl_body_only_comments() {
    let text = indoc! {r#"
            # this is a comment
        "#};
    let body: Body = text.parse().unwrap();
    println!("{:#?}", body);
    assert!(body.iter().count() == 0);
    assert_ne!(*body.decor(), Decor::default());
}

#[test]
fn hcl_body_also_comments() {
    let text = indoc! {r#"
            # this is a comment
            locals {}
        "#};
    let body: Body = text.parse().unwrap();
    println!("{:#?}", body);
    assert!(body.iter().count() == 1);

    // it doesn't eq the default since it has Some(Empty) for suffix instead of None
    assert_eq!(*body.decor(), {
        let mut decor = Decor::default();
        decor.set_suffix("");
        decor
    });
}
