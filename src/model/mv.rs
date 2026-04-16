use std::fmt::Display;

use crate::model::{Color, Point};

/// 一手棋 (None 表示 Pass)
///
/// `color` 字段表示该着法的执行方，而非游戏状态的"当前轮到谁"。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub color: Color,
    pub point: Option<Point>,
}

impl Move {
    /// 创建 Pass 着法
    pub fn pass(color: Color) -> Self {
        Self { color, point: None }
    }

    /// 创建落子着法（带边界验证）
    pub fn new(color: Color, point: Point, board_size: u8) -> Option<Self> {
        if point.is_valid(board_size) {
            Some(Self {
                color,
                point: Some(point),
            })
        } else {
            None
        }
    }

    /// 是否为 Pass
    pub fn is_pass(&self) -> bool {
        self.point.is_none()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.point {
            Some(pt) => write!(f, "{}@{}", self.color, pt.to_gtp_coord()),
            None => write!(f, "{}:pass", self.color),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let mv1 = Move::new(Color::Black, Point { x: 3, y: 3 }, 19).unwrap();
        assert_eq!(mv1.color, Color::Black);
        assert_eq!(mv1.point, Some(Point { x: 3, y: 3 }));

        let mv2 = Move::pass(Color::White);
        assert_eq!(mv2.color, Color::White);
        assert!(mv2.is_pass());
    }
}
