#[test]
fn test_validator_detects_simple_ko_violation() {
    // root: white at bb, black at ab, ba, cb (so bb has a liberty at bc)
    // black plays at bc capturing white at bb -> creates ko at bb
    // white immediately plays at bb -> should be illegal due to ko
    let sgf = "(;FF[4]SZ[5]AW[bb]AB[ab][ba][cb];B[bc];W[bb])";
    let tree = go_game::parse(sgf).expect("parse sgf");
    let res = go_game::validate(&tree);
    assert!(!res.is_valid(), "Validator should flag illegal ko recapture");
}

#[test]
fn test_validator_rejects_suicide() {
    // root: black stones surround bb (ab, ba, cb, bc), white plays bb suicide
    let sgf = "(;FF[4]SZ[5]AB[ab][ba][cb][bc];W[bb])";
    let tree = go_game::parse(sgf).expect("parse sgf");
    let res = go_game::validate(&tree);
    assert!(!res.is_valid(), "Validator should reject suicide moves by default");
}
