use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

use crate::model::{Color, Move, Point};

/// 棋盘状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// 棋盘大小
    pub size: u8,
    grid: Vec<Vec<Option<Color>>>,
}

impl Board {
    /// 创建空棋盘
    pub fn new(size: u8) -> Self {
        assert!(size >= 2 && size <= 25, "Invalid board size");
        let grid = vec![vec![None; size as usize]; size as usize];
        Board { size, grid }
    }

    /// 根据 setup 列表初始化棋盘 (AB/AW)，默认列表是无序的
    pub fn from_setup(size: u8, stones: &[(Point, Color)]) -> Self {
        let mut board = Self::new(size);
        for &(pt, color) in stones {
            if pt.is_valid(size) {
                board.grid[pt.y as usize][pt.x as usize] = Some(color);
            }
        }
        board
    }

    /// 获取棋子颜色
    pub fn get(&self, pt: Point) -> Option<Color> {
        if pt.is_valid(self.size) {
            self.grid[pt.y as usize][pt.x as usize]
        } else {
            None
        }
    }

    /// 放置棋子 (不检查合法性)
    pub fn set(&mut self, pt: Point, color: Color) {
        if pt.is_valid(self.size) {
            self.grid[pt.y as usize][pt.x as usize] = Some(color);
        }
    }

    /// 移除棋子
    pub fn remove(&mut self, pt: Point) {
        if pt.is_valid(self.size) {
            self.grid[pt.y as usize][pt.x as usize] = None;
        }
    }

    /// 可视化棋盘（文本格式）
    pub fn to_string_with_moves(&self, last_move: Option<Move>) -> String {
        let mut result = String::new();

        // 顶部坐标
        result.push_str("   ");
        for x in 0..self.size {
            let c = if x >= 8 {
                (b'A' + x + 1) as char
            } else {
                (b'A' + x) as char
            };
            result.push(c);
            result.push(' ');
        }
        result.push('\n');

        // 棋盘内容
        for y in 0..self.size {
            result.push_str(&format!("{:2} ", y + 1));
            for x in 0..self.size {
                let pt = Point { x, y };
                let ch = match self.get(pt) {
                    Some(Color::Black) => "●",
                    Some(Color::White) => "○",
                    None => {
                        // 标记最后落子位置
                        if let Some(mv) = last_move {
                            if mv.point == Some(pt) && !mv.is_pass() {
                                "*"
                            } else {
                                "+"
                            }
                        } else {
                            "+"
                        }
                    }
                };
                result.push_str(ch);
                result.push(' ');
            }
            result.push_str(&format!(" {}\n", y + 1));
        }

        // 底部坐标
        result.push_str("   ");
        for x in 0..self.size {
            let c = if x >= 8 {
                (b'A' + x + 1) as char
            } else {
                (b'A' + x) as char
            };
            result.push(c);
            result.push(' ');
        }
        result
    }

    /// 检查位置是否为空
    pub fn is_empty(&self, pt: Point) -> bool {
        self.get(pt).is_none()
    }

    /// 获取相邻点 (四方向)
    pub fn neighbors(&self, pt: Point) -> Vec<Point> {
        let mut nbs = Vec::new();
        let (x, y) = (pt.x as i32, pt.y as i32);
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for (dx, dy) in dirs {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < self.size as i32 && ny >= 0 && ny < self.size as i32 {
                nbs.push(Point {
                    x: nx as u8,
                    y: ny as u8,
                });
            }
        }
        nbs
    }

    /// 获取连通块 (相同颜色的相邻棋子)
    pub fn get_group(&self, pt: Point) -> HashSet<Point> {
        let color = match self.get(pt) {
            Some(c) => c,
            None => return HashSet::new(),
        };
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(pt);
        visited.insert(pt);

        while let Some(p) = queue.pop_front() {
            for neighbor in self.neighbors(p) {
                if !visited.contains(&neighbor) && self.get(neighbor) == Some(color) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        visited
    }

    /// 计算连通块的气数
    pub fn count_liberties(&self, group: &HashSet<Point>) -> usize {
        let mut liberties = HashSet::new();
        for &pt in group {
            for nb in self.neighbors(pt) {
                if self.get(nb).is_none() {
                    liberties.insert(nb);
                }
            }
        }
        liberties.len()
    }

    /// 移除指定颜色所有无气棋子 (返回被移除的棋子列表)
    pub fn remove_dead_groups(&mut self, color: Color) -> Vec<Point> {
        let mut removed = Vec::new();
        // 找出所有对方的棋子，检查其连通块是否有气
        let mut visited = HashSet::new();
        for y in 0..self.size {
            for x in 0..self.size {
                let pt = Point { x, y };
                if let Some(c) = self.get(pt) {
                    if c == color && !visited.contains(&pt) {
                        let group = self.get_group(pt);
                        visited.extend(group.iter().copied());
                        if self.count_liberties(&group) == 0 {
                            for &p in &group {
                                self.remove(p);
                                removed.push(p);
                            }
                        }
                    }
                }
            }
        }
        removed
    }

    /// 检查落子合法性
    ///
    /// 规则：
    /// 1. 位置必须在棋盘内且为空
    /// 2. 落子后必须有气，或能提掉对方棋子
    /// 3. [可选] 劫规则：不能立即回提单子
    pub fn is_legal(&self, mv: &Move, ko_point: Option<Point>, allow_suicide: bool) -> bool {
        if mv.is_pass() {
            return true; // Pass 总是合法的
        }

        let pt = mv.point.unwrap();

        // 1. 位置必须为空
        if !self.is_empty(pt) {
            return false;
        }

        // 2. 模拟落子
        let mut temp_board = self.clone();
        temp_board.set(pt, mv.color);

        // 3. 检查是否能提掉对方棋子
        let opponent = mv.color.opposite();
        let mut captured = false;
        for nb in temp_board.neighbors(pt) {
            if temp_board.get(nb) == Some(opponent) {
                let group = temp_board.get_group(nb);
                if temp_board.count_liberties(&group) == 0 {
                    captured = true;
                    break;
                }
            }
        }

        // 4. 检查自己的气
        let my_group = temp_board.get_group(pt);
        let my_liberties = temp_board.count_liberties(&my_group);

        // 5. 合法性判断
        if my_liberties > 0 {
            // 有气，合法
            // 劫规则检查
            if !captured && my_group.len() == 1 && my_liberties == 1 {
                if let Some(ko) = ko_point {
                    if pt == ko {
                        return false; // 违反劫规则
                    }
                }
            }
            true
        } else if captured {
            // 无气但能提子，合法
            true
        } else {
            // 无气且不能提子（自杀）
            allow_suicide
        }
    }

    /// 执行着法
    ///
    /// 返回: (是否成功, 被提掉的棋子列表, 新的劫点)
    /// 如果返回 None 表示着法非法
    pub fn apply_move(
        &mut self,
        mv: &Move,
        ko_point: Option<Point>,
        allow_suicide: bool,
    ) -> Option<(Vec<Point>, Option<Point>)> {
        if !self.is_legal(mv, ko_point, allow_suicide) {
            return None;
        }

        if mv.is_pass() {
            return Some((Vec::new(), None));
        }

        let pt = mv.point.unwrap();
        let color = mv.color;
        let opponent = color.opposite();

        // 1. 落子
        self.set(pt, color);

        // 2. 提掉对方无气的棋子
        let mut captured = Vec::new();
        for nb in self.neighbors(pt) {
            if self.get(nb) == Some(opponent) {
                let group = self.get_group(nb);
                if self.count_liberties(&group) == 0 {
                    for &p in &group {
                        self.remove(p);
                        captured.push(p);
                    }
                }
            }
        }

        // 3. 计算劫点 (简单劫：提单子且自己被提后也是单子)
        let new_ko = if captured.len() == 1 {
            let my_group = self.get_group(pt);
            if my_group.len() == 1 && self.count_liberties(&my_group) == 1 {
                Some(captured[0]) // 被提的位置是潜在的劫点
            } else {
                None
            }
        } else {
            None
        };

        Some((captured, new_ko))
    }

    /// 计算双方地盘（简化版：数子法，不考虑眼位判断）
    ///
    /// 需要移除死子后调用
    pub fn count_stones(&self, color: Color) -> usize {
        self.grid
            .iter()
            .flatten()
            .filter(|&&c| c == Some(color))
            .count()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        // 顶部坐标
        result.push_str("   ");
        for x in 0..self.size {
            let c = if x >= 8 {
                (b'A' + x + 1) as char
            } else {
                (b'A' + x) as char
            };
            result.push(c);
            result.push(' ');
        }
        result.push('\n');

        // 棋盘内容
        let size = self.size as usize;
        for y in 0..size {
            result.push_str(&format!("{:2} ", y + 1));
            for x in 0..size {
                let ch = match self.grid[y][x] {
                    Some(Color::Black) => "●",
                    Some(Color::White) => "○",
                    None => "+",
                };
                result.push_str(ch);
                result.push(' ');
            }
            result.push_str(&format!(" {}\n", y + 1));
        }

        // 底部坐标
        result.push_str("   ");
        for x in 0..self.size {
            let c = if x >= 8 {
                (b'A' + x + 1) as char
            } else {
                (b'A' + x) as char
            };
            result.push(c);
            result.push(' ');
        }
        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_coord_conversion() {
        // 测试 GTP 坐标转换
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
    fn test_ko_rule() {
        let mut board = Board::new(5);
        // 创建劫的形状
        board.set(Point { x: 1, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 3 }, Color::Black);
        board.set(Point { x: 2, y: 2 }, Color::Black);
        board.set(Point { x: 3, y: 1 }, Color::White);
        board.set(Point { x: 3, y: 3 }, Color::White);
        board.set(Point { x: 2, y: 4 }, Color::White);

        println!("{}", board.to_string_with_moves(None));
        // 白提黑一子
        let mv_white = Move::new(Color::White, Point { x: 2, y: 1 }, 5).unwrap();
        let (_, ko_point) = board.apply_move(&mv_white, None, false).unwrap();
        
        assert_eq!(ko_point, Some(Point { x: 2, y: 2 }));
        println!("{}", board.to_string_with_moves(None));
        // 黑不能立即回提（劫）
        let mv_black = Move::new(Color::Black, Point { x: 2, y: 2 }, 5).unwrap();
        assert!(!board.is_legal(&mv_black, ko_point, false));
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
        assert_eq!(board.count_liberties(&group), 4);

        // 连接后气数减少
        board.set(Point { x: 2, y: 3 }, Color::Black);
        let group = board.get_group(pt);
        assert_eq!(group.len(), 2);
        assert_eq!(board.count_liberties(&group), 6); // 2*4 - 2(共享边) = 6
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
        println!("{}", board.to_string_with_moves(None));
        // 白提黑一子
        let mv_white = Move::new(Color::White, Point { x: 1, y: 1 }, 5).unwrap();
        let (_, ko_point) = board.apply_move(&mv_white, None, false).unwrap();
        println!("{}", board.to_string_with_moves(None));
        assert_eq!(ko_point, Some(Point { x: 2, y: 1 }));

        // 黑不能立即回提（劫）
        let mv_black = Move::new(Color::Black, Point { x: 3, y: 2 }, 5).unwrap();
        // assert!(!board.is_legal(&mv_black, ko_point, false));
        board.apply_move(&mv_black, ko_point, false);
        println!("{}", board.to_string_with_moves(None));

        // 黑先走其他位置,白应一手棋
        let mv_black_other = Move::new(Color::Black, Point { x: 0, y: 0 }, 5).unwrap();
        let mv_white_other = Move::new(Color::White, Point { x: 3, y: 0 }, 5).unwrap();
        assert!(board.is_legal(&mv_black_other, ko_point, false));
        board.apply_move(&mv_black_other, ko_point, false);
        assert!(board.is_legal(&mv_white_other, ko_point, false));

        // 现在黑可以回提了
        assert!(board.is_legal(&mv_black, None, false));

    }
}
