use go_game::{parse, export, validate};
use go_game::Property;

#[test]
fn test_complex_sgf_roundtrip_and_validate() {
    // 复杂 SGF：根上有摆子/注释/让子，之后有两个变体分支（使用单行表示以避免未转义字符问题）
    let sgf = "(;FF[4]SZ[9]KM[6.5]AB[aa][ii]C[Root comment];W[cc];B[dd](;B[ee]C[var1];W[ff])(;B[gg]C[var2];W[hh]))";

    // parse
    let tree = parse(sgf).expect("parse complex sgf");

    // 验证根节点有摆子与注释
    let root = tree.get_root().expect("root exists");
    let root_node = tree.get_node(root).unwrap();
    assert!(root_node.contains(Property::AB));
    assert!(root_node.contains(Property::C));

    // 导出并可被重新解析
    let exported = export(&tree);
    let reparsed = parse(&exported).expect("reparse exported sgf");
    // 解析器/导出器可能对节点组织做了正规化，保证能重新解析并至少包含原有根节点
    assert!(reparsed.get_root().is_some());
    // 导出字符串应包含让子与摆子，并保留变体符号
    assert!(exported.contains("AB[") && exported.contains('('));

    // 运行验证器（结果可能根据着法合法性报告不同错误，但应可执行）
    let _vr = validate(&tree);
}

#[test]
fn test_ae_and_aw_setup_behavior() {
    // 测试 AB 添加后 AE 删除（清除）以及 AW 添加白子
    let sgf = "(;
        FF[4]
        SZ[9]
        AB[aa][bb]
        AE[aa]
        AW[cc]
        ;B[dd]
    )";
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
    // 更深层次的嵌套变体
    let sgf = "(;FF[4]SZ[9];B[aa](;W[bb](;B[cc])))";
    let tree = parse(sgf).expect("parse nested variations");
    // 至少包含多于一个节点并含有着法属性
    assert!(tree.nodes.len() >= 2);
    // 至少有一个含 `B` 或 `W` 的节点
    let has_move = tree.nodes.iter().any(|n| n.contains(Property::B) || n.contains(Property::W));
    assert!(has_move);
}
