use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

use crate::model::{Color, EyeAnalysis, EyeType, GroupStatus, Move, Point};

/// 棋盘
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
        // 记录最后一手棋
        if let Some(mv) = last_move {
            result.push_str(&format!("\nLast move: {}", mv));
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

    /// 获取对角点（四方向）
    pub fn diagonals(&self, pt: Point) -> Vec<Point> {
        let mut diags = Vec::new();
        let (x, y) = (pt.x as i32, pt.y as i32);
        let dirs = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        for (dx, dy) in dirs {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < self.size as i32 && ny >= 0 && ny < self.size as i32 {
                diags.push(Point {
                    x: nx as u8,
                    y: ny as u8,
                });
            }
        }
        diags
    }

    /// 获取连通块 (相同颜色的相邻棋子)，必须完全连通
    pub fn get_group(&self, pt: Point) -> HashSet<Point> {
        let color = match self.get(pt) {
            Some(c) => c,
            None => return HashSet::new(),
        };
        let mut visited = vec![vec![false; self.size as usize]; self.size as usize];
        let mut queue = VecDeque::new();
        let mut result = HashSet::new();
        
        queue.push_back(pt);
        visited[pt.x as usize][pt.y as usize] = true;
        result.insert(pt);

        // queue-based BFS 遍历连通块
        while let Some(p) = queue.pop_front() {
            for neighbor in self.neighbors(p) {
                let (x,y) = (neighbor.x as usize, neighbor.y as usize);
                if !visited[x][y] && self.get(neighbor) == Some(color) {
                    visited[x][y] = true;
                    result.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        result
    }

    /// 计算连通块的气点
    pub fn count_liberties(&self, group: &HashSet<Point>) -> HashSet<Point> {
        let mut liberties = HashSet::new();
        for &pt in group {
            for nb in self.neighbors(pt) {
                if self.get(nb).is_none() {
                    liberties.insert(nb);
                }
            }
        }
        liberties
    }

    /// 同时收集连通块和气数（优化：避免重复遍历），返回 (连通块, 气点集合)
    /// 注意：调用此方法前应先检查 pt 是否有棋子，否则返回的连通块会是空的，气数也会是0。
    pub fn collect_group_and_liberties(&self, pt: Point) -> (HashSet<Point>, HashSet<Point>) {
        let color = self.get(pt).unwrap();
        let mut group = HashSet::new();
        let mut liberties = HashSet::new();
        let mut queue = VecDeque::new();
        group.insert(pt);
        queue.push_back(pt);

        while let Some(p) = queue.pop_front() {
            for nb in self.neighbors(p) {
                match self.get(nb) {
                    Some(c) if c == color && !group.contains(&nb) => {
                        group.insert(nb);
                        queue.push_back(nb);
                    }
                    None => {
                        liberties.insert(nb);
                    }
                    _ => {}
                }
            }
        }
        (group, liberties)
    }

    /// 移除因落子而死的棋子 (返回被移除的棋子列表)
    pub fn remove_dead_groups_after_move (&mut self, mv: &Move) -> Vec<Point> {
        let mut removed = Vec::new();
        let opponent = mv.color.opposite();

        for nb in self.neighbors(mv.point.unwrap()) {
            if self.get(nb) == Some(opponent) {
                let group = self.get_group(nb);
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

    /// 移除指定颜色所有无气棋子 (返回被移除的棋子列表)
    pub fn remove_dead_groups(&mut self, color: Color) -> Vec<Point> {
        let size = self.size;
        let mut visited = vec![vec![false; size as usize]; size as usize];
        let mut removed = Vec::new();
    
        for x in 0..size {
            for y in 0..size {
                if visited[x as usize][y as usize] { continue; }
                let pt = Point { x, y };
                if let Some(c) = self.get(pt) {
                    if c == color {
                        // BFS 同时标记 group 和计算气数
                        let mut queue = VecDeque::new();
                        let mut group = Vec::new();      // 存储组内点
                        let mut has_liberty = false;
                        queue.push_back(pt);
                        visited[x as usize][y as usize] = true;
                        group.push(pt);
    
                        while let Some(p) = queue.pop_front() {
                            for nb in self.neighbors(p) {
                                if visited[nb.x as usize][nb.y as usize] { continue; }
                                match self.get(nb) {
                                    Some(c2) if c2 == color => {
                                        visited[nb.x as usize][nb.y as usize] = true;
                                        queue.push_back(nb);
                                        group.push(nb);
                                    }
                                    None => has_liberty = true,
                                    _ => {} // 对方棋子，不影响气
                                }
                            }
                        }
    
                        if !has_liberty {
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

    /// 判断某个空点是否为指定颜色的眼
    pub fn analyze_eye(&self, pt: Point, color: Color) -> Option<EyeType> {
        // 1. 必须是空点
        if self.get(pt).is_some() {
            return None;
        }

        // 2. 检查四邻点：必须都是同色棋子或边界
        let neighbors = self.neighbors(pt);
        if neighbors.iter().any(|&nb| self.get(nb) != Some(color)) {
            return Some(EyeType::False);
        }

        // 3. 检查对角点（判断真假眼的关键）
        let diagonals = self.diagonals(pt);
        let mut friendly_corners = 0;
        let mut enemy_corners = 0;

        for diag in diagonals {
            match self.get(diag) {
                Some(c) if c == color => friendly_corners += 1,
                Some(_) => enemy_corners += 1, // 对方棋子
                None => {}                     // 空点，不影响
            }
        }

        // 4. 判断眼型
        // 边界上的眼：只需要3个邻点，对角要求降低
        let is_corner = pt.x == 0 || pt.x == self.size - 1 || pt.y == 0 || pt.y == self.size - 1;
        let is_edge =
            is_corner || pt.x == 0 || pt.x == self.size - 1 || pt.y == 0 || pt.y == self.size - 1;

        if enemy_corners > 0 {
            Some(EyeType::False)
        } else if is_corner && friendly_corners >= 1 {
            Some(EyeType::Real)
        } else if is_edge && friendly_corners >= 2 {
            Some(EyeType::Real)
        } else if !is_edge && friendly_corners >= 3 {
            Some(EyeType::Real)
        } else if friendly_corners >= 2 {
            Some(EyeType::Half)
        } else {
            Some(EyeType::False)
        }
    }

    /// 分析连通块的眼位和死活状态
    pub fn analyze_group(&self, pt: Point) -> Option<EyeAnalysis> {
        let color = self.get(pt)?;
        let group = self.get_group(pt);

        if group.is_empty() {
            return None;
        }

        // 1. 找出所有潜在眼位（空点且被该组包围）
        let mut potential_eyes = HashSet::new();
        for &stone in &group {
            for nb in self.neighbors(stone) {
                if self.get(nb).is_none() {
                    // 检查这个空点是否完全被该组"控制"
                    if self.is_controlled_by(nb, color, &group) {
                        potential_eyes.insert(nb);
                    }
                }
            }
        }

        // 2. 分析每个潜在眼
        let mut eyes = Vec::new();
        let mut real_count = 0;

        for &eye_pt in &potential_eyes {
            if let Some(eye_type) = self.analyze_eye(eye_pt, color) {
                eyes.push((eye_pt, eye_type));
                if eye_type == EyeType::Real {
                    real_count += 1;
                }
            }
        }

        // 3. 判断死活状态
        let status = self.determine_status(&group, color, real_count, &eyes);

        Some(EyeAnalysis {
            eyes,
            real_eye_count: real_count,
            status,
        })
    }

    /// 判断空点是否被指定颜色的组"控制"（用于眼位识别）
    fn is_controlled_by(&self, pt: Point, color: Color, group: &HashSet<Point>) -> bool {
        // 该点的所有邻点必须是同色棋子或边界，且至少一个邻点必须是该组的一部分
        let neighbors = self.neighbors(pt);
        let mut has_group_neighbor = false;

        for nb in neighbors {
            match self.get(nb) {
                Some(c) if c == color => {
                    if group.contains(&nb) {
                        has_group_neighbor = true;
                    }
                }
                Some(_) => return false, // 被对方占据
                None => return false,    // 空点，不受控制
            }
        }

        has_group_neighbor
    }

    /// 根据眼位和外部气判断死活状态
    fn determine_status(
        &self,
        group: &HashSet<Point>,
        color: Color,
        real_eyes: usize,
        eyes: &[(Point, EyeType)],
    ) -> GroupStatus {
        // 两只真眼 = 活棋
        if real_eyes >= 2 {
            return GroupStatus::Alive;
        }

        // 检查是否双活（共享气且双方都无法紧气）
        if self.is_seki(group, color) {
            return GroupStatus::Seki;
        }

        // 一只真眼 + 劫材 = 劫活/劫杀
        if real_eyes == 1 {
            // 简化：检查是否有劫的可能（需要更复杂的劫分析）
            if eyes.iter().any(|(_, t)| *t == EyeType::Half) {
                return GroupStatus::Ko;
            }
        }

        // 无眼或假眼：检查外部气数
        let liberties = self.count_liberties(group);
        if liberties.len() == 0 {
            GroupStatus::Dead
        } else if liberties.len() <= 2 {
            GroupStatus::Uncertain // 需要看谁先走
        } else {
            GroupStatus::Uncertain // 气多，暂时安全
        }
    }

    /// 检测双活（简化版）
    fn is_seki(&self, group: &HashSet<Point>, color: Color) -> bool {
        // 双活条件：
        // 1. 该组与对方组共享气
        // 2. 双方紧气都会自杀

        let opponent = color.opposite();
        let shared_liberties: HashSet<Point> = group
            .iter()
            .flat_map(|&pt| self.neighbors(pt))
            .filter(|&nb| self.get(nb).is_none())
            .collect();

        if shared_liberties.is_empty() {
            return false;
        }

        // 检查每个共享气点：如果对方落子会自杀，且我方落子也会自杀
        for &lib in &shared_liberties {
            // 模拟对方落子
            let mut temp = self.clone();
            temp.set(lib, opponent);
            let opp_group = temp.get_group(lib);
            if temp.count_liberties(&opp_group).len() == 0 {
                // 对方落子自杀，检查我方
                temp.set(lib, color); // 恢复后模拟我方
                let my_group = temp.get_group(lib);
                if temp.count_liberties(&my_group).len() == 0 {
                    return true; // 双活
                }
            }
        }
        false
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
                if temp_board.count_liberties(&group).len() == 0 {
                    captured = true;
                    break;
                }
            }
        }

        // 4. 检查自己的气
        let my_group = temp_board.get_group(pt);
        let my_liberties = temp_board.count_liberties(&my_group);

        // 5. 合法性判断
        if my_liberties.len() > 0 {
            // 有气，合法
            // 劫规则检查
            if !captured && my_group.len() == 1 && my_liberties.len() == 1 {
                if let Some(ko) = ko_point {
                    if pt == ko {
                        return false; // 违反劫规则
                    }
                }
            }
            true
        } else if captured {
            // 无气但能提子
            // 无气能提子时也需要检查劫规则，否则可能出现无气提子但违反劫规则的情况
            if let Some(ko) = ko_point {
                if pt == ko {
                    return false; // 违反劫规则
                }
            }
            true
        } else {
            // 无气且不能提子（自杀）
            // 劫规则不适用于自杀着法，因为自杀着法本身就是非法的（除非允许自杀）
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

        // 1. 落子
        self.set(pt, color);

        // 2. 提掉对方无气的棋子
        let captured = self.remove_dead_groups_after_move(mv);

        // 3. 计算劫点 (简单劫：提单子且自己被提后也是单子)
        let new_ko = if captured.len() == 1 {
            let my_group = self.get_group(pt);
            if my_group.len() == 1 && self.count_liberties(&my_group).len() == 1 {
                Some(captured[0]) // 被提的位置是潜在的劫点
            } else {
                None
            }
        } else {
            None
        };

        Some((captured, new_ko))
    }

    /// 计算指定颜色的棋子数量
    ///
    /// 不区分死活，纯粹统计棋盘上存在的棋子数量。
    pub fn count_stones(&self, color: Color) -> usize {
        self.grid
            .iter()
            .flatten()
            .filter(|&&c| c == Some(color))
            .count()
    }

    /// 计算指定颜色的"地盘"（简化：空点 + 活棋）
    pub fn count_territory(&self, color: Color) -> usize {
        let mut territory = 0;
        let mut visited = HashSet::new();

        for y in 0..self.size {
            for x in 0..self.size {
                let pt = Point { x, y };
                if self.get(pt).is_none() && !visited.contains(&pt) {
                    // BFS 找出连通空区域
                    let mut region = HashSet::new();
                    let mut queue = VecDeque::new();
                    queue.push_back(pt);
                    region.insert(pt);
                    visited.insert(pt);

                    let mut borders = HashSet::new(); // 区域边界颜色

                    while let Some(p) = queue.pop_front() {
                        for nb in self.neighbors(p) {
                            if !visited.contains(&nb) {
                                match self.get(nb) {
                                    None => {
                                        visited.insert(nb);
                                        region.insert(nb);
                                        queue.push_back(nb);
                                    }
                                    Some(c) => {
                                        borders.insert(c);
                                    }
                                }
                            }
                        }
                    }

                    // 如果区域只被一种颜色包围，算作该方地盘
                    if borders.len() == 1 && borders.contains(&color) {
                        territory += region.len();
                    }
                }
            }
        }

        // 加上活棋数量
        territory + self.count_stones(color)
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
    fn test_setup_board() {
        let stones = vec![
            (Point { x: 4, y: 4 }, Color::Black),
            (Point { x: 3, y: 9 }, Color::White),
        ];
        let board = Board::from_setup(9, &stones);
        assert_eq!(board.count_stones(Color::Black), 1);
        assert_eq!(board.count_stones(Color::White), 0);
    }

    /// 测试棋盘基本功能：放置棋子、提子、合法性检查
    #[test]
    fn test_basic_board_operations() {
        let mut board = Board::new(5);
        assert!(board.is_empty(Point { x: 2, y: 2 }));
        board.set(Point { x: 2, y: 2 }, Color::Black);
        assert_eq!(board.get(Point { x: 2, y: 2 }), Some(Color::Black));
        board.remove(Point { x: 2, y: 2 });
        assert!(board.is_empty(Point { x: 2, y: 2 }));
    }

    /// 测试相邻点
    #[test]
    fn test_neighbors() {
        //点位于中间
        let board = Board::new(5);
        let pt = Point { x: 2, y: 2 };
        let neighbors = board.neighbors(pt);
        let expected = vec![
            Point { x: 2, y: 3 },
            Point { x: 2, y: 1 },
            Point { x: 3, y: 2 },
            Point { x: 1, y: 2 },
        ];
        assert_eq!(neighbors.len(), expected.len());
        for nb in expected {
            assert!(neighbors.contains(&nb));
        }

        //点位于边界
        let pt = Point { x: 1, y: 0 };
        let neighbors = board.neighbors(pt);
        let expected = vec![
            Point { x: 0, y: 0 },
            Point { x: 1, y: 1 },
            Point { x: 2, y: 0 },
        ];
        assert_eq!(neighbors.len(), expected.len());
        for nb in expected {
            assert!(neighbors.contains(&nb));
        }

        //点位于角落
        let pt = Point { x: 4, y: 4 };
        let neighbors = board.neighbors(pt);
        let expected = vec![Point { x: 3, y: 4 }, Point { x: 4, y: 3 }];
        assert_eq!(neighbors.len(), expected.len());
        for nb in expected {
            assert!(neighbors.contains(&nb));
        }
    }

    /// 测试对角点
    #[test]
    fn test_diagonals() {
        //点位于中间
        let board = Board::new(5);
        let pt = Point { x: 2, y: 2 };
        let diagonals = board.diagonals(pt);
        let expected = vec![
            Point { x: 1, y: 1 },
            Point { x: 3, y: 1 },
            Point { x: 3, y: 3 },
            Point { x: 1, y: 3 },
        ];
        assert_eq!(diagonals.len(), expected.len());
        for nb in expected {
            assert!(diagonals.contains(&nb));
        }

        //点位于边界
        let pt = Point { x: 1, y: 0 };
        let diagonals = board.diagonals(pt);
        let expected = vec![
            Point { x: 0, y: 1 },
            Point { x: 2, y: 1 },
        ];
        assert_eq!(diagonals.len(), expected.len());
        for nb in expected {
            assert!(diagonals.contains(&nb));
        }

        //点位于角落
        let pt = Point { x: 4, y: 4 };
        let diagonals = board.diagonals(pt);
        let expected = vec![Point { x: 3, y: 3 }];
        assert_eq!(diagonals.len(), expected.len());
        for nb in expected {
            assert!(diagonals.contains(&nb));
        }
    }

    /// 测试连通块
    #[test]
    fn test_get_group() {
        let mut board = Board::new(5);
        // 创建一个简单的连通块
        board.set(Point { x: 1, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 2, y: 1 }, Color::Black);

        let group = board.get_group(Point { x: 1, y: 1 });
        let expected = vec![
            Point { x: 1, y: 1 },
            Point { x: 1, y: 2 },
            Point { x: 2, y: 1 },
        ];
        assert_eq!(group.len(), expected.len());
        for pt in expected {
            assert!(group.contains(&pt));
        }
    }

    /// 测试基本提子逻辑
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
        assert_eq!(board.count_liberties(&group).len(), 4);

        // 连接后气数减少
        board.set(Point { x: 2, y: 3 }, Color::Black);
        let group = board.get_group(pt);
        assert_eq!(group.len(), 2);
        assert_eq!(board.count_liberties(&group).len(), 6); // 2*4 - 2(共享边) = 6
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
        println!("Initial board:\n{}\n", board);

        // 白提黑一子
        let mv_white = Move::new(Color::White, Point { x: 1, y: 1 }, 5).unwrap();
        let (mv_black, ko_point) = board.apply_move(&mv_white, None, false).unwrap();
        println!("{}", board.to_string_with_moves(Some(mv_white)));

        assert_eq!(ko_point, Some(Point { x: 2, y: 1 }));
        assert_eq!(mv_black.len(), 1);
        assert_eq!(mv_black[0], Point { x: 2, y: 1 });

        // 黑不能立即回提（劫）
        let mv_black = Move::new(Color::Black, Point { x: 2, y: 1 }, 5).unwrap();
        assert!(!board.is_legal(&mv_black, ko_point, false));
        let res = board.apply_move(&mv_black, ko_point, false);
        assert!(res.is_none()); //黑应该不能立即回提（劫）
        println!("棋盘应该没有变化：\n{}\n", board.to_string_with_moves(None));

        // 黑先走其他位置,白应一手棋
        let mv_black_other = Move::new(Color::Black, Point { x: 0, y: 0 }, 5).unwrap();
        let mv_white_other = Move::new(Color::White, Point { x: 3, y: 0 }, 5).unwrap();
        assert!(board.is_legal(&mv_black_other, ko_point, false));
        let (_, ko_point) = board.apply_move(&mv_black_other, ko_point, false).unwrap();
        println!("{}", board.to_string_with_moves(Some(mv_black_other)));

        assert!(board.is_legal(&mv_white_other, ko_point, false));
        let (_, ko_point) = board.apply_move(&mv_white_other, ko_point, false).unwrap();
        println!("{}", board.to_string_with_moves(Some(mv_white_other)));

        // 现在黑可以回提了
        assert_eq!(ko_point, None);
        assert!(board.is_legal(&mv_black, ko_point, false));
        let (captured_stones, ko_point) = board.apply_move(&mv_black, ko_point, false).unwrap();
        assert_eq!(captured_stones[0], Point { x: 1, y: 1 });
        // 回提后新的劫点是之前被提的位置
        assert_eq!(ko_point.unwrap(), Point { x: 1, y: 1 });
    }

    

    #[test]
    fn test_eye_analysis() {
        let mut board = Board::new(5);
        // 角落的空点（0,0）
        board.set(Point { x: 0, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 0 }, Color::Black);
        // 中间的空点（1,1）
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 2, y: 1 }, Color::Black);
        board.set(Point { x: 2, y: 2 }, Color::Black);
        // 边上的空点（2,0）
        board.set(Point { x: 3, y: 0 }, Color::Black);
        board.set(Point { x: 3, y: 1 }, Color::White);

        let eye1 = Point { x: 0, y: 0 };
        let eye2 = Point { x: 1, y: 1 };
        let eye3 = Point { x: 2, y: 0 };

        // 中间的空点是一个真眼
        let eye_type = board.analyze_eye(eye3, Color::Black);
        assert_eq!(eye_type, Some(EyeType::Real));

        // 边上的空点是假眼
        let eye_type = board.analyze_eye(eye2, Color::Black);
        assert_eq!(eye_type, Some(EyeType::Real));

        // 角落的空点是真眼
        let eye_type = board.analyze_eye(eye1, Color::Black);
        assert_eq!(eye_type, Some(EyeType::False));
    }

}
