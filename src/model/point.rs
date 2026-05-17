use std::fmt::Display;

/// 棋盘坐标结构体（使用 0-based 索引）
///
/// # 坐标系统说明
/// - `x`: 横坐标，从左到右递增 (0 ~ board_size-1)
/// - `y`: 纵坐标，从上到下递增 (0 ~ board_size-1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    /// 横坐标（列索引，0-based）
    pub x: u8,
    /// 纵坐标（行索引，0-based）
    pub y: u8,
}

impl Point {
    /// 创建新的坐标点
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    /// 创建带边界验证的坐标点
    ///
    /// 仅当坐标在棋盘范围内时才创建成功
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

    /// 转换为 GTP 围棋坐标表示
    ///
    /// GTP 坐标格式为 `{列}{行}`，例如 "D4"
    /// - 列使用字母 A-T（跳过 'I'），从左到右
    /// - 行使用数字 1-19（可配置），从下往上
    ///
    /// # 示例
    /// ```
    /// let pt = Point { x: 3, y: 15 }; // 0-based 坐标
    /// assert_eq!(pt.to_gtp(19), "D4");
    /// ```
    pub fn to_gtp(&self, board_size: u8) -> String {
        let col = if self.x >= 8 {
            (b'A' + self.x + 1) as char
        } else {
            (b'A' + self.x) as char
        };
        let row = board_size - self.y;
        format!("{}{}", col, row)
    }

    /// 从 GTP 坐标解析
    ///
    /// 支持的格式如 "A19", "J10" 等
    ///
    /// # 示例
    /// ```
    /// let pt = Point::from_gtp("D4", 19).unwrap();
    /// assert_eq!(pt.x, 3);
    /// assert_eq!(pt.y, 15);
    /// ```
    pub fn from_gtp(s: &str, board_size: u8) -> Option<Self> {
        let s = s.trim().to_uppercase();
        let mut chars = s.chars();
        let col_char = chars.next()?;
        let row_str: String = chars.collect();

        let col = match col_char {
            'A'..='H' => col_char as u8 - b'A',
            'J'..='T' => col_char as u8 - b'A' - 1,
            _ => return None,
        };

        let row: u8 = board_size - row_str.parse::<u8>().ok()?;

        Self::new_valid(col, row, board_size)
    }

    /// 转换为 SGF 双字符坐标
    ///
    /// SGF 坐标使用小写字母 a-s 表示 0-18
    ///
    /// # 示例
    /// ```
    /// let pt = Point { x: 3, y: 15 };
    /// assert_eq!(pt.to_sgf(), "dp");
    /// ```
    pub fn to_sgf(&self) -> String {
        let to_sgf_char = |v: u8| -> char { if v < 19 { (b'a' + v) as char } else { '?' } };
        format!("{}{}", to_sgf_char(self.x), to_sgf_char(self.y))
    }

    /// 从 SGF 双字符坐标解析
    ///
    /// 支持的格式如 "pd", "aa" 等
    ///
    /// # 示例
    /// ```
    /// let pt = Point::from_sgf("dp", 19).unwrap();
    /// assert_eq!(pt.x, 3);
    /// assert_eq!(pt.y, 15);
    /// ```
    pub fn from_sgf(s: &str, board_size: u8) -> Option<Self> {
        let from_sgf_char = |c: char| -> Option<u8> {
            match c {
                'a'..='s' => Some(c as u8 - b'a'),
                _ => None,
            }
        };
        let mut chars = s.chars();
        let x = from_sgf_char(chars.next()?)?;
        let y = from_sgf_char(chars.next()?)?;
        if chars.next().is_some() {
            return None;
        }
        Self::new_valid(x, y, board_size)
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
        assert!(Point::from_sgf("ss", 19).is_some());
        assert!(Point::from_sgf("tt", 19).is_none());
    }

    #[test]
    fn test_point_coord_gtp_conversion() {
        let pt = Point { x: 0, y: 0 };
        assert_eq!(pt.to_gtp(19), "A19");

        // 0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 点坐标
        // A  B  C  D  E  F  G  H  J  K  L  M  N  O  P  Q  R  S  T  列坐标
        // 19 18 17 16 18 14 13 12 11 10 9  8  7  6  5  4  3  2  1  行坐标
        // 跳过 I 的测试
        let pt = Point { x: 8, y: 9 }; // 应该是 J10
        assert_eq!(pt.to_gtp(19), "J10");

        // 反向解析
        let pt = Point::from_gtp("A19", 19).unwrap();
        assert_eq!(pt.x, 0);
        assert_eq!(pt.y, 0);

        // 0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 点坐标
        // A  B  C  D  E  F  G  H  J  K  L  M  N  O  P  Q  R  S  T  列坐标
        // 19 18 17 16 18 14 13 12 11 10 9  8  7  6  5  4  3  2  1  行坐标
        let pt = Point::from_gtp("J10", 19).unwrap();
        assert_eq!(pt.x, 8); // 跳过 I
        assert_eq!(pt.y, 9);

        // 无效坐标测试
        assert_eq!(Point::from_gtp("I1", 19), None);

        // 无效输入测试
        assert_eq!(Point::from_gtp("A", 19), None);
        assert_eq!(Point::from_gtp("AA", 19), None);
        assert_eq!(Point::from_gtp("", 19), None);

        // 边界测试
        assert_eq!(Point::from_gtp("T19", 19).unwrap(), Point { x: 18, y: 0 });
        assert_eq!(Point::from_gtp("U1", 19), None);
    }

    #[test]
    fn test_sgf_coord_pair() {
        let pt = Point { x: 3, y: 15 };
        let sgf = pt.to_sgf();
        assert_eq!(sgf, "dp");
        let parsed_pt = Point::from_sgf(&sgf, 19).unwrap();
        assert_eq!(parsed_pt, pt);
    }

    #[test]
    fn test_invalid_sgf_coord() {
        assert_eq!(Point::from_sgf("aa", 19), Some(Point { x: 0, y: 0 }));
        assert_eq!(Point::from_sgf("a", 19), None); // 不足两个字符
        assert_eq!(Point::from_sgf("aaa", 19), None); // 多余字符
        assert_eq!(Point::from_sgf("a1", 19), None); // 非法字符
        assert_eq!(Point::from_sgf("ss", 19), Some(Point { x: 18, y: 18 }));
        assert_eq!(Point::from_sgf("tt", 19), None); // 超出范围
    }
}
