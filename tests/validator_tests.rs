
#[test]
fn test_validator_basic() {
    // valid game
    let sgf = "(;FF[4]SZ[9];B[aa];W[bb])";
    let res = go_game::validate(&go_game::parse(sgf).unwrap());
    assert!(res.is_valid());

    // turn order violation: B then B
    let sgf2 = "(;FF[4]SZ[9];B[aa];B[bb])";
    let tree2 = go_game::parse(sgf2).unwrap();
    let res2 = go_game::validate(&tree2);
    assert!(!res2.is_valid());

    // occupied point
    let sgf3 = "(;FF[4]SZ[9]AB[aa];B[aa])";
    let tree3 = go_game::parse(sgf3).unwrap();
    let res3 = go_game::validate(&tree3);
    assert!(!res3.is_valid());

    // double pass without result
    let sgf4 = "(;FF[4]SZ[9];B[];W[] )";
    let tree4 = go_game::parse(sgf4).unwrap();
    let res4 = go_game::validate(&tree4);
    assert!(!res4.is_valid());
}
