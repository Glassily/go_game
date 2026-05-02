#[test]
fn test_export_roundtrip() {
    let sgf = "(;FF[4]SZ[9]KM[6.5];B[aa];W[bb])";
    let tree = go_game::parse(sgf).expect("parse should succeed");
    let exported = go_game::export(&tree);
    // 导出的字符串应可再次解析
    let reparsed = go_game::parse(&exported).expect("reparse exported should succeed");
    // 比较节点数量
    assert_eq!(tree.nodes.len(), reparsed.nodes.len());
}
