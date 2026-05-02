use go_game::Board;
use go_game::model::{Point, Color, Move};

#[test]
fn test_neighbors_and_block_and_liberties() {
    let b = Board::new(3);
    let nbs = b.neighbors(Point { x: 0, y: 0 });
    assert!(nbs.contains(&Point { x: 1, y: 0 }));
    assert!(nbs.contains(&Point { x: 0, y: 1 }));

    let mut b2 = Board::new(5);
    // create a 2-stone block
    b2.set(Point { x: 1, y: 1 }, Color::Black);
    b2.set(Point { x: 2, y: 1 }, Color::Black);
    let block = b2.get_block(Point { x: 1, y: 1 });
    assert_eq!(block.len(), 2);
    let libs = b2.count_liberties(&block);
    assert!(libs.len() >= 2);
}

#[test]
fn test_remove_dead_groups() {
    let mut b = Board::new(5);
    // 白子在 (1,1)
    b.set(Point { x: 1, y: 1 }, Color::White);
    // 黑子包围（但不放在提子点）
    b.set(Point { x: 0, y: 1 }, Color::Black);
    b.set(Point { x: 1, y: 0 }, Color::Black);
    b.set(Point { x: 2, y: 1 }, Color::Black);
    // 此时白子还有一气在 (1,2)
    // 放黑在 (1,2)
    let mv = Move::new(Color::Black, Point { x: 1, y: 2 });
    b.set(Point { x: 1, y: 2 }, Color::Black);
    let removed = b.remove_dead_groups(&mv);
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0], Point { x: 1, y: 1 });
}
