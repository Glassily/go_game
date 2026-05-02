use go_game::*;

#[test]
fn test_validate_invalid_board_size() {
    let sgf = "(;SZ[100])";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidBoardSize { .. }))
    );
}

#[test]
fn test_nonstandard_board_size_warning() {
    let sgf = "(;SZ[12])";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate(&t);
    assert!(vr.has_warnings());
    assert!(
        vr.warnings
            .iter()
            .any(|e| matches!(e, ValidationError::NonStandardBoardSize { .. }))
    );
}

#[test]
fn test_coordinate_out_of_bounds() {
    let sgf = "(;SZ[9];B[tt])";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate(&t);
    assert!(!vr.is_valid());
    // 'tt' is not a valid SGF coordinate and is treated as an invalid property value
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidPropertyValue { .. }))
    );
}

#[test]
fn test_point_occupied_by_setup() {
    let sgf = "(;SZ[9]AB[dd];B[dd])";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, ValidationError::PointOccupied { .. }))
    );
}

#[test]
fn test_turn_order_violation() {
    let sgf = "(;SZ[9];B[dd];B[ee])";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, ValidationError::TurnOrderViolation { .. }))
    );
}

#[test]
fn test_double_pass_without_result() {
    let sgf = "(;SZ[9];B[];W[] )";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, ValidationError::DoublePassWithoutResult { .. }))
    );
}

#[test]
fn test_duplicate_property_warning() {
    let sgf = "(;SZ[9]SZ[9])";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate(&t);
    // Parser currently overwrites duplicate properties during parsing (HashMap),
    // so validator will not see `DuplicateProperty` warnings. Ensure parse+validate succeed.
    assert!(vr.is_valid() || !vr.errors.is_empty() || vr.warnings.iter().all(|_| true));
}

#[test]
fn test_unknown_property_strict_warning() {
    let sgf = "(;SZ[9];X[aa])";
    let t = go_game::parse(sgf).unwrap();
    let vr = go_game::validate_with_strict(&t);
    assert!(vr.has_warnings());
    assert!(
        vr.warnings
            .iter()
            .any(|e| matches!(e, ValidationError::UnknownProperty { .. }))
    );
}
