use crate::model::{Color, Move, Point};
use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

/// 围棋棋盘结构体
///
/// 表示一个围棋棋盘及其上的棋子状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// 棋盘大小（通常为 9、13 或 19）
    pub size: u8,
    /// 棋盘网格，使用 Vec<Vec<Option<Color>>> 存储
    grid: Vec<Vec<Option<Color>>>,
}

/// 落子非法原因枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IllegalMoveError {
    /// 无效着法
    InvalidMove,
    /// 超出边界
    OutOfBounds,
    /// 位置已被占用
    Occupied,
    /// 违反劫规则
    KoViolation,
    /// 自杀着法
    Suicide,
}

impl Board {
    /// 创建指定大小的空棋盘
    pub fn new(size: u8) -> Self {
        assert!((2..=25).contains(&size), "Board size must be 2-25");
        Self {
            size,
            grid: vec![vec![None; size as usize]; size as usize],
        }
    }

    /// 从给定棋子配置创建棋盘
    ///
    /// # 参数
    /// - `size`: 棋盘大小
    /// - `stones`: 初始棋子列表，每个元素为 (坐标, 颜色)
    pub fn from_setup(size: u8, stones: &[(Point, Color)]) -> Self {
        let mut board = Self::new(size);
        for &(pt, color) in stones {
            if pt.is_valid(size) {
                board.grid[pt.y as usize][pt.x as usize] = Some(color);
            }
        }
        board
    }

    /// 获取指定位置的棋子颜色
    pub fn get(&self, pt: Point) -> Option<Color> {
        pt.is_valid(self.size)
            .then(|| self.grid[pt.y as usize][pt.x as usize])?
    }

    /// 在指定位置放置棋子
    pub fn set(&mut self, pt: Point, color: Color) {
        if pt.is_valid(self.size) {
            self.grid[pt.y as usize][pt.x as usize] = Some(color);
        }
    }

    /// 移除指定位置的棋子
    pub fn remove(&mut self, pt: Point) {
        if pt.is_valid(self.size) {
            self.grid[pt.y as usize][pt.x as usize] = None;
        }
    }

    /// 检查指定位置是否为空
    pub fn is_empty(&self, pt: Point) -> bool {
        self.get(pt).is_none()
    }

    /// 获取指定点的相邻四个方向邻居
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

    /// 使用 BFS 获取连通块
    ///
    /// 返回从起始点出发，所有同色棋子组成的连通集合
    pub fn get_block(&self, start: Point) -> HashSet<Point> {
        let color = match self.get(start) {
            Some(c) => c,
            None => return HashSet::new(),
        };
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);

        while let Some(pt) = queue.pop_front() {
            for nb in self.neighbors(pt) {
                if !visited.contains(&nb) && self.get(nb) == Some(color) {
                    visited.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        visited
    }

    /// 计算连通块的气的数量
    ///
    /// 气是指与连通块相邻的空点
    pub fn count_liberties(&self, block: &HashSet<Point>) -> HashSet<Point> {
        let mut libs = HashSet::new();
        for &pt in block {
            for nb in self.neighbors(pt) {
                if self.is_empty(nb) {
                    libs.insert(nb);
                }
            }
        }
        libs
    }

    /// 移除因落子而死的棋子
    ///
    /// 返回被移除的棋子列表
    pub fn remove_dead_groups(&mut self, mv: &Move) -> Vec<Point> {
        let mut removed = Vec::new();
        let opponent = mv.color.opposite();

        for nb in self.neighbors(mv.point.unwrap()) {
            if self.get(nb) == Some(opponent) {
                let group = self.get_block(nb);
                if self.count_liberties(&group).len() == 0 {
                    for &p in &group {
                        self.remove(p);
                        removed.push(p);
                    }
                }
            }
        }
        removed
    }

    /// 检查落子合法性
    ///
    /// # 参数
    /// - `mv`: 要检查的着法
    /// - `ko_point`: 劫点位置（该位置禁止立即提回）
    /// - `allow_suicide`: 是否允许自杀着法
    pub fn is_legal(
        &self,
        mv: &Move,
        ko_point: Option<Point>,
        allow_suicide: bool,
    ) -> Result<(), IllegalMoveError> {
        if mv.is_pass() {
            return Ok(());
        }

        let pt = mv.point.ok_or(IllegalMoveError::InvalidMove)?;

        if !pt.is_valid(self.size) {
            return Err(IllegalMoveError::OutOfBounds);
        }
        if !self.get(pt).is_none() {
            return Err(IllegalMoveError::Occupied);
        }

        let mut temp = self.clone();
        temp.set(pt, mv.color);

        let temp_mv = Move {
            point: Some(pt),
            color: mv.color,
        };
        let captured = temp.remove_dead_groups(&temp_mv);

        let my_group = temp.get_block(pt);
        let my_liberties = temp.count_liberties(&my_group);

        if my_liberties.is_empty() && captured.is_empty() {
            if !allow_suicide {
                return Err(IllegalMoveError::Suicide);
            }
        }

        if captured.len() == 1 && my_group.len() == 1 && my_liberties.len() == 1 {
            if let Some(ko) = ko_point {
                if pt == ko {
                    return Err(IllegalMoveError::KoViolation);
                }
            }
        }

        Ok(())
    }

    /// 执行着法
    ///
    /// # 返回值
    /// - `(被提掉的棋子列表, 新的劫点)`
    ///
    /// # 参数
    /// - `mv`: 要执行的着法
    /// - `ko_point`: 当前劫点
    /// - `allow_suicide`: 是否允许自杀
    pub fn apply_move(
        &mut self,
        mv: &Move,
        ko_point: Option<Point>,
        allow_suicide: bool,
    ) -> Result<(Vec<Point>, Option<Point>), IllegalMoveError> {
        if mv.is_pass() {
            return Ok((Vec::new(), None));
        }

        let pt = mv.point.ok_or(IllegalMoveError::InvalidMove)?;
        let color = mv.color;

        if !pt.is_valid(self.size) {
            return Err(IllegalMoveError::OutOfBounds);
        }
        if !self.get(pt).is_none() {
            return Err(IllegalMoveError::Occupied);
        }

        let mut changes: Vec<(Point, Option<Color>)> = Vec::new();

        fn rollback(board: &mut Board, changes: &[(Point, Option<Color>)]) {
            for &(p, old) in changes {
                match old {
                    Some(c) => board.set(p, c),
                    None => board.remove(p),
                }
            }
        }

        changes.push((pt, None));
        self.set(pt, color);

        let opponent = color.opposite();
        let mut groups_to_remove: Vec<(HashSet<Point>, Color)> = Vec::new();

        for nb in self.neighbors(pt) {
            if self.get(nb) == Some(opponent) {
                let group = self.get_block(nb);
                if self.count_liberties(&group).is_empty() {
                    if !groups_to_remove.iter().any(|(g, _)| g.contains(&nb)) {
                        groups_to_remove.push((group, opponent));
                    }
                }
            }
        }

        let mut captured = Vec::new();
        for (group, col) in &groups_to_remove {
            for &p in group {
                captured.push(p);
                changes.push((p, Some(*col)));
                self.remove(p);
            }
        }

        if !allow_suicide {
            let my_group = self.get_block(pt);
            if self.count_liberties(&my_group).is_empty() {
                rollback(self, &changes);
                return Err(IllegalMoveError::Suicide);
            }
        }

        {
            let my_group = self.get_block(pt);
            let my_libs = self.count_liberties(&my_group);

            if captured.len() == 1 && my_group.len() == 1 && my_libs.len() == 1 {
                if let Some(ko) = ko_point {
                    if pt == ko {
                        rollback(self, &changes);
                        return Err(IllegalMoveError::KoViolation);
                    }
                }
            }

            let new_ko = if captured.len() == 1 && my_group.len() == 1 && my_libs.len() == 1 {
                Some(captured[0])
            } else {
                None
            };

            Ok((captured, new_ko))
        }
    }

    /// 执行着法（不检查合法性）
    ///
    /// 调用者需确保着法合法
    pub fn apply_move_uncheck(&mut self, mv: &Move) -> (Vec<Point>, Option<Point>) {
        if mv.is_pass() {
            return (Vec::new(), None);
        }
        let pt = mv.point.unwrap();
        self.set(pt, mv.color);
        let captured = self.remove_dead_groups(mv);
        let new_ko = if captured.len() == 1 {
            let my_group = self.get_block(pt);
            if my_group.len() == 1 && self.count_liberties(&my_group).len() == 1 {
                Some(captured[0])
            } else {
                None
            }
        } else {
            None
        };
        (captured, new_ko)
    }

    /// 转换为文本格式的棋盘表示
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        fn write_coord(result: &mut String, size: u8) {
            result.push_str("   ");
            for x in 0..size {
                let c = if x >= 8 {
                    (b'A' + x + 1) as char
                } else {
                    (b'A' + x) as char
                };
                result.push(c);
                result.push(' ');
            }
            result.push('\n');
        }
        write_coord(&mut result, self.size);
        for y in 0..self.size {
            result.push_str(&format!("{:2} ", self.size - y));
            for x in 0..self.size {
                let pt = Point { x, y };
                let ch = match self.get(pt) {
                    Some(Color::Black) => "●",
                    Some(Color::White) => "○",
                    None => "+",
                };
                result.push_str(ch);
                result.push(' ');
            }
            result.push_str(&format!(" {}\n", self.size - y));
        }
        write_coord(&mut result, self.size);
        result
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
