//! SGF 验证器测试
//!
//! 测试覆盖：
//! - 基础验证（合法着法、非法着法）
//! - 棋盘大小验证
//! - 坐标越界检测
//! - 劫（Ko）规则
//! - 禁止自杀着
//! - 落子顺序验证

use go_game::{parse, validate, validate_with_strict};

/// 测试合法棋谱的基本验证
#[test]
fn test_validator_basic() {
    let sgf = "(;FF[4]SZ[9];B[aa];W[bb])";
    let res = validate(&parse(sgf).unwrap());
    assert!(res.is_valid());
}

#[test]
fn test_validate_invalid_board_size() {
    let sgf = "(;SZ[100])";
    let t = parse(sgf).unwrap();
    let vr = validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, go_game::ValidationError::InvalidBoardSize { .. }))
    );
}

#[test]
fn test_nonstandard_board_size_warning() {
    let sgf = "(;SZ[12])";
    let t = parse(sgf).unwrap();
    let vr = validate(&t);
    assert!(vr.has_warnings());
    assert!(
        vr.warnings
            .iter()
            .any(|e| matches!(e, go_game::ValidationError::NonStandardBoardSize { .. }))
    );
}

#[test]
fn test_coordinate_out_of_bounds() {
    let sgf = "(;SZ[9];B[tt])";
    let t = parse(sgf).unwrap();
    let vr = validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, go_game::ValidationError::InvalidPropertyValue { .. }))
    );
}

#[test]
fn test_turn_order_violation() {
    let sgf = "(;FF[4]SZ[9];B[aa];B[bb])";
    let tree = parse(sgf).unwrap();
    let res = validate(&tree);
    assert!(!res.is_valid());
}

#[test]
fn test_point_occupied_by_setup() {
    let sgf = "(;SZ[9]AB[dd];B[dd])";
    let t = parse(sgf).unwrap();
    let vr = validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, go_game::ValidationError::PointOccupied { .. }))
    );
}

#[test]
fn test_double_pass_without_result() {
    let sgf = "(;SZ[9];B[];W[] )";
    let t = parse(sgf).unwrap();
    let vr = validate(&t);
    assert!(!vr.is_valid());
    assert!(
        vr.errors
            .iter()
            .any(|e| matches!(e, go_game::ValidationError::DoublePassWithoutResult { .. }))
    );
}

#[test]
fn test_validator_detects_simple_ko_violation() {
    let sgf = "(;FF[4]SZ[5]AW[bb]AB[ab][ba][cb];B[bc];W[bb])";
    let tree = parse(sgf).expect("parse sgf");
    let res = validate(&tree);
    assert!(
        !res.is_valid(),
        "Validator should flag illegal ko recapture"
    );
}

#[test]
fn test_validator_rejects_suicide() {
    let sgf = "(;FF[4]SZ[5]AB[ab][ba][cb][bc];W[bb])";
    let tree = parse(sgf).expect("parse sgf");
    let res = validate(&tree);
    assert!(
        !res.is_valid(),
        "Validator should reject suicide moves by default"
    );
}

#[test]
fn test_unknown_property_strict_warning() {
    let sgf = "(;SZ[9];X[aa])";
    let t = parse(sgf).unwrap();
    let vr = validate_with_strict(&t);
    assert!(vr.has_warnings());
    assert!(
        vr.warnings
            .iter()
            .any(|e| matches!(e, go_game::ValidationError::UnknownProperty { .. }))
    );
}
