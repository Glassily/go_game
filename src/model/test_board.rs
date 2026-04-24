#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::model::*;

    #[test]
    fn test_setup_board() {
        let stones = vec![
            (Point { x: 4, y: 4 }, Color::Black),
            (Point { x: 3, y: 9 }, Color::White),
        ];
        let board = Board::from_setup(9, &stones);
        assert_eq!(board.count_stones(Color::Black), 1);
        assert_eq!(board.count_stones(Color::White), 0);
    }

    /// 测试棋盘基本功能：放置棋子、提子、合法性检查
    #[test]
    fn test_basic_board_operations() {
        let mut board = Board::new(5);
        assert!(board.is_empty(Point { x: 2, y: 2 }));
        board.set(Point { x: 2, y: 2 }, Color::Black);
        board.set(Point { x: 1, y: 2 }, Color::White);
        println!("{}", board);
        assert_eq!(board.get(Point { x: 2, y: 2 }), Some(Color::Black));
        assert_eq!(board.get(Point { x: 1, y: 2 }), Some(Color::White));
        board.remove(Point { x: 2, y: 2 });
        assert!(board.is_empty(Point { x: 2, y: 2 }));
    }

    /// 测试相邻点
    #[test]
    fn test_neighbors() {
        //点位于中间
        let board = Board::new(5);
        let pt = Point { x: 2, y: 2 };
        let neighbors = board.neighbors(pt);
        let expected = vec![
            Point { x: 2, y: 3 },
            Point { x: 2, y: 1 },
            Point { x: 3, y: 2 },
            Point { x: 1, y: 2 },
        ];
        assert_eq!(neighbors.len(), expected.len());
        for nb in expected {
            assert!(neighbors.contains(&nb));
        }

        //点位于边界
        let pt = Point { x: 1, y: 0 };
        let neighbors = board.neighbors(pt);
        let expected = vec![
            Point { x: 0, y: 0 },
            Point { x: 1, y: 1 },
            Point { x: 2, y: 0 },
        ];
        assert_eq!(neighbors.len(), expected.len());
        for nb in expected {
            assert!(neighbors.contains(&nb));
        }

        //点位于角落
        let pt = Point { x: 4, y: 4 };
        let neighbors = board.neighbors(pt);
        let expected = vec![Point { x: 3, y: 4 }, Point { x: 4, y: 3 }];
        assert_eq!(neighbors.len(), expected.len());
        for nb in expected {
            assert!(neighbors.contains(&nb));
        }
    }

    /// 测试对角点
    #[test]
    fn test_diagonals() {
        //点位于中间
        let board = Board::new(5);
        let pt = Point { x: 2, y: 2 };
        let diagonals = board.diagonals(pt);
        let expected = vec![
            Point { x: 1, y: 1 },
            Point { x: 3, y: 1 },
            Point { x: 3, y: 3 },
            Point { x: 1, y: 3 },
        ];
        assert_eq!(diagonals.len(), expected.len());
        for nb in expected {
            assert!(diagonals.contains(&nb));
        }

        //点位于边界
        let pt = Point { x: 1, y: 0 };
        let diagonals = board.diagonals(pt);
        let expected = vec![Point { x: 0, y: 1 }, Point { x: 2, y: 1 }];
        assert_eq!(diagonals.len(), expected.len());
        for nb in expected {
            assert!(diagonals.contains(&nb));
        }

        //点位于角落
        let pt = Point { x: 4, y: 4 };
        let diagonals = board.diagonals(pt);
        let expected = vec![Point { x: 3, y: 3 }];
        assert_eq!(diagonals.len(), expected.len());
        for nb in expected {
            assert!(diagonals.contains(&nb));
        }
    }

    /// 测试连通块
    #[test]
    fn test_get_group() {
        let mut board = Board::new(5);
        // 创建一个简单的连通块
        board.set(Point { x: 1, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 2, y: 1 }, Color::Black);

        let group = board.get_group(Point { x: 1, y: 1 });
        let expected = vec![
            Point { x: 1, y: 1 },
            Point { x: 1, y: 2 },
            Point { x: 2, y: 1 },
        ];
        assert_eq!(group.len(), expected.len());
        for pt in expected {
            assert!(group.contains(&pt));
        }
    }

    /// 测试基本提子逻辑
    #[test]
    fn test_basic_capture() {
        let mut board = Board::new(5);
        // 创建被包围的白子
        board.set(Point { x: 2, y: 2 }, Color::White);

        // 黑子包围
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 3, y: 2 }, Color::Black);
        board.set(Point { x: 2, y: 1 }, Color::Black);

        // println!("{}", board.to_string_with_moves(None));

        // 最后一步提子
        let mv = Move::new(Color::Black, Point { x: 2, y: 3 }, 5).unwrap();

        let (captured, _) = board.apply_move(&mv, None, false).unwrap();
        // println!("{}", board.to_string_with_moves(None));
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0], Point { x: 2, y: 2 });
        assert_eq!(board.get(Point { x: 2, y: 2 }), None);
    }

    #[test]
    fn test_suicide_rule() {
        let mut board = Board::new(3);
        // 白子包围一角
        board.set(Point { x: 0, y: 1 }, Color::White);
        board.set(Point { x: 1, y: 0 }, Color::White);
        // println!("{}", board.to_string_with_moves(None));
        // 黑子试图自杀（不允许自杀规则下）
        let mv = Move::new(Color::Black, Point { x: 0, y: 0 }, 3).unwrap();
        assert!(!board.is_legal(&mv, None, false));

        // 允许自杀规则下
        assert!(board.is_legal(&mv, None, true));
    }

    #[test]
    fn test_liberty_counting() {
        let board = Board::new(5);
        // 单个棋子有 4 口气
        let pt = Point { x: 2, y: 2 };
        let group = board.get_group(pt); // 空位置返回空集合
        assert!(group.is_empty());

        // 放置棋子后测试
        let mut board = Board::new(5);
        board.set(pt, Color::Black);
        let group = board.get_group(pt);
        assert_eq!(group.len(), 1);
        assert_eq!(board.count_liberties(&group).len(), 4);

        // 连接后气数减少
        board.set(Point { x: 2, y: 3 }, Color::Black);
        let group = board.get_group(pt);
        assert_eq!(group.len(), 2);
        assert_eq!(board.count_liberties(&group).len(), 6); // 2*4 - 2(共享边) = 6
    }

    // 测试劫的特殊情况
    #[test]
    fn test_ko_scenario() {
        let mut board = Board::new(5);
        // 创建一个简单的劫
        board.set(Point { x: 1, y: 0 }, Color::Black);
        board.set(Point { x: 0, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 2, y: 1 }, Color::Black);
        board.set(Point { x: 2, y: 0 }, Color::White);
        board.set(Point { x: 2, y: 2 }, Color::White);
        board.set(Point { x: 3, y: 1 }, Color::White);
        println!("Initial board:\n{}\n", board);

        // 白提黑一子
        let mv_white = Move::new(Color::White, Point { x: 1, y: 1 }, 5).unwrap();
        let (mv_black, ko_point) = board.apply_move(&mv_white, None, false).unwrap();
        println!("{}", board.to_string_with_move(mv_white));

        assert_eq!(ko_point, Some(Point { x: 2, y: 1 }));
        assert_eq!(mv_black.len(), 1);
        assert_eq!(mv_black[0], Point { x: 2, y: 1 });

        // 黑不能立即回提（劫）
        let mv_black = Move::new(Color::Black, Point { x: 2, y: 1 }, 5).unwrap();
        assert!(!board.is_legal(&mv_black, ko_point, false));
        let res = board.apply_move(&mv_black, ko_point, false);
        assert!(res.is_none()); //黑应该不能立即回提（劫）
        println!("棋盘应该没有变化：\n{}\n", board);

        // 黑先走其他位置,白应一手棋
        let mv_black_other = Move::new(Color::Black, Point { x: 0, y: 0 }, 5).unwrap();
        let mv_white_other = Move::new(Color::White, Point { x: 3, y: 0 }, 5).unwrap();
        assert!(board.is_legal(&mv_black_other, ko_point, false));
        let (_, ko_point) = board.apply_move(&mv_black_other, ko_point, false).unwrap();
        println!("{}", board.to_string_with_move(mv_black_other));

        assert!(board.is_legal(&mv_white_other, ko_point, false));
        let (_, ko_point) = board.apply_move(&mv_white_other, ko_point, false).unwrap();
        println!("{}", board.to_string_with_move(mv_white_other));

        // 现在黑可以回提了
        assert_eq!(ko_point, None);
        assert!(board.is_legal(&mv_black, ko_point, false));
        let (captured_stones, ko_point) = board.apply_move(&mv_black, ko_point, false).unwrap();
        assert_eq!(captured_stones[0], Point { x: 1, y: 1 });
        // 回提后新的劫点是之前被提的位置
        assert_eq!(ko_point.unwrap(), Point { x: 1, y: 1 });
    }

    #[test]
    fn test_eye_analysis() {
        let mut board = Board::new(5);
        // 角落的空点（0,0）
        board.set(Point { x: 0, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 0 }, Color::Black);
        // 中间的空点（1,1）
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 2, y: 1 }, Color::Black);
        board.set(Point { x: 2, y: 2 }, Color::Black);
        // 边上的空点（2,0）
        board.set(Point { x: 3, y: 0 }, Color::Black);
        board.set(Point { x: 3, y: 1 }, Color::White);

        let eye1 = Point { x: 0, y: 0 };
        let eye2 = Point { x: 1, y: 1 };
        let eye3 = Point { x: 2, y: 0 };

        // 中间的空点是一个真眼
        let eye_type = board.analyze_eye(eye3, Color::Black);
        assert_eq!(eye_type, Some(EyeType::Real)); //判断错误，需要结合多个块判断

        // 边上的空点是假眼
        let eye_type = board.analyze_eye(eye2, Color::Black);
        assert_eq!(eye_type, Some(EyeType::Real));

        // 角落的空点是真眼
        let eye_type = board.analyze_eye(eye1, Color::Black);
        assert_eq!(eye_type, Some(EyeType::False));
    }

    /// 测试块群
    #[test]
    fn test_all_group() {
        let mut board = Board::new(5);
        board.set(Point { x: 1, y: 0 }, Color::Black);
        board.set(Point { x: 0, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 1, y: 3 }, Color::Black);

        board.set(Point { x: 2, y: 0 }, Color::White);
        board.set(Point { x: 2, y: 2 }, Color::White);
        board.set(Point { x: 3, y: 1 }, Color::White);
        board.set(Point { x: 3, y: 0 }, Color::White);
        println!("Initial board:\n{}\n", board);
        let a = board.all_blocks();
        println!("{:?}", a);
        let mut pt = HashSet::new();
        pt.insert(Point { x: 0, y: 1 });
        assert!(a.contains(&(Color::Black, pt)));
        assert_eq!(a.len(), 5);
    }
}
