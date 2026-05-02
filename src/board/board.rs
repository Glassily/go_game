use crate::model::{Color, Move, Point};
use std::collections::{HashSet, VecDeque};

/// 围棋棋盘
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub size: u8,
    grid: Vec<Vec<Option<Color>>>,
}

/// 落子非法原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IllegalMoveError {
    InvalidMove,
    OutOfBounds,
    Occupied,
    KoViolation,
    Suicide,
}

impl Board {
    pub fn new(size: u8) -> Self {
        assert!((2..=25).contains(&size), "Board size must be 2-25");
        Self {
            size,
            grid: vec![vec![None; size as usize]; size as usize],
        }
    }

    pub fn from_setup(size: u8, stones: &[(Point, Color)]) -> Self {
        let mut board = Self::new(size);
        for &(pt, color) in stones {
            if pt.is_valid(size) {
                board.grid[pt.y as usize][pt.x as usize] = Some(color);
            }
        }
        board
    }

    pub fn get(&self, pt: Point) -> Option<Color> {
        pt.is_valid(self.size)
            .then(|| self.grid[pt.y as usize][pt.x as usize])?
    }

    pub fn set(&mut self, pt: Point, color: Color) {
        if pt.is_valid(self.size) {
            self.grid[pt.y as usize][pt.x as usize] = Some(color);
        }
    }

    pub fn remove(&mut self, pt: Point) {
        if pt.is_valid(self.size) {
            self.grid[pt.y as usize][pt.x as usize] = None;
        }
    }

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

    /// BFS 获取连通块
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

    /// 计算连通块的气
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

    /// 移除因落子而死的棋子 (返回被移除的棋子列表)
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
    pub fn is_legal(
        &self,
        mv: &Move,
        ko_point: Option<Point>,
        allow_suicide: bool,
    ) -> Result<(), IllegalMoveError> {
        // Pass 总是合法
        if mv.is_pass() {
            return Ok(());
        }

        let pt = mv.point.ok_or(IllegalMoveError::InvalidMove)?;

        // 1. 位置合法性
        if !pt.is_valid(self.size) {
            return Err(IllegalMoveError::OutOfBounds);
        }
        if !self.get(pt).is_none() {
            return Err(IllegalMoveError::Occupied);
        }

        // 2. 临时棋盘模拟
        let mut temp = self.clone();
        temp.set(pt, mv.color);

        // 模拟提子
        let temp_mv = Move {
            point: Some(pt),
            color: mv.color,
        };
        let captured = temp.remove_dead_groups(&temp_mv);

        // 计算自己连通块和气
        let my_group = temp.get_block(pt);
        let my_liberties = temp.count_liberties(&my_group);

        // 3. 自杀检查
        if my_liberties.is_empty() && captured.is_empty() {
            if !allow_suicide {
                return Err(IllegalMoveError::Suicide);
            }
        }

        // 4. 统一的劫检查：提一子 + 己方单子一口气 + 落子点正好是劫点
        if captured.len() == 1 && my_group.len() == 1 && my_liberties.len() == 1 {
            if let Some(ko) = ko_point {
                if pt == ko {
                    return Err(IllegalMoveError::KoViolation);
                }
            }
        }

        Ok(())
    }

    /// 执行着法，返回: (被提掉的棋子, 新的劫点)
    /// 
    /// 带规则检测（原位操作，失败则回滚）
    pub fn apply_move(
        &mut self,
        mv: &Move,
        ko_point: Option<Point>,
        allow_suicide: bool,
    ) -> Result<(Vec<Point>, Option<Point>), IllegalMoveError> {
        // Pass 直接返回
        if mv.is_pass() {
            return Ok((Vec::new(), None));
        }

        let pt = mv.point.ok_or(IllegalMoveError::InvalidMove)?;
        let color = mv.color;

        // 基本合法性（不修改状态）
        if !pt.is_valid(self.size) {
            return Err(IllegalMoveError::OutOfBounds);
        }
        if !self.get(pt).is_none() {
            return Err(IllegalMoveError::Occupied);
        }

        // 记录所有修改 (点 -> 旧值) 以便回滚
        let mut changes: Vec<(Point, Option<Color>)> = Vec::new();

        // 记录并落子
        changes.push((pt, None)); // 原来必为空
        self.set(pt, color);

        // 找出所有相邻且被提走的对方块
        let opponent = color.opposite();
        let mut groups_to_remove: Vec<(HashSet<Point>, Color)> = Vec::new();

        for nb in self.neighbors(pt) {
            if self.get(nb) == Some(opponent) {
                let group = self.get_block(nb);
                if self.count_liberties(&group).is_empty() {
                    // 避免重复加入同一块
                    if !groups_to_remove.iter().any(|(g, _)| g.contains(&nb)) {
                        groups_to_remove.push((group, opponent));
                    }
                }
            }
        }

        // 移除死子，同时记录
        let mut captured = Vec::new();
        for (group, col) in &groups_to_remove {
            for &p in group {
                captured.push(p);
                changes.push((p, Some(*col)));
                self.remove(p);
            }
        }

        // 检查自杀（若不允许）
        if !allow_suicide {
            let my_group = self.get_block(pt);
            if self.count_liberties(&my_group).is_empty() {
                // 回滚
                self.rollback(&changes);
                return Err(IllegalMoveError::Suicide);
            }
        }

        // 劫检查
        {
            let my_group = self.get_block(pt);
            let my_libs = self.count_liberties(&my_group);

            if captured.len() == 1 && my_group.len() == 1 && my_libs.len() == 1 {
                if let Some(ko) = ko_point {
                    if pt == ko {
                        // 回滚
                        self.rollback(&changes);
                        return Err(IllegalMoveError::KoViolation);
                    }
                }
            }

            // 计算新劫点（仅当合法时）
            let new_ko = if captured.len() == 1 && my_group.len() == 1 && my_libs.len() == 1 {
                Some(captured[0])
            } else {
                None
            };

            Ok((captured, new_ko))
        }
    }

    /// 辅助函数：回滚棋盘修改
    fn rollback(&mut self, changes: &[(Point, Option<Color>)]) {
        // p：更改的位置，old：位置上原来的颜色
        for &(p, old) in changes {
            match old {
                Some(c) => self.set(p, c),
                None => self.remove(p),
            }
        }
    }

    /// 执行着法，返回: (被提掉的棋子, 新的劫点)
    pub fn apply_move_uncheck(&mut self, mv: &Move) -> (Vec<Point>, Option<Point>) {
        if mv.is_pass() {
            return (Vec::new(), None);
        }
        let pt = mv.point.unwrap(); // 调用者保证
        self.set(pt, mv.color);
        let captured = self.remove_dead_groups(mv);
        // 劫点
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

    /// 文本可视化
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        self.write_coord(&mut result);
        // 棋盘内容
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
        self.write_coord(&mut result);
        result
    }

    fn write_coord(&self, s: &mut String) {
        s.push_str("   ");
        for x in 0..self.size {
            let c = if x >= 8 {
                (b'A' + x + 1) as char
            } else {
                (b'A' + x) as char
            };
            s.push(c);
            s.push(' ');
        }
        s.push('\n');
    }
}
