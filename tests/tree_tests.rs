//! GameTree 数据结构测试
//!
//! 测试覆盖：
//! - 节点添加与属性存储
//! - 前序遍历（preorder_iter）
//! - 广度优先遍历（bfs_iter）

use go_game::GameTree;
use go_game::Property;
use std::collections::HashMap;

#[test]
fn test_tree_add_and_iter() {
    let mut t = GameTree::new();
    let mut data = HashMap::new();
    data.insert(Property::SZ, vec!["9".into()]);
    let root = t.add_node(None, data).expect("add root");
    assert_eq!(t.get_root(), Some(root));

    let mut d2 = HashMap::new();
    d2.insert(Property::B, vec!["aa".into()]);
    let c1 = t.add_node(Some(root), d2).expect("add child");
    assert_eq!(t.get_children(root).len(), 1);

    let mut it = t.preorder_iter();
    let first = it.next().unwrap();
    let second = it.next().unwrap();
    assert_eq!(first, root);
    assert_eq!(second, c1);
}

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

    let mut it = t.bfs_iter();
    assert_eq!(it.next().unwrap(), root);
    let next1 = it.next().unwrap();
    let next2 = it.next().unwrap();
    assert!((next1 == a_idx && next2 == b_idx) || (next1 == b_idx && next2 == a_idx));
}
