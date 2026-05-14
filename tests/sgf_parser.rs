//! SGF 解析器单元测试
//!
//! 测试覆盖：
//! - 基础解析：节点、属性、树结构
//! - 变体（分支）解析：同层分支、嵌套分支
//! - 特殊属性解析：摆子(AB/AW)、元信息(KM/RU/PW/PB)、注释(C)

use go_game::Property;

// ============================================================================
// 基础解析测试
// ============================================================================

/// 测试解析最简单的 SGF：只有一个 GameTree 包含根节点和后续着法链
#[test]
fn test_parse_minimal_sgf() {
    let sgf = "(;FF[4]SZ[9])";
    let tree = go_game::parse(sgf).expect("parse should succeed");
    assert!(tree.get_root().is_some());
}

/// 测试解析包含基本属性的 SGF：棋盘大小、贴目、基础着法
#[test]
fn test_parse_basic_properties() {
    let sgf = "(;FF[4]SZ[9]KM[5.5];B[aa];W[bb])";
    let tree = go_game::parse(sgf).expect("parse should succeed");

    let root = tree.get_root().expect("root exists");
    let node = tree.get_node(root).expect("root node");

    assert_eq!(node.get_first(Property::SZ).map(|s| s.as_str()), Some("9"));
    assert_eq!(
        node.get_first(Property::KM).map(|s| s.as_str()),
        Some("5.5")
    );

    // 验证着法链结构
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

/// 测试解析带有多值属性的 SGF（如 AB 添加多个黑子）
#[test]
fn test_parse_multi_value_properties() {
    let sgf = "(;FF[4]SZ[9]AB[aa][bb][cc])";
    let tree = go_game::parse(sgf).expect("parse should succeed");

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

/// 测试解析简单变体：
/// 树结构：
///   root
///     └── B[aa]
///           ├── W[bb]
///           └── W[cc]
#[test]
fn test_parse_simple_variations() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb])(;W[cc]))";
    let tree = go_game::parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();
    let root_children = tree.get_children(root);

    // root 有 1 个子节点 B[aa]
    assert_eq!(root_children.len(), 1);
    let b_node = root_children[0];
    assert!(tree.get_node(b_node).unwrap().contains(Property::B));

    // B[aa] 有 2 个变体子节点
    let variations = tree.get_children(b_node);
    assert_eq!(variations.len(), 2);

    // 变体 1: W[bb]
    let v1 = tree.get_node(variations[0]).unwrap();
    assert!(v1.contains(Property::W));
    assert_eq!(v1.get_first(Property::W).map(|s| s.as_str()), Some("bb"));

    // 变体 2: W[cc]
    let v2 = tree.get_node(variations[1]).unwrap();
    assert!(v2.contains(Property::W));
    assert_eq!(v2.get_first(Property::W).map(|s| s.as_str()), Some("cc"));
}

/// 测试解析主线中间的变体：
/// 树结构：
///   root
///     └── B[aa]
///           └── W[bb]
///                 ├── B[cc]
///                 └── B[dd]
#[test]
fn test_parse_variation_in_main_line() {
    let sgf = "(;FF[4]SZ[9];B[aa];W[bb](;B[cc])(;B[dd]))";
    let tree = go_game::parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();

    // 验证主线着法链
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

    // 验证 W[bb] 后的两个变体
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

/// 测试解析多值变体（超过 2 个分支）：
///   root
///     └── B[aa]
///           ├── W[bb] → B[cc]
///           ├── W[dd]
///           └── B[ee] → W[ff]
#[test]
fn test_parse_multiple_variations() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb];B[cc])(;W[dd])(;B[ee];W[ff]))";
    let tree = go_game::parse(sgf).expect("parse should succeed");

    let b_node = tree.get_children(tree.get_root().unwrap())[0];
    let variations = tree.get_children(b_node);

    assert_eq!(variations.len(), 3);

    // 变体 1: W[bb] → B[cc]
    let v1 = tree.get_node(variations[0]).unwrap();
    assert!(v1.contains(Property::W));
    assert_eq!(tree.get_children(variations[0]).len(), 1);
    assert!(
        tree.get_node(tree.get_children(variations[0])[0])
            .unwrap()
            .contains(Property::B)
    );

    // 变体 2: W[dd]（单步变体，无后续）
    let v2 = tree.get_node(variations[1]).unwrap();
    assert!(v2.contains(Property::W));
    assert!(tree.get_children(variations[1]).is_empty());

    // 变体 3: B[ee] → W[ff]
    let v3 = tree.get_node(variations[2]).unwrap();
    assert!(v3.contains(Property::B));
    assert_eq!(tree.get_children(variations[2]).len(), 1);
}

/// 测试解析嵌套变体（变体中再包含变体）：
///   root
///     └── B[aa]
///           ├── W[bb] → B[cc]
///           └── W[dd] → B[ee] → W[ff]
#[test]
fn test_parse_nested_variations() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb](;B[cc]))(;W[dd];B[ee](;W[ff])))";
    let tree = go_game::parse(sgf).expect("parse should succeed");

    let b_node = tree.get_children(tree.get_root().unwrap())[0];
    let variations = tree.get_children(b_node);

    assert_eq!(variations.len(), 2);

    // 变体 1: 浅层嵌套 W[bb] → B[cc]
    let v1 = variations[0];
    assert!(tree.get_node(v1).unwrap().contains(Property::W));
    assert_eq!(tree.get_children(v1).len(), 1);
    assert!(
        tree.get_node(tree.get_children(v1)[0])
            .unwrap()
            .contains(Property::B)
    );

    // 变体 2: 深层嵌套 W[dd] → B[ee] → W[ff]
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

/// 测试解析深层嵌套变体（4+ 层）：
///   root
///     └── B[aa]
///           ├── W[bb] → B[cc] → W[dd] → (B[ee], B[ff])
///           └── W[gg]
#[test]
fn test_parse_deep_nesting() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb](;B[cc](;W[dd](;B[ee])(;B[ff])))(;W[gg]))(;W[hh]))";
    let tree = go_game::parse(sgf).expect("parse should succeed");

    let b_node = tree.get_children(tree.get_root().unwrap())[0];
    let level1 = tree.get_children(b_node);

    assert_eq!(level1.len(), 2);

    // 左分支：W[bb] → ...
    let left = level1[0];
    assert!(tree.get_node(left).unwrap().contains(Property::W));

    // 右分支：W[hh]
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

/// 测试解析带摆子（AB/AW）的 SGF 并带变体
#[test]
fn test_parse_setup_with_variations() {
    let sgf = "(;FF[4]SZ[9]AB[aa][bb]AW[cc];B[dd](;W[ee];B[ff])(;W[gg]))";
    let tree = go_game::parse(sgf).expect("parse should succeed");

    let root = tree.get_root().unwrap();
    let root_node = tree.get_node(root).unwrap();

    // 验证摆子属性
    assert!(root_node.contains(Property::AB));
    assert!(root_node.contains(Property::AW));
    let ab_vals = root_node.get(Property::AB).unwrap();
    assert!(ab_vals.contains(&"aa".to_string()));
    assert!(ab_vals.contains(&"bb".to_string()));

    // 验证变体结构
    let b_node = tree.get_children(root)[0];
    assert!(tree.get_node(b_node).unwrap().contains(Property::B));

    let variations = tree.get_children(b_node);
    assert_eq!(variations.len(), 2);

    // 变体 1: W[ee] → B[ff]
    assert!(tree.get_node(variations[0]).unwrap().contains(Property::W));
    assert_eq!(tree.get_children(variations[0]).len(), 1);
    assert!(
        tree.get_node(tree.get_children(variations[0])[0])
            .unwrap()
            .contains(Property::B)
    );

    // 变体 2: W[gg]
    assert!(tree.get_node(variations[1]).unwrap().contains(Property::W));
}

/// 测试解析元信息属性（KM、RU、PW、PB）
#[test]
fn test_parse_meta_properties() {
    let sgf = "(;FF[4]SZ[9]KM[6.5]RU[Japanese]PW[Black]PB[White])";
    let tree = go_game::parse(sgf).expect("parse should succeed");

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

/// 测试解析带注释（C 属性）的变体
#[test]
fn test_parse_comment_in_variation() {
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb]C[comment in var1])(;W[cc]C[comment in var2]))";
    let tree = go_game::parse(sgf).expect("parse should succeed");

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
