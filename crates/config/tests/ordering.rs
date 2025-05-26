use ordermap::OrderSet;

/// Indexmap does not check ordering for comparison, so we use ordermap instead.
#[test]
pub fn it_works() -> eyre::Result<()> {
    let mut a = OrderSet::new();
    a.insert("a".to_string());
    a.insert("b".to_string());
    let mut b = a.clone();
    b.swap_indices(0, 1);
    assert_ne!(a, b);
    Ok(())
}
