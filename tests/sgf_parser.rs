use go_game::Property;

#[test]
fn test_parse_simple_sgf() {
    let sgf = "(;FF[4]SZ[9]KM[5.5];B[aa];W[bb])";
    let tree = go_game::parse(sgf).expect("parse should succeed");
    let root = tree.get_root().expect("root exists");
    let node = tree.get_node(root).expect("root node");

    // SZ parsed
    assert_eq!(node.get_first(Property::SZ).map(|s| s.as_str()), Some("9"));
    // KM parsed
    assert_eq!(
        node.get_first(Property::KM).map(|s| s.as_str()),
        Some("5.5")
    );

    // 子节点链
    let children = tree.get_children(root);
    assert_eq!(children.len(), 1);
    let n1 = tree.get_node(children[0]).unwrap();
    assert!(n1.contains(Property::B));
    let n1_children = tree.get_children(children[0]);
    assert_eq!(n1_children.len(), 1);
    let n2 = tree.get_node(n1_children[0]).unwrap();
    assert!(n2.contains(Property::W));
}
