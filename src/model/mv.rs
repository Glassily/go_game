use std::fmt::Display;

use crate::model::{Color, Point};

/// 围棋着法结构体
///
/// 表示围棋中的一步棋，可以是落子或 Pass
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    /// 执行着法的一方颜色
    pub color: Color,
    /// 落子位置，None 表示 Pass
    pub point: Option<Point>,
}

impl Move {
    /// 创建 Pass 着法
    pub fn pass(color: Color) -> Self {
        Self { color, point: None }
    }

    /// 创建落子着法
    pub fn new(color: Color, point: Point) -> Self {
        Self {
            color,
            point: Some(point),
        }
    }

    /// 创建带边界验证的着法
    ///
    /// 仅当落子位置在棋盘范围内时才创建成功
    pub fn new_valid(color: Color, point: Point, board_size: u8) -> Option<Self> {
        if point.is_valid(board_size) {
            Some(Self {
                color,
                point: Some(point),
            })
        } else {
            None
        }
    }

    /// 判断是否为 Pass
    pub fn is_pass(&self) -> bool {
        self.point.is_none()
    }

    /// 转换为 GTP 格式字符串
    ///
    /// 格式为 `{color}@{point}` 或 `{color}:pass`
    pub fn to_string_gtp(&self, board_size: u8) -> String {
        match self.point {
            Some(pt) => format!("{}@{}", self.color, pt.to_gtp(board_size)),
            None => format!("{}:pass", self.color),
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.point {
            Some(pt) => write!(f, "{}@{}", self.color, pt),
            None => write!(f, "{}:pass", self.color),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let mv1 = Move::new(Color::Black, Point { x: 3, y: 3 });
        assert_eq!(mv1.color, Color::Black);
        assert_eq!(mv1.point, Some(Point { x: 3, y: 3 }));

        let mv2 = Move::pass(Color::White);
        assert_eq!(mv2.color, Color::White);
        assert!(mv2.is_pass());
    }
}
