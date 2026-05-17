use go_game::{Color, GoRecord, Move, Point, export};

#[test]
fn test_default_record_creation() {
    let record = GoRecord::default();

    assert_eq!(record.board.size, 19);
    assert_eq!(record.next_to_move(), Color::Black);
    assert!(record.tree.get_root().is_some());
}

#[test]
fn test_default_record_sgf_properties() {
    let record = GoRecord::default();
    let root_idx = record.tree.get_root().unwrap();
    let root = record.tree.get_node(root_idx).unwrap();

    assert!(root.contains(go_game::Property::GM));
    assert!(root.contains(go_game::Property::FF));
    assert!(root.contains(go_game::Property::SZ));
    assert!(root.contains(go_game::Property::RU));
    assert!(root.contains(go_game::Property::KM));
}

#[test]
fn test_add_first_move() {
    let mut record = GoRecord::default();

    assert_eq!(record.next_to_move(), Color::Black);

    let mv = Move::new(Color::Black, Point::new(3, 3));
    let result = record.add_move(mv);
    assert!(result.is_ok(), "First move should be valid");

    assert_eq!(record.next_to_move(), Color::White);
    assert_eq!(record.board.get(Point::new(3, 3)), Some(Color::Black));
}

#[test]
fn test_add_two_moves() {
    let mut record = GoRecord::default();

    let mv1 = Move::new(Color::Black, Point::new(3, 3));
    record.add_move(mv1).unwrap();

    assert_eq!(record.next_to_move(), Color::White);

    let mv2 = Move::new(Color::White, Point::new(4, 3));
    let result = record.add_move(mv2);
    assert!(result.is_ok(), "Second move should be valid");

    assert_eq!(record.next_to_move(), Color::Black);
    assert_eq!(record.board.get(Point::new(4, 3)), Some(Color::White));
}

#[test]
fn test_third_move_should_be_black() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    assert_eq!(
        record.next_to_move(),
        Color::Black,
        "Third move should be Black"
    );
}

#[test]
fn test_export_sgf_after_moves() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    let sgf = export(&record.tree);
    eprintln!("SGF output: {}", sgf);
    assert!(sgf.contains(";B[dd]"), "Should contain black move at dd");
    assert!(sgf.contains(";W[ed]"), "Should contain white move at ed");
}

#[test]
fn test_export_sgf_preserves_root_properties() {
    let record = GoRecord::default();
    let sgf = export(&record.tree);

    assert!(sgf.contains("GM[1]"), "Should contain GM[1]");
    assert!(sgf.contains("FF[4]"), "Should contain FF[4]");
    assert!(sgf.contains("SZ[19]"), "Should contain SZ[19]");
    assert!(sgf.contains("RU[Japanese]"), "Should contain RU[Japanese]");
    assert!(sgf.contains("KM[6.5]"), "Should contain KM[6.5]");
}

#[test]
fn test_turn_alternation_correct() {
    let mut record = GoRecord::default();
    let expected_turns = vec![
        Color::Black,
        Color::White,
        Color::Black,
        Color::White,
        Color::Black,
    ];

    let points = vec![
        Point::new(3, 3),
        Point::new(4, 3),
        Point::new(5, 3),
        Point::new(6, 3),
        Point::new(7, 3),
    ];

    for (i, (expected_color, pt)) in expected_turns.iter().zip(points.iter()).enumerate() {
        assert_eq!(
            record.next_to_move(),
            *expected_color,
            "Turn {} should be {:?}",
            i + 1,
            expected_color
        );

        let mv = Move::new(*expected_color, *pt);
        record.add_move(mv).expect("Move should be valid");
    }
}
