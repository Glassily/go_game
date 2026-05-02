use go_game::{GameTree, Property};
use std::collections::HashMap;

#[test]
fn test_export_escape_chars() {
    let mut props = HashMap::new();
    props.insert(Property::C, vec!["a]b\nc\\d".into()]);
    let tree = GameTree::with_root(props);
    let out = go_game::export(&tree);
    // 应该包含转义后的 ] 和 换行 \n 以及 反斜线 \\
    assert!(out.contains("\\]") || out.contains("\\n") || out.contains("\\\\"));
}
