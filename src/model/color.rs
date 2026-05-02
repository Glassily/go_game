use std::fmt::Display;

/// 棋子颜色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Color {
    /// 黑子 ●
    Black,
    /// 白子 ○
    White,
}

impl Color {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'B' | 'b' => Some(Color::Black),
            'W' | 'w' => Some(Color::White),
            _ => None,
        }
    }

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
