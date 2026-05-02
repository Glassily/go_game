
#[test]
fn test_parse_invalid_property() {
    let sgf = "(;FF[4]SZ[9];X[aa])";
    let r = go_game::parse(sgf);
    // Parser now accepts unknown properties as `Other(...)`, so parsing should succeed
    assert!(r.is_ok());
}

#[test]
fn test_unterminated_value() {
    let sgf = "(;FF[4]SZ[9];B[aa];W[bb";
    let r = go_game::parse(sgf);
    assert!(r.is_err());
}
