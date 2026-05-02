use go_game::GameTree;
use go_game::Property;
use std::collections::HashMap;

#[test]
fn test_bfs_iter() {
    let mut t = GameTree::new();
    let mut root_props = HashMap::new();
    root_props.insert(Property::SZ, vec!["9".into()]);
    let root = t.add_node(None, root_props).unwrap();

    let mut a = HashMap::new();
    a.insert(Property::B, vec!["aa".into()]);
    let a_idx = t.add_node(Some(root), a).unwrap();
    let mut b = HashMap::new();
    b.insert(Property::W, vec!["bb".into()]);
    let b_idx = t.add_node(Some(root), b).unwrap();

    // BFS should visit root, then children a and b (in insertion order)
    let mut it = t.bfs_iter();
    assert_eq!(it.next().unwrap(), root);
    let next1 = it.next().unwrap();
    let next2 = it.next().unwrap();
    assert!((next1 == a_idx && next2 == b_idx) || (next1 == b_idx && next2 == a_idx));
}
