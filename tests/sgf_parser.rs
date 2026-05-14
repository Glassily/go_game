//! SGF 解析与导出测试
//!
//! 测试覆盖：
//! - 基础解析：节点、属性、树结构
//! - 变体（分支）解析：同层分支、嵌套分支
//! - 特殊属性解析：摆子(AB/AW)、元信息(KM/RU/PW/PB)、注释(C)
//! - 导出往返测试：解析 → 导出 → 重新解析
//! - 特殊字符转义

use go_game::GameTree;
use go_game::Property;
use go_game::{export, parse, validate};
use std::collections::HashMap;

// ============================================================================
// 基础解析测试
// ============================================================================

#[test]
fn test_parse_minimal_sgf() {
    let sgf = "(;FF[4]SZ[9])";
    let tree = parse(sgf).expect("parse should succeed");
    assert!(tree.get_root().is_some());
}

#[test]
fn test_parse_basic_properties() {
    let sgf = "(;FF[4]SZ[9]KM[5.5];B[aa];W[bb])";
    let tree = parse(sgf).expect("parse should succeed");

    let root = tree.get_root().expect("root exists");
    let node = tree.get_node(root).expect("root node");

    assert_eq!(node.get_first(Property::SZ).map(|s| s.as_str()), Some("9"));
    assert_eq!(
        node.get_first(Property::KM).map(|s| s.as_str()),
        Some("5.5")
    );

    let children = tree.get_children(root);
    assert_eq!(children.len(), 1);

    let n1 = tree.get_node(children[0]).unwrap();
    assert!(n1.contains(Property::B));
    assert_eq!(n1.get_first(Property::B).map(|s| s.as_str()), Some("aa"));

    let n1_children = tree.get_children(children[0]);
    assert_eq!(n1_children.len(), 1);

    let n2 = tree.get_node(n1_children[0]).unwrap();
    assert!(n2.contains(Property::W));
    assert_eq!(n2.get_first(Property::W).map(|s| s.as_str()), Some("bb"));
}

#[test]
fn test_parse_multi_value_properties() {
    let sgf = "(;FF[4]SZ[9]AB[aa][bb][cc])";
    let tree = parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();
    let node = tree.get_node(root).unwrap();

    let ab_vals = node.get(Property::AB).unwrap();
    assert_eq!(ab_vals.len(), 3);
    assert!(ab_vals.contains(&"aa".to_string()));
    assert!(ab_vals.contains(&"bb".to_string()));
    assert!(ab_vals.contains(&"cc".to_string()));
}

// ============================================================================
// 变体（分支）解析测试
// ============================================================================

/// 树结构：
///   root
///     └── B[aa]
///           ├── W[bb]
///           └── W[cc]
#[test]
fn test_parse_simple_variations() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb])(;W[cc]))";
    let tree = parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();
    let root_children = tree.get_children(root);

    assert_eq!(root_children.len(), 1);
    let b_node = root_children[0];
    assert!(tree.get_node(b_node).unwrap().contains(Property::B));

    let variations = tree.get_children(b_node);
    assert_eq!(variations.len(), 2);

    let v1 = tree.get_node(variations[0]).unwrap();
    assert!(v1.contains(Property::W));
    assert_eq!(v1.get_first(Property::W).map(|s| s.as_str()), Some("bb"));

    let v2 = tree.get_node(variations[1]).unwrap();
    assert!(v2.contains(Property::W));
    assert_eq!(v2.get_first(Property::W).map(|s| s.as_str()), Some("cc"));
}

/// 树结构：
///   root
///     └── B[aa]
///           └── W[bb]
///                 ├── B[cc]
///                 └── B[dd]
#[test]
fn test_parse_variation_in_main_line() {
    let sgf = "(;FF[4]SZ[9];B[aa];W[bb](;B[cc])(;B[dd]))";
    let tree = parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();

    let root_children = tree.get_children(root);
    assert_eq!(root_children.len(), 1);
    assert!(
        tree.get_node(root_children[0])
            .unwrap()
            .contains(Property::B)
    );

    let n1_children = tree.get_children(root_children[0]);
    assert_eq!(n1_children.len(), 1);
    assert!(tree.get_node(n1_children[0]).unwrap().contains(Property::W));

    let variations = tree.get_children(n1_children[0]);
    assert_eq!(variations.len(), 2);

    assert!(tree.get_node(variations[0]).unwrap().contains(Property::B));
    assert_eq!(
        tree.get_node(variations[0])
            .unwrap()
            .get_first(Property::B)
            .map(|s| s.as_str()),
        Some("cc")
    );

    assert!(tree.get_node(variations[1]).unwrap().contains(Property::B));
    assert_eq!(
        tree.get_node(variations[1])
            .unwrap()
            .get_first(Property::B)
            .map(|s| s.as_str()),
        Some("dd")
    );
}

/// 树结构：
///   root
///     └── B[aa]
///           ├── W[bb] → B[cc]
///           ├── W[dd]
///           └── B[ee] → W[ff]
#[test]
fn test_parse_multiple_variations() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb];B[cc])(;W[dd])(;B[ee];W[ff]))";
    let tree = parse(sgf).expect("parse should succeed");

    let b_node = tree.get_children(tree.get_root().unwrap())[0];
    let variations = tree.get_children(b_node);

    assert_eq!(variations.len(), 3);

    let v1 = tree.get_node(variations[0]).unwrap();
    assert!(v1.contains(Property::W));
    assert_eq!(tree.get_children(variations[0]).len(), 1);
    assert!(
        tree.get_node(tree.get_children(variations[0])[0])
            .unwrap()
            .contains(Property::B)
    );

    let v2 = tree.get_node(variations[1]).unwrap();
    assert!(v2.contains(Property::W));
    assert!(tree.get_children(variations[1]).is_empty());

    let v3 = tree.get_node(variations[2]).unwrap();
    assert!(v3.contains(Property::B));
    assert_eq!(tree.get_children(variations[2]).len(), 1);
}

/// 树结构：
///   root
///     └── B[aa]
///           ├── W[bb] → B[cc]
///           └── W[dd] → B[ee] → W[ff]
#[test]
fn test_parse_nested_variations() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb](;B[cc]))(;W[dd];B[ee](;W[ff])))";
    let tree = parse(sgf).expect("parse should succeed");

    let b_node = tree.get_children(tree.get_root().unwrap())[0];
    let variations = tree.get_children(b_node);

    assert_eq!(variations.len(), 2);

    let v1 = variations[0];
    assert!(tree.get_node(v1).unwrap().contains(Property::W));
    assert_eq!(tree.get_children(v1).len(), 1);
    assert!(
        tree.get_node(tree.get_children(v1)[0])
            .unwrap()
            .contains(Property::B)
    );

    let v2 = variations[1];
    assert!(tree.get_node(v2).unwrap().contains(Property::W));
    let v2_children = tree.get_children(v2);
    assert_eq!(v2_children.len(), 1);

    let v2_c1 = tree.get_node(v2_children[0]).unwrap();
    assert!(v2_c1.contains(Property::B));
    let v2_c1_children = tree.get_children(v2_children[0]);
    assert_eq!(v2_c1_children.len(), 1);

    assert!(
        tree.get_node(v2_c1_children[0])
            .unwrap()
            .contains(Property::W)
    );
}

/// 树结构：
///   root
///     └── B[aa]
///           ├── W[bb] → B[cc] → W[dd] → (B[ee], B[ff])
///           └── W[gg]
#[test]
fn test_parse_deep_nesting() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb](;B[cc](;W[dd](;B[ee])(;B[ff])))(;W[gg]))(;W[hh]))";
    let tree = parse(sgf).expect("parse should succeed");

    let b_node = tree.get_children(tree.get_root().unwrap())[0];
    let level1 = tree.get_children(b_node);

    assert_eq!(level1.len(), 2);

    let left = level1[0];
    assert!(tree.get_node(left).unwrap().contains(Property::W));

    let right = level1[1];
    let right_data = tree.get_node(right).unwrap();
    assert!(right_data.contains(Property::W));
    assert_eq!(
        right_data.get_first(Property::W).map(|s| s.as_str()),
        Some("hh")
    );
}

// ============================================================================
// 特殊属性解析测试
// ============================================================================

#[test]
fn test_parse_setup_with_variations() {
    let sgf = "(;FF[4]SZ[9]AB[aa][bb]AW[cc];B[dd](;W[ee];B[ff])(;W[gg]))";
    let tree = parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();
    let root_node = tree.get_node(root).unwrap();

    assert!(root_node.contains(Property::AB));
    assert!(root_node.contains(Property::AW));
    let ab_vals = root_node.get(Property::AB).unwrap();
    assert!(ab_vals.contains(&"aa".to_string()));
    assert!(ab_vals.contains(&"bb".to_string()));

    let b_node = tree.get_children(root)[0];
    assert!(tree.get_node(b_node).unwrap().contains(Property::B));

    let variations = tree.get_children(b_node);
    assert_eq!(variations.len(), 2);

    assert!(tree.get_node(variations[0]).unwrap().contains(Property::W));
    assert_eq!(tree.get_children(variations[0]).len(), 1);
    assert!(
        tree.get_node(tree.get_children(variations[0])[0])
            .unwrap()
            .contains(Property::B)
    );

    assert!(tree.get_node(variations[1]).unwrap().contains(Property::W));
}

#[test]
fn test_parse_meta_properties() {
    let sgf = "(;FF[4]SZ[9]KM[6.5]RU[Japanese]PW[Black]PB[White])";
    let tree = parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();
    let node = tree.get_node(root).unwrap();

    assert_eq!(
        node.get_first(Property::KM).map(|s| s.as_str()),
        Some("6.5")
    );
    assert_eq!(
        node.get_first(Property::RU).map(|s| s.as_str()),
        Some("Japanese")
    );
    assert_eq!(
        node.get_first(Property::PW).map(|s| s.as_str()),
        Some("Black")
    );
    assert_eq!(
        node.get_first(Property::PB).map(|s| s.as_str()),
        Some("White")
    );
}

#[test]
fn test_parse_comment_in_variation() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb]C[comment in var1])(;W[cc]C[comment in var2]))";
    let tree = parse(sgf).expect("parse should succeed");

    let b_node = tree.get_children(tree.get_root().unwrap())[0];
    let variations = tree.get_children(b_node);

    assert_eq!(variations.len(), 2);

    let v1 = tree.get_node(variations[0]).unwrap();
    assert!(v1.contains(Property::W));
    assert!(v1.contains(Property::C));

    let v2 = tree.get_node(variations[1]).unwrap();
    assert!(v2.contains(Property::W));
    assert!(v2.contains(Property::C));
}

// ============================================================================
// 解析错误测试
// ============================================================================

#[test]
fn test_parse_invalid_property() {
    let sgf = "(;FF[4]SZ[9];X[aa])";
    let r = parse(sgf);
    assert!(r.is_ok());
}

#[test]
fn test_unterminated_value() {
    let sgf = "(;FF[4]SZ[9];B[aa];W[bb";
    let r = parse(sgf);
    assert!(r.is_err());
}

// ============================================================================
// 导出与往返测试
// ============================================================================

#[test]
fn test_export_roundtrip() {
    let sgf = "(;FF[4]SZ[9]KM[6.5];B[aa];W[bb])";
    let tree = parse(sgf).expect("parse should succeed");
    let exported = export(&tree);
    let reparsed = parse(&exported).expect("reparse exported should succeed");
    assert_eq!(tree.nodes.len(), reparsed.nodes.len());
}

#[test]
fn test_complex_sgf_roundtrip_and_validate() {
    let sgf = "(;FF[4]SZ[9]KM[6.5]AB[aa][ii]C[Root comment];W[cc];B[dd](;B[ee]C[var1];W[ff])(;B[gg]C[var2];W[hh]))";

    let tree = parse(sgf).expect("parse complex sgf");

    let root = tree.get_root().expect("root exists");
    let root_node = tree.get_node(root).unwrap();
    assert!(root_node.contains(Property::AB));
    assert!(root_node.contains(Property::C));

    let exported = export(&tree);
    let reparsed = parse(&exported).expect("reparse exported sgf");
    assert!(reparsed.get_root().is_some());
    assert!(exported.contains("AB[") && exported.contains('('));

    let _vr = validate(&tree);
}

#[test]
fn test_ae_and_aw_setup_behavior() {
    let sgf = "(;FF[4]SZ[9]AB[aa][bb]AE[aa]AW[cc];B[dd])";
    let tree = parse(sgf).expect("parse ae/aw");
    let root = tree.get_root().unwrap();
    let node = tree.get_node(root).unwrap();
    assert!(node.contains(Property::AB));
    assert!(node.contains(Property::AE));
    assert!(node.contains(Property::AW));

    let exported = export(&tree);
    let reparsed = parse(&exported).expect("reparse exported");
    assert_eq!(reparsed.nodes.len(), tree.nodes.len());
}

#[test]
fn test_nested_variations_structure() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb](;B[cc])))";
    let tree = parse(sgf).expect("parse nested variations");
    assert!(tree.nodes.len() >= 2);
    let has_move = tree
        .nodes
        .iter()
        .any(|n| n.contains(Property::B) || n.contains(Property::W));
    assert!(has_move);
}

#[test]
fn test_export_escape_chars() {
    let mut props = HashMap::new();
    props.insert(Property::C, vec!["a]b\nc\\d".into()]);
    let tree = GameTree::with_root(props);
    let out = export(&tree);
    assert!(out.contains("\\]") || out.contains("\\n") || out.contains("\\\\"));
}
