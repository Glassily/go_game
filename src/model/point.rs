use std::fmt::Display;

/// 棋盘坐标 (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    /// 横坐标 (0-based)
    pub x: u8,
    /// 纵坐标 (0-based)
    pub y: u8,
}

impl Point {
    /// 创建带边界验证的坐标
    pub fn new(x: u8, y: u8, board_size: u8) -> Option<Self> {
        if x < board_size && y < board_size {
            Some(Self { x, y })
        } else {
            None
        }
    }

    /// 验证坐标是否在当前棋盘范围内
    pub fn is_valid(&self, board_size: u8) -> bool {
        self.x < board_size && self.y < board_size
    }

    /// 转换为围棋坐标表示 (如: (0,0) -> "A1", (3,15) -> "D16")
    /// 注意：跳过字母 'I' 以避免混淆
    pub fn to_gtp_coord(&self) -> String {
        let col = if self.x >= 8 {
            (b'A' + self.x + 1) as char // 跳过 'I'
        } else {
            (b'A' + self.x) as char
        };
        let row = self.y + 1; // GTP 坐标从 1 开始
        format!("{}{}", col, row)
    }

    /// 从 GTP 坐标解析 (如: "A1" -> Point{x:0, y:0})
    pub fn from_gtp_coord(s: &str, board_size: u8) -> Option<Self> {
        let s = s.trim().to_uppercase();
        let mut chars = s.chars();
        let col_char = chars.next()?;
        let row_str: String = chars.collect();
        let row: u8 = row_str.parse::<u8>().ok()?.checked_sub(1)?;

        // 处理跳过 'I' 的情况
        let col = if col_char >= 'J' {
            col_char as u8 - b'A' - 1
        } else {
            col_char as u8 - b'A'
        };

        Self::new(col, row, board_size)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_coord_conversion() {
        let pt = Point { x: 0, y: 0 };
        assert_eq!(pt.to_gtp_coord(), "A1");

        let pt = Point { x: 3, y: 15 };
        assert_eq!(pt.to_gtp_coord(), "D16");

        // 跳过 I 的测试
        let pt = Point { x: 8, y: 0 }; // 应该是 J1
        assert_eq!(pt.to_gtp_coord(), "J1");

        // 反向解析
        let pt = Point::from_gtp_coord("A1", 19).unwrap();
        assert_eq!(pt.x, 0);
        assert_eq!(pt.y, 0);

        let pt = Point::from_gtp_coord("J10", 19).unwrap();
        assert_eq!(pt.x, 8); // 跳过 I
        assert_eq!(pt.y, 9);
    }
}
