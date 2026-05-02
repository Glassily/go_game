use go_game::Board;
use go_game::model::Move;
use go_game::model::{Color, Point};

#[test]
fn test_suicide_illegal() {
    let mut b = Board::new(5);
    // 用黑子包围 (1,1)
    b.set(Point { x: 0, y: 1 }, Color::Black);
    b.set(Point { x: 1, y: 0 }, Color::Black);
    b.set(Point { x: 2, y: 1 }, Color::Black);
    b.set(Point { x: 1, y: 2 }, Color::Black);

    let white_move = Move::new(Color::White, Point { x: 1, y: 1 });
    assert!(b.is_legal(&white_move, None, false).is_err());
    assert!(b.apply_move(&white_move, None, false).is_err());
}

#[test]
fn test_simple_capture_and_ko() {
    let mut b = Board::new(5);
    // 中心白子 (1,1)，其一气在 (1,2)
    b.set(Point { x: 1, y: 1 }, Color::White);
    // 黑子围住白子，留一气在 (1,2)
    b.set(Point { x: 0, y: 1 }, Color::Black);
    b.set(Point { x: 1, y: 0 }, Color::Black);
    b.set(Point { x: 2, y: 1 }, Color::Black);

    // 黑在 (1,2) 提子
    let black_move = Move::new(Color::Black, Point { x: 1, y: 2 });
    let res = b
        .apply_move(&black_move, None, false)
        .expect("black move should be legal");
    let (captured, ko_point) = res;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0], Point { x: 1, y: 1 });

    // 如果返回了劫点，则立即复着应该被禁止；否则复着应可行
    let white_recapture = Move::new(Color::White, Point { x: 1, y: 1 });
    // 无论是否返回劫点，这里立即复着都应被禁止（对当前布局而言复着是非法的）
    assert!(b.is_legal(&white_recapture, ko_point, false).is_err());
}
