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
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    /// 创建带边界验证的坐标
    pub fn new_valid(x: u8, y: u8, board_size: u8) -> Option<Self> {
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
        } else if col_char <= 'H' {
            col_char as u8 - b'A'
        } else {
            return None; // 'I' 是无效的列
        };

        Self::new_valid(col, row, board_size)
    }

    /// SGF 单字符坐标转换 (a-t 跳过 i) -> 0-based 数值
    /// 例如: 'a'→0, 'h'→7, 'j'→8, 's'→18
    pub fn from_sgf_coord_char(c: char) -> Option<u8> {
        match c {
            //0..=7
            'a'..='h' => Some(c as u8 - b'a'),
            //8..=18
            'j'..='z' => Some(c as u8 - b'j' + 8), //跳过i
            _ => None,
        }
    }

    /// SGF 双字符坐标解析 (如 "pd") → Point
    pub fn from_sgf(s: &str, board_size: u8) -> Option<Self> {
        let mut chars = s.chars();
        let x = Self::from_sgf_coord_char(chars.next()?)?;
        let y = Self::from_sgf_coord_char(chars.next()?)?;
        if chars.next().is_some() {
            return None;
        } // 多余字符
        let pt = Self { x, y };
        if pt.is_valid(board_size) {
            Some(pt)
        } else {
            None
        }
    }

    /// Point → SGF 双字符坐标 (如 "pd")
    pub fn to_sgf(&self) -> String {
        let x = match self.x {
            0..=7 => (b'a' + self.x) as char,
            8..=18 => (b'j' + self.x - 8) as char,
            _ => '?',
        };
        let y = match self.y {
            0..=7 => (b'a' + self.y) as char,
            8..=18 => (b'j' + self.y - 8) as char,
            _ => '?',
        };
        format!("{}{}", x, y)
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
    fn test_point_validation() {
        assert!(Point::new_valid(0, 0, 19).is_some());
        assert!(Point::new_valid(18, 18, 19).is_some());
        assert!(Point::new_valid(19, 0, 19).is_none());
        assert!(Point::from_sgf("tt", 19).is_some());
        assert!(Point::from_sgf("uu", 19).is_none()); // t=18, u=19, 19x19 棋盘最大索引 18
    }

    #[test]
    fn test_point_coord_gtp_conversion() {
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

        // 无效坐标测试
        assert_eq!(Point::from_gtp_coord("I1", 19), None);

        // 无效输入测试
        assert_eq!(Point::from_gtp_coord("A", 19), None);
        assert_eq!(Point::from_gtp_coord("AA", 19), None);

        // 边界测试
        assert_eq!(
            Point::from_gtp_coord("T19", 19).unwrap(),
            Point { x: 18, y: 18 }
        );
        assert_eq!(Point::from_gtp_coord("U1", 19), None);
    }

    #[test]
    fn test_sgf_coord_roundtrip() {
        for c in "abcdefghjklmnopqrst".chars() {
            if let Some(val) = Point::from_sgf_coord_char(c) {
                let pt = Point { x: val, y: 0 };
                let result = pt.to_sgf();
                assert_eq!(result.chars().next(), Some(c));
            }
        }
    }

    #[test]
    fn test_sgf_coord_pair() {
        let pt = Point { x: 3, y: 15 };
        let sgf = pt.to_sgf();
        assert_eq!(sgf, "dq");
        let parsed_pt = Point::from_sgf(&sgf, 19).unwrap();
        assert_eq!(parsed_pt, pt);
    }

    #[test]
    fn test_invalid_sgf_coord() {
        assert_eq!(Point::from_sgf("aa", 19), Some(Point { x: 0, y: 0 }));
        assert_eq!(Point::from_sgf("a", 19), None); // 不足两个字符
        assert_eq!(Point::from_sgf("aaa", 19), None); // 多余字符
        assert_eq!(Point::from_sgf("a1", 19), None); // 非法字符
        assert_eq!(Point::from_sgf("tt", 19), Some(Point { x: 18, y: 18 }));
        assert_eq!(Point::from_sgf("uu", 19), None); // 超出范围
    }
}
