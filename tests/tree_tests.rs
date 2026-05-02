use std::collections::HashMap;
use go_game::GameTree;
use go_game::Property;

#[test]
fn test_tree_add_and_iter() {
    let mut t = GameTree::new();
    let mut data = HashMap::new();
    data.insert(Property::SZ, vec!["9".into()]);
    let root = t.add_node(None, data).expect("add root");
    assert_eq!(t.get_root(), Some(root));

    // add child
    let mut d2 = HashMap::new();
    d2.insert(Property::B, vec!["aa".into()]);
    let c1 = t.add_node(Some(root), d2).expect("add child");
    assert_eq!(t.get_children(root).len(), 1);

    // preorder should visit root then child
    let mut it = t.preorder_iter();
    let first = it.next().unwrap();
    let second = it.next().unwrap();
    assert_eq!(first, root);
    assert_eq!(second, c1);
}
