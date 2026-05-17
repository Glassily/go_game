use std::fmt::Display;

/// 棋子颜色枚举
///
/// 表示围棋中的黑白双方
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Color {
    /// 黑子 ●
    Black,
    /// 白子 ○
    White,
}

impl Color {
    /// 从字符创建颜色
    ///
    /// - `'B'` 或 `'b'` → 黑子
    /// - `'W'` 或 `'w'` → 白子
    /// - 其他字符 → None
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'B' | 'b' => Some(Color::Black),
            'W' | 'w' => Some(Color::White),
            _ => None,
        }
    }

    /// 切换到对手的颜色
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
