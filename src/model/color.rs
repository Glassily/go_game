use std::fmt::Display;

use crate::model::Point;

/// 棋子颜色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// 黑子●
    Black,
    /// 白子○
    White,
}

impl Color {
    /// 切换颜色
    pub fn opposite(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Black => write!(f, "B"),
            Color::White => write!(f, "W"),
        }
    }
}

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
