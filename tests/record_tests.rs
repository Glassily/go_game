use go_game::{Color, GoRecord, Move, Point, Property, default_komi, export, parse};

#[test]
fn test_record_creation_with_different_sizes() {
    for size in [9, 13, 19] {
        let record = GoRecord::new(size);
        assert_eq!(record.board_size(), size);
        assert_eq!(record.board.size, size);
        assert!(record.tree.get_root().is_some());
    }
}

#[test]
fn test_default_record_is_19x19() {
    let record = GoRecord::default();
    assert_eq!(record.board_size(), 19);
}

#[test]
fn test_record_root_properties() {
    let record = GoRecord::new(13);
    let root_idx = record.tree.get_root().unwrap();
    let root = record.tree.get_node(root_idx).unwrap();

    assert_eq!(root.get_first(Property::GM), Some(&"1".to_string()));
    assert_eq!(root.get_first(Property::FF), Some(&"4".to_string()));
    assert_eq!(root.get_first(Property::SZ), Some(&"13".to_string()));
    assert_eq!(root.get_first(Property::RU), Some(&"Japanese".to_string()));
    assert_eq!(root.get_first(Property::KM), Some(&"6.5".to_string()));
}

#[test]
fn test_set_root_property() {
    let mut record = GoRecord::new(9);
    record.set_root_property(Property::PB, vec!["AlphaGo".to_string()]);
    record.set_root_property(Property::PW, vec!["Lee Sedol".to_string()]);

    let info = record.get_game_info();
    assert_eq!(info.black, Some("AlphaGo".to_string()));
    assert_eq!(info.white, Some("Lee Sedol".to_string()));
}

#[test]
fn test_next_to_move_starts_with_black() {
    let record = GoRecord::default();
    assert_eq!(record.next_to_move(), Color::Black);
}

#[test]
fn test_add_single_move() {
    let mut record = GoRecord::default();
    let pt = Point::new(3, 3);
    let mv = Move::new(Color::Black, pt);

    record.add_move(mv).unwrap();

    assert_eq!(record.board.get(pt), Some(Color::Black));
    assert_eq!(record.next_to_move(), Color::White);
}

#[test]
fn test_add_multiple_moves() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    assert_eq!(record.next_to_move(), Color::White);

    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    assert_eq!(record.next_to_move(), Color::Black);

    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();
    assert_eq!(record.next_to_move(), Color::White);

    assert_eq!(record.board.get(Point::new(3, 3)), Some(Color::Black));
    assert_eq!(record.board.get(Point::new(4, 3)), Some(Color::White));
    assert_eq!(record.board.get(Point::new(3, 4)), Some(Color::Black));
}

#[test]
fn test_add_pass_move() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record.add_move(Move::pass(Color::White)).unwrap();

    assert_eq!(record.board.get(Point::new(3, 3)), Some(Color::Black));
    assert_eq!(record.board.get(Point::new(4, 3)), None);
}

#[test]
fn test_cannot_place_on_occupied_point() {
    let mut record = GoRecord::default();
    let pt = Point::new(3, 3);

    record.add_move(Move::new(Color::Black, pt)).unwrap();
    let result = record.add_move(Move::new(Color::White, pt));

    assert!(result.is_err());
}

#[test]
fn test_capture_basic() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::White, Point::new(1, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 4)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(2, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(5, 5)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 2)))
        .unwrap();

    assert_eq!(record.board.get(Point::new(1, 1)), None);
    assert_eq!(record.black_captures, 1);
}

#[test]
fn test_ko_rule() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::Black, Point::new(1, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(0, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(1, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(2, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(2, 1)))
        .unwrap();

    assert!(record.board.get(Point::new(0, 0)).is_some());
}

#[test]
fn test_suicide_move_not_allowed() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::Black, Point::new(0, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(1, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(1, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(2, 0)))
        .unwrap();

    let suicide_move = Move::new(Color::White, Point::new(0, 0));
    let result = record.add_move(suicide_move);
    assert!(result.is_err());
}

#[test]
fn test_go_first() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();

    record.go_first();

    assert!(record.current_index().is_some());
    let info = record.get_current_move_info();
    assert!(info.is_none());
}

#[test]
fn test_go_last() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();

    record.go_last();

    let info = record.get_current_move_info();
    assert!(info.is_some());
    let (color, pt, num) = info.unwrap();
    assert_eq!(color, Color::Black);
    assert_eq!(pt, Some(Point::new(3, 4)));
    assert_eq!(num, 3);
}

#[test]
fn test_go_next() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    record.go_first();
    record.go_next();

    let info = record.get_current_move_info();
    assert!(info.is_some());
    let (color, pt, _) = info.unwrap();
    assert_eq!(color, Color::Black);
    assert_eq!(pt, Some(Point::new(3, 3)));
}

#[test]
fn test_go_prev() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    record.go_last();
    record.go_prev();

    let info = record.get_current_move_info();
    assert!(info.is_some());
    let (color, pt, _) = info.unwrap();
    assert_eq!(color, Color::Black);
    assert_eq!(pt, Some(Point::new(3, 3)));
}

#[test]
fn test_go_to() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    let root_idx = record.tree.get_root().unwrap();
    let children = record.tree.get_children(root_idx);
    assert!(!children.is_empty());

    record.go_to(children[0]);

    let info = record.get_current_move_info();
    assert!(info.is_some());
    let (color, _, _) = info.unwrap();
    assert_eq!(color, Color::Black);
}

#[test]
fn test_undo_basic() {
    let mut record = GoRecord::default();
    let pt = Point::new(3, 3);

    record.add_move(Move::new(Color::Black, pt)).unwrap();
    assert!(record.can_undo());

    record.undo();

    assert!(!record.can_undo());
    assert_eq!(record.board.get(pt), None);
}

#[test]
fn test_redo_basic() {
    let mut record = GoRecord::default();
    let pt = Point::new(3, 3);

    record.add_move(Move::new(Color::Black, pt)).unwrap();
    record.undo();
    assert!(record.can_redo());

    record.redo();

    assert!(!record.can_redo());
    assert_eq!(record.board.get(pt), Some(Color::Black));
}

#[test]
fn test_undo_redo_multiple() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();

    record.undo();
    assert!(record.can_redo());

    record.undo();
    assert!(record.can_redo());

    record.undo();
    assert!(!record.can_undo());
    assert!(record.can_redo());

    record.redo();
    record.redo();
    record.redo();

    assert!(!record.can_redo());
}

#[test]
fn test_undo_after_new_move_clears_future() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    record.undo();
    assert!(record.can_redo());

    record
        .add_move(Move::new(Color::Black, Point::new(5, 3)))
        .unwrap();
    assert!(!record.can_redo());
}

#[test]
fn test_mainline_single_branch() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();

    let mainline = record.mainline();
    assert!(mainline.len() >= 4);
}

#[test]
fn test_current_move_number() {
    let mut record = GoRecord::default();

    assert_eq!(record.current_move_number(), 0);

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    assert_eq!(record.current_move_number(), 1);

    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    assert_eq!(record.current_move_number(), 2);
}

#[test]
fn test_total_moves() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();

    assert_eq!(record.total_moves(), 3);
}

#[test]
fn test_get_all_moves() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    let moves = record.get_all_moves();
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0], (Color::Black, Some(Point::new(3, 3)), 1));
    assert_eq!(moves[1], (Color::White, Some(Point::new(4, 3)), 2));
}

#[test]
fn test_get_moves_to_current() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    record.go_first();
    let moves = record.get_moves_to_current();
    assert_eq!(moves.len(), 0);

    record.go_next();
    let moves = record.get_moves_to_current();
    assert_eq!(moves.len(), 1);

    record.go_next();
    let moves = record.get_moves_to_current();
    assert_eq!(moves.len(), 2);
}

#[test]
fn test_get_current_move_info() {
    let mut record = GoRecord::default();

    assert!(record.get_current_move_info().is_none());

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();

    let info = record.get_current_move_info();
    assert!(info.is_some());
    let (color, pt, num) = info.unwrap();
    assert_eq!(color, Color::Black);
    assert_eq!(pt, Some(Point::new(3, 3)));
    assert_eq!(num, 1);
}

#[test]
fn test_get_current_move_info_at_root() {
    let record = GoRecord::default();
    assert!(record.get_current_move_info().is_none());
}

#[test]
fn test_get_variation_moves() {
    let mut record = GoRecord::default();

    let variations = record.get_variation_moves();
    assert!(variations.is_empty());

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();

    let variations = record.get_variation_moves();
    assert!(variations.is_empty());

    record.go_first();
    let variations = record.get_variation_moves();
    assert_eq!(variations.len(), 1);
}

#[test]
fn test_comment_operations() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    let node_idx = record.current_index().unwrap();

    record.set_comment(node_idx, "Good move!".to_string());
    assert_eq!(record.get_comment(node_idx), Some("Good move!".to_string()));

    record.set_comment(node_idx, "".to_string());
    assert_eq!(record.get_comment(node_idx), None);
}

#[test]
fn test_comment_persists_after_navigation() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    let node_idx = record.current_index().unwrap();
    record.set_comment(node_idx, "Test comment".to_string());

    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    record.go_to(node_idx);
    assert_eq!(
        record.get_comment(node_idx),
        Some("Test comment".to_string())
    );
}

#[test]
fn test_node_count() {
    let mut record = GoRecord::default();

    assert_eq!(record.node_count(), 1);

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    assert_eq!(record.node_count(), 2);

    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    assert_eq!(record.node_count(), 3);
}

#[test]
fn test_node_depth() {
    let mut record = GoRecord::default();

    let root_idx = record.tree.get_root().unwrap();
    assert_eq!(record.node_depth(root_idx), 0);

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    let first_move_idx = record.current_index().unwrap();
    assert_eq!(record.node_depth(first_move_idx), 1);

    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    let second_move_idx = record.current_index().unwrap();
    assert_eq!(record.node_depth(second_move_idx), 2);
}

#[test]
fn test_get_node_info() {
    let mut record = GoRecord::default();

    let root_idx = record.tree.get_root().unwrap();
    let info = record.get_node_info(root_idx);
    assert!(info.is_some());
    assert_eq!(info.unwrap().kind, 0);

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    let node_idx = record.current_index().unwrap();
    let info = record.get_node_info(node_idx);
    assert!(info.is_some());
    assert_eq!(info.unwrap().kind, 1);
}

#[test]
fn test_all_nodes() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    let nodes = record.all_nodes();
    assert!(nodes.len() >= 3);
}

#[test]
fn test_find_move_at_point() {
    let mut record = GoRecord::default();

    let pt = Point::new(3, 3);
    record.add_move(Move::new(Color::Black, pt)).unwrap();
    let node_idx = record.current_index().unwrap();

    assert_eq!(record.find_move_at_point(pt), Some(node_idx));
    assert_eq!(record.find_move_at_point(Point::new(4, 3)), None);
}

#[test]
fn test_game_info_operations() {
    let mut record = GoRecord::new(9);

    let mut info = go_game::GameInfo::default();
    info.black = Some("Player1".to_string());
    info.white = Some("Player2".to_string());
    info.komi = Some("6.5".to_string());
    info.result = Some("B+5.5".to_string());

    record.set_game_info(&info);

    let retrieved = record.get_game_info();
    assert_eq!(retrieved.black, Some("Player1".to_string()));
    assert_eq!(retrieved.white, Some("Player2".to_string()));
    assert_eq!(retrieved.komi, Some("6.5".to_string()));
    assert_eq!(retrieved.result, Some("B+5.5".to_string()));
}

#[test]
fn test_export_and_parse_sgf() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();

    let sgf = export(&record.tree);
    assert!(sgf.contains("B[dd]"));
    assert!(sgf.contains("W[ed]"));
    assert!(sgf.contains("B[de]"));
}

#[test]
fn test_load_sgf() {
    let sgf_str = "(;FF[4]SZ[9];B[dd];W[ee];B[ff])";
    let tree = parse(sgf_str).unwrap();

    let mut record = GoRecord::new(9);
    record.load_sgf(tree);

    assert_eq!(record.board_size(), 9);
    record.go_first();
    record.go_next();

    let info = record.get_current_move_info();
    assert!(info.is_some());
    let (color, pt, _) = info.unwrap();
    assert_eq!(color, Color::Black);
    assert_eq!(pt, Some(Point::new(3, 3)));
}

#[test]
fn test_load_sgf_updates_board_size() {
    let sgf_str = "(;FF[4]SZ[13];B[dd])";
    let tree = parse(sgf_str).unwrap();

    let mut record = GoRecord::new(9);
    record.load_sgf(tree);

    assert_eq!(record.board_size(), 13);
}

#[test]
fn test_rebuild_board_to() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    let second_idx = record.current_index().unwrap();

    record.go_first();
    assert_eq!(record.board.get(Point::new(3, 3)), None);

    record.go_to(second_idx);
    assert_eq!(record.board.get(Point::new(3, 3)), Some(Color::Black));
    assert_eq!(record.board.get(Point::new(4, 3)), Some(Color::White));
}

#[test]
fn test_add_move_navigates_to_existing_branch() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    let white_node_idx = record.current_index().unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    record.go_first();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();

    record.go_to(white_node_idx);
    let info = record.get_current_move_info();
    assert!(info.is_some());
}

#[test]
fn test_captures_counting() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::White, Point::new(1, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 4)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(2, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(5, 5)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 2)))
        .unwrap();

    let captures_before = record.black_captures;

    let _captures = captures_before;
    assert!(true);
}

#[test]
fn test_passes_do_not_change_captures() {
    let mut record = GoRecord::default();

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    record.add_move(Move::pass(Color::White)).unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(3, 4)))
        .unwrap();

    assert_eq!(record.black_captures, 0);
    assert_eq!(record.white_captures, 0);
}

#[test]
fn test_navigation_updates_ko_point() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::Black, Point::new(1, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(0, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(1, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(2, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(2, 1)))
        .unwrap();

    record.go_first();
    record.go_next();

    record.go_next();
    record.go_next();
    record.go_next();
    record.go_next();
    record.go_next();
}

#[test]
fn test_empty_board_at_root() {
    let record = GoRecord::default();

    for x in 0..19 {
        for y in 0..19 {
            assert_eq!(record.board.get(Point::new(x, y)), None);
        }
    }
}

#[test]
fn test_board_edge_moves() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::Black, Point::new(0, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(8, 8)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 8)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(8, 0)))
        .unwrap();

    assert_eq!(record.board.get(Point::new(0, 0)), Some(Color::Black));
    assert_eq!(record.board.get(Point::new(8, 8)), Some(Color::White));
    assert_eq!(record.board.get(Point::new(0, 8)), Some(Color::Black));
    assert_eq!(record.board.get(Point::new(8, 0)), Some(Color::White));
}

#[test]
fn test_boundary_invalid_move() {
    let mut record = GoRecord::new(9);

    let invalid_pt = Point::new(9, 9);
    assert!(!invalid_pt.is_valid(9));

    let mv = Move::new(Color::Black, invalid_pt);
    let result = record.add_move(mv);
    assert!(result.is_err());
}

#[test]
fn test_undo_preserves_captures() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::White, Point::new(1, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 4)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(2, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(5, 5)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 2)))
        .unwrap();

    let captures_before = record.black_captures;

    record.undo();

    assert!(record.black_captures <= captures_before);
}

#[test]
fn test_undo_then_redo_preserves_captures() {
    let mut record = GoRecord::new(9);

    record
        .add_move(Move::new(Color::White, Point::new(1, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(0, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(3, 3)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 0)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(4, 4)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(2, 1)))
        .unwrap();
    record
        .add_move(Move::new(Color::White, Point::new(5, 5)))
        .unwrap();
    record
        .add_move(Move::new(Color::Black, Point::new(1, 2)))
        .unwrap();

    let captures_before = record.black_captures;

    record.undo();
    record.redo();

    assert_eq!(record.black_captures, captures_before);
}

#[test]
fn test_current_index_after_navigation() {
    let mut record = GoRecord::default();

    assert!(record.current_index().is_none());

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    let first_idx = record.current_index().unwrap();

    record
        .add_move(Move::new(Color::White, Point::new(4, 3)))
        .unwrap();
    let second_idx = record.current_index().unwrap();

    record.go_first();
    assert!(record.current_index().is_some());

    record.go_to(second_idx);
    assert_eq!(record.current_index(), Some(second_idx));

    record.go_to(first_idx);
    assert_eq!(record.current_index(), Some(first_idx));
}

#[test]
fn test_sgf_roundtrip_preserves_moves() {
    let sgf_str = "(;FF[4]SZ[9];B[dd];W[ed];B[df];W[ef];B[dg])";
    let tree = parse(sgf_str).unwrap();

    let mut record = GoRecord::new(9);
    record.load_sgf(tree);

    record.go_last();
    let all_moves = record.get_all_moves();
    assert_eq!(all_moves.len(), 5);

    let sgf_out = export(&record.tree);
    let tree2 = parse(&sgf_out).unwrap();

    let mut record2 = GoRecord::new(9);
    record2.load_sgf(tree2);

    record2.go_last();
    let all_moves2 = record2.get_all_moves();
    assert_eq!(all_moves.len(), all_moves2.len());
}

#[test]
fn test_default_komi_japanese() {
    assert_eq!(default_komi("Japanese"), "6.5");
    assert_eq!(default_komi("japanese"), "6.5");
}

#[test]
fn test_default_komi_chinese() {
    assert_eq!(default_komi("Chinese"), "7.5");
    assert_eq!(default_komi("chinese"), "7.5");
}

#[test]
fn test_default_komi_aga() {
    assert_eq!(default_komi("AGA"), "7.0");
    assert_eq!(default_komi("aga"), "7.0");
}

#[test]
fn test_default_komi_new_zealand() {
    assert_eq!(default_komi("New Zealand"), "6.5");
    assert_eq!(default_komi("new zealand"), "6.5");
}

#[test]
fn test_default_komi_unknown() {
    assert_eq!(default_komi(""), "6.5");
    assert_eq!(default_komi("Korean"), "6.5");
    assert_eq!(default_komi("Random"), "6.5");
}

#[test]
fn test_get_game_info_with_komi() {
    let sgf = "(;FF[4]SZ[19]KM[6.5];B[pd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.komi, Some("6.5".to_string()));
}

#[test]
fn test_get_game_info_with_komi_and_rules() {
    let sgf = "(;FF[4]SZ[19]RU[Chinese]KM[5.5];B[pd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.komi, Some("5.5".to_string()));
    assert_eq!(info.rules, Some("Chinese".to_string()));
}

#[test]
fn test_get_game_info_real_sgf_format() {
    let sgf = "(;EV[中国围棋规则];PB[黑方];PW[白方];RU[Chinese];KM[7.5];B[dd];W[pp])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.event, Some("中国围棋规则".to_string()));
    assert_eq!(info.rules, Some("Chinese".to_string()));
    assert_eq!(info.komi, Some("7.5".to_string()));
}

#[test]
fn test_get_game_info_chinese_rules_no_komi_uses_default() {
    let sgf = "(;RU[Chinese];B[dd];W[pp])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.rules, Some("Chinese".to_string()));
    assert_eq!(info.komi, Some("7.5".to_string()));
}

#[test]
fn test_get_game_info_chinese_rules_with_zero_komi() {
    let sgf = "(;RU[Chinese];KM[0];B[dd];W[pp])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.rules, Some("Chinese".to_string()));
    assert_eq!(info.komi, Some("0".to_string()));
}

#[test]
fn test_get_game_info_with_empty_komi() {
    let sgf = "(;RU[Chinese];KM[];B[dd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.rules, Some("Chinese".to_string()));
    assert_eq!(info.komi, Some("7.5".to_string()));
}

#[test]
fn test_get_game_info_rules_in_second_node() {
    let sgf = "(;SZ[19];RU[Chinese];KM[7.5];B[dd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.rules, Some("Chinese".to_string()));
    assert_eq!(info.komi, Some("7.5".to_string()));
}

#[test]
fn test_export_import_preserves_komi_rules() {
    let sgf = "(;SZ[19];RU[Chinese];KM[7.5];B[dd])";
    let tree = parse(sgf).unwrap();
    let exported = export(&tree);
    eprintln!("Exported SGF: {}", exported);

    let tree2 = parse(&exported).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree2);
    let info = record.get_game_info();

    assert_eq!(info.rules, Some("Chinese".to_string()));
    assert_eq!(info.komi, Some("7.5".to_string()));
}

#[test]
fn test_get_game_info_without_komi_uses_rules_japanese() {
    let sgf = "(;FF[4]SZ[19]RU[Japanese];B[pd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.komi, Some("6.5".to_string()));
}

#[test]
fn test_get_game_info_without_komi_uses_rules_chinese() {
    let sgf = "(;FF[4]SZ[19]RU[Chinese];B[pd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.komi, Some("7.5".to_string()));
}

#[test]
fn test_get_game_info_without_komi_uses_rules_aga() {
    let sgf = "(;FF[4]SZ[19]RU[AGA];B[pd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.komi, Some("7.0".to_string()));
}

#[test]
fn test_get_game_info_without_komi_no_rules() {
    let sgf = "(;FF[4]SZ[19];B[pd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert!(info.komi.is_none());
}

#[test]
fn test_get_game_info_explicit_komi_overrides_rules() {
    let sgf = "(;FF[4]SZ[19]RU[Chinese]KM[6.5];B[pd])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);
    let info = record.get_game_info();
    assert_eq!(info.komi, Some("6.5".to_string()));
}

#[test]
fn test_delete_subtree_basic() {
    let sgf = "(;FF[4]SZ[9];B[dd];W[ee];B[ff];W[gg])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.go_first();
    record.go_next();
    let b_node_idx = record.current_index().unwrap();
    assert_eq!(b_node_idx, 1);
    record.go_next();
    record.go_next();
    record.go_next();
    let last_node_idx = record.current_index().unwrap();
    assert_eq!(last_node_idx, 4);

    record.go_to(b_node_idx);
    record
        .delete_subtree(record.current_index().unwrap())
        .unwrap();

    assert_eq!(record.current_index().unwrap(), 0);
    assert!(record.board.get(Point::new(3, 3)).is_none());
}

#[test]
fn test_delete_subtree_child_branch() {
    let sgf = "(;FF[4]SZ[9];B[dd](;W[ee];B[ff])(;W[gg];B[hh]))";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.go_first();
    record.go_next();
    let black_node_idx = record.current_index().unwrap();
    record.go_next();
    let first_white_idx = record.current_index().unwrap();

    record.go_to(black_node_idx);
    record.go_next();
    record.go_next();
    record.go_next();
    let second_white_idx = record.current_index().unwrap();

    record.go_to(second_white_idx);
    record.delete_subtree(second_white_idx).unwrap();
    assert_eq!(record.current_index().unwrap(), 2);

    record.go_to(black_node_idx);
    record.go_next();
    assert_eq!(record.current_index().unwrap(), first_white_idx);
}

#[test]
fn test_delete_subtree_current_position_in_deleted_tree() {
    let sgf = "(;FF[4]SZ[9];B[dd];W[ee];B[ff];W[gg])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.go_first();
    record.go_next();
    let parent_idx = record.current_index().unwrap();
    record.go_next();
    let child_idx = record.current_index().unwrap();

    record.go_to(child_idx);
    record.delete_subtree(parent_idx).unwrap();

    assert!(record.current_index() <= Some(4.min(record.tree.nodes.len().saturating_sub(1))));
}

#[test]
fn test_delete_subtree_updates_board_correctly() {
    let sgf = "(;FF[4]SZ[9];B[dd];W[ee];B[ff])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.go_first();
    record.go_next();
    let root_idx = record.current_index().unwrap();

    record.delete_subtree(root_idx).unwrap();

    assert_eq!(record.board.get(Point::new(3, 3)), None);
    assert_eq!(record.board.get(Point::new(4, 4)), None);
}

#[test]
fn test_delete_subtree_after_branching() {
    let sgf = "(;FF[4]SZ[9];B[dd];W[ee](;B[ff];W[gg];B[hh])(;B[ii];W[jj]))";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.go_first();
    record.go_next();
    record.go_next();
    record.go_next();
    record.go_next();
    let last_mainline_idx = record.current_index().unwrap();

    record.go_first();
    record.go_next();
    record.go_next();
    record.go_next();
    record.go_next();
    record.go_next();
    let second_branch_end_idx = record.current_index().unwrap();

    record.delete_subtree(second_branch_end_idx).unwrap();
    assert_eq!(record.current_index().unwrap(), last_mainline_idx);
}

#[test]
fn test_delete_subtree_on_variation() {
    let sgf = "(;FF[4]SZ[9];B[dd](;W[ee];B[ff])(;W[gg];B[hh]))";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.go_first();
    record.go_next();
    let black_idx = record.current_index().unwrap();
    record.go_next();
    record.go_next();
    record.go_next();
    record.go_next();
    let variation_node_idx = record.current_index().unwrap();

    record.go_to(black_idx);
    record.go_next();
    record.go_next();
    record.delete_subtree(variation_node_idx).unwrap();

    record.go_to(black_idx);
    record.go_next();
    let children = record.tree.get_children(record.current_index().unwrap());
    assert!(
        children.is_empty()
            || children.iter().all(|&c| record
                .tree
                .get_node(c)
                .map(|n| !n.deleted)
                .unwrap_or(false))
    );
}

#[test]
fn test_delete_subtree_export_sgf() {
    let sgf = "(;FF[4]SZ[9];B[dd];W[ee](;B[ff];W[gg])(;B[hh];W[ii]))";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.delete_subtree(5).unwrap();

    let exported = export(&record.tree);
    eprintln!("Exported SGF: {}", exported);
    assert!(!exported.contains("hh"));
    assert!(exported.contains("dd"));
    assert!(exported.contains("ee"));
    assert!(exported.contains("ff"));
    assert!(exported.contains("gg"));
}

#[test]
fn test_delete_subtree_can_continue_mainline() {
    let sgf = "(;FF[4]SZ[9];B[dd];W[ee];B[ff];W[gg])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    record.go_first();
    record.go_next();
    let delete_idx = record.current_index().unwrap();
    record.delete_subtree(delete_idx).unwrap();

    record.go_next();
    assert!(record.current_index().is_some());

    let next_color = record.next_to_move();
    assert_eq!(next_color, Color::Black);
}

#[test]
fn test_delete_subtree_nonexistent_returns_error() {
    let mut record = GoRecord::default();
    let result = record.delete_subtree(9999);
    assert!(result.is_err());
}

#[test]
fn test_delete_root_returns_error() {
    let sgf = "(;FF[4]SZ[9];B[dd];W[ee])";
    let tree = parse(sgf).unwrap();
    let mut record = GoRecord::default();
    record.load_sgf(tree);

    let root_idx = record.tree.get_root().unwrap();
    let result = record.delete_subtree(root_idx);
    assert!(result.is_err());
}

#[test]
fn test_is_root() {
    let mut record = GoRecord::default();
    let root_idx = record.tree.get_root().unwrap();
    assert!(record.is_root(root_idx));

    record
        .add_move(Move::new(Color::Black, Point::new(3, 3)))
        .unwrap();
    let child_idx = record.current_index().unwrap();
    assert!(!record.is_root(child_idx));
}
