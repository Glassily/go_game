use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use crate::model::{Color, EmptyRegion, EyeAnalysis, EyeType, GroupSet, GroupStatus, Move, Point};

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
    pub fn from_setup(
        size: u8,
        stones: impl IntoIterator<Item = impl Borrow<(Point, Color)>>,
    ) -> Self {
        let mut board = Self::new(size);
        for item in stones {
            let (pt, color) = *item.borrow();
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
    pub fn to_string_gtp(&self) -> String {
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

    /// 可视化棋盘（文本格式）
    pub fn to_string_with_move(&self, last_move: Move) -> String {
        let mut result = String::new();
        self.write_coord(&mut result);

        // 棋盘内容
        for y in 0..self.size {
            result.push_str(&format!("{:2} ", self.size - y));
            for x in 0..self.size {
                let pt = Point { x, y };
                // 判断last_move
                if let Some(mv_pt) = last_move.point {
                    if pt == mv_pt {
                        result.push('*');
                        result.push(' ');
                        continue;
                    }
                }
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
        // 记录最后一手棋
        if last_move.point.is_some() {
            result.push_str(&format!(
                "\nLast move: {}\n",
                last_move.to_string_gtp(self.size)
            ));
        }
        result
    }

    /// 可视化棋盘（文本格式），标记会覆盖棋子
    pub fn to_string_with_labels(&self, labels: Vec<Point>) -> String {
        let mut result = String::new();
        self.write_coord(&mut result);
        // 棋盘内容
        for y in 0..self.size {
            result.push_str(&format!("{:2} ", self.size - y));
            for x in 0..self.size {
                let pt = Point { x, y };
                let ch = if labels.contains(&pt) {
                    "∆"
                } else {
                    match self.get(pt) {
                        Some(Color::Black) => "●",
                        Some(Color::White) => "○",
                        None => "+",
                    }
                };
                result.push_str(ch);
                result.push(' ');
            }
            result.push_str(&format!(" {}\n", self.size - y));
        }
        self.write_coord(&mut result);
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
    pub fn get_block(&self, pt: Point) -> HashSet<Point> {
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
                let (x, y) = (neighbor.x as usize, neighbor.y as usize);
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
    pub fn count_liberties(&self, block: &HashSet<Point>) -> HashSet<Point> {
        let mut liberties = HashSet::new();
        for &pt in block {
            for nb in self.neighbors(pt) {
                if self.get(nb).is_none() {
                    liberties.insert(nb);
                }
            }
        }
        liberties
    }

    /// 同时收集连通块和气数，返回 (连通块, 气点集合)
    pub fn collect_block_and_liberties(&self, pt: Point) -> (HashSet<Point>, HashSet<Point>) {
        let color = self.get(pt).unwrap();
        let mut block = HashSet::new();
        let mut liberties = HashSet::new();
        let mut queue = VecDeque::new();
        block.insert(pt);
        queue.push_back(pt);

        while let Some(p) = queue.pop_front() {
            for nb in self.neighbors(p) {
                match self.get(nb) {
                    Some(c) if c == color && !block.contains(&nb) => {
                        block.insert(nb);
                        queue.push_back(nb);
                    }
                    None => {
                        liberties.insert(nb);
                    }
                    _ => {}
                }
            }
        }
        (block, liberties)
    }

    /// 获取所有连通块，返回(颜色，连通块)
    pub fn all_blocks(&self) -> Vec<(Color, HashSet<Point>)> {
        let mut visited: HashSet<Point> = HashSet::new();
        let mut blocks = Vec::new();
        for x in 0..self.size {
            for y in 0..self.size {
                let pt = Point { x, y };
                if let Some(color) = self.get(pt) {
                    if !visited.contains(&pt) {
                        let block = self.get_block(pt);
                        visited.extend(block.iter());
                        blocks.push((color, block));
                    }
                }
            }
        }
        blocks
    }

    /// 获取所有连通块，返回(颜色，连通块，气点)
    pub fn collect_blocks_and_liberties(&self) -> Vec<(Color, HashSet<Point>, HashSet<Point>)> {
        let mut visited: HashSet<Point> = HashSet::new();
        let mut blocks = Vec::new();
        for x in 0..self.size {
            for y in 0..self.size {
                let pt = Point { x, y };
                if let Some(color) = self.get(pt) {
                    if !visited.contains(&pt) {
                        let (block, liberties) = self.collect_block_and_liberties(pt);
                        visited.extend(block.iter());
                        blocks.push((color, block, liberties));
                    }
                }
            }
        }
        blocks
    }

    /// 将同色连通块合并成群（基于共享气）
    pub fn merge_blocks_into_groups(&self) -> Vec<GroupSet> {
        let blocks = self.all_blocks();
        let n = blocks.len();
        if n == 0 {
            return vec![];
        }

        // 并查集
        let mut parent = (0..n).collect::<Vec<_>>();
        fn find(parent: &mut Vec<usize>, x: usize) -> usize {
            if parent[x] != x {
                parent[x] = find(parent, parent[x]);
            }
            parent[x]
        }
        fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent[rb] = ra;
            }
        }

        // 遍历所有空点，收集邻接的同色块，合并它们
        for x in 0..self.size {
            for y in 0..self.size {
                let pt = Point { x, y };
                if self.get(pt).is_some() {
                    continue;
                }

                // 收集该空点邻接的所有同色块索引
                let mut adj_blocks = Vec::new();
                let mut first_color = None;
                for (idx, (color, block)) in blocks.iter().enumerate() {
                    if block.iter().any(|&p| self.neighbors(p).contains(&pt)) {
                        match first_color {
                            None => {
                                // 记录第一个块的颜色
                                first_color = Some(*color);
                                adj_blocks.push(idx);
                            }
                            Some(c) if c == *color => {
                                // 同色才加入
                                adj_blocks.push(idx);
                            }
                            _ => {} // 不同色直接跳过
                        }
                    }
                }
                // 合并这些块,只有同色块数量≥2时才合并
                if adj_blocks.len() > 1 {
                    let first = adj_blocks[0];
                    for &other in &adj_blocks[1..] {
                        union(&mut parent, first, other);
                    }
                }
            }
        }
        // 聚合合并后的群
        let mut groups: HashMap<usize, GroupSet> = HashMap::new();
        for (idx, (color, block)) in blocks.into_iter().enumerate() {
            let root = find(&mut parent, idx);
            let entry = groups.entry(root).or_insert_with(|| GroupSet {
                color,
                points: HashSet::new(),
                liberties: HashSet::new(),
            });
            entry.points.extend(block);
        }

        // 重新计算每个群的 liberties（所有块的气的并集）
        for group in groups.values_mut() {
            let mut libs = HashSet::new();
            for &pt in &group.points {
                for nb in self.neighbors(pt) {
                    if self.get(nb).is_none() {
                        libs.insert(nb);
                    }
                }
            }
            group.liberties = libs;
        }
        groups.into_values().collect()
    }

    /// 找出所有连通空区域（包括外部区域）
    pub fn find_empty_regions(&self) -> Vec<EmptyRegion> {
        let mut visited = HashSet::new();
        let mut regions = Vec::new();

        for x in 0..self.size {
            for y in 0..self.size {
                let pt = Point { x, y };
                if self.get(pt).is_some() || visited.contains(&pt) {
                    continue;
                }

                // BFS 收集连通空点
                let mut queue = VecDeque::new();
                let mut region_points = HashSet::new();
                let mut border_colors = HashSet::new();
                let mut touches_edge = false;

                queue.push_back(pt);
                visited.insert(pt);
                region_points.insert(pt);

                while let Some(p) = queue.pop_front() {
                    // 检查是否触及边界
                    if p.x == 0 || p.x == self.size - 1 || p.y == 0 || p.y == self.size - 1 {
                        touches_edge = true;
                    }

                    for nb in self.neighbors(p) {
                        match self.get(nb) {
                            None => {
                                if !visited.contains(&nb) {
                                    visited.insert(nb);
                                    region_points.insert(nb);
                                    queue.push_back(nb);
                                }
                            }
                            Some(c) => {
                                border_colors.insert(c);
                            }
                        }
                    }
                }

                regions.push(EmptyRegion {
                    points: region_points,
                    border_colors,
                    touches_edge,
                });
            }
        }

        regions
    }

    /// 仅获取完全被棋子包围的内部空区域（不接触边界）
    pub fn find_internal_empty_regions(&self) -> Vec<EmptyRegion> {
        self.find_empty_regions()
            .into_iter()
            .filter(|r| !r.touches_edge)
            .collect()
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

    /// 移除指定颜色所有无气棋子 (返回被移除的棋子列表)
    pub fn remove_all_dead(&mut self, color: Color) -> Vec<Point> {
        let size = self.size;
        let mut visited = vec![vec![false; size as usize]; size as usize];
        let mut removed = Vec::new();

        for x in 0..size {
            for y in 0..size {
                if visited[x as usize][y as usize] {
                    continue;
                }
                let pt = Point { x, y };
                if let Some(c) = self.get(pt) {
                    if c == color {
                        // BFS 同时标记 group 和计算气数
                        let mut queue = VecDeque::new();
                        let mut group = Vec::new(); // 存储组内点
                        let mut has_liberty = false;
                        queue.push_back(pt);
                        visited[x as usize][y as usize] = true;
                        group.push(pt);

                        while let Some(p) = queue.pop_front() {
                            for nb in self.neighbors(p) {
                                if visited[nb.x as usize][nb.y as usize] {
                                    continue;
                                }
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

    /// 分析整个棋盘上所有块群的状态
    pub fn analyze_all_groups(&self) -> Vec<(GroupSet, GroupStatus)> {
        let groups = self.merge_blocks_into_groups();
        let internal_regions = self.find_internal_empty_regions();
        groups
            .into_iter()
            .map(|group| {
                // 找出该群包围的内部空区域
                let my_regions: Vec<&EmptyRegion> = internal_regions
                    .iter()
                    .filter(|r| {
                        r.border_colors.len() == 1 && r.border_colors.contains(&group.color)
                    })
                    .collect();
                // 单点真眼简化
                let real_eyes = my_regions.iter().filter(|r| r.points.len() == 1).count();
                let status = if real_eyes >= 2 {
                    GroupStatus::Alive
                } else {
                    // 需要更复杂的判断...
                    GroupStatus::Uncertain
                };
                (group, status)
            })
            .collect()
    }

    /// 判断某个空点是否为指定颜色的眼
    pub fn analyze_eye(&self, pt: Point, color: Color) -> Option<EyeType> {
        // 1. 必须是空点
        if self.get(pt).is_some() {
            return None;
        }

        // 2. 检查四邻点：必须都是同色棋子或边界，不能有对方棋子，也不能有空点
        let neighbors = self.neighbors(pt);
        if neighbors.iter().any(|&nb| self.get(nb) != Some(color)) {
            return Some(EyeType::False);
        }

        // 3. 检查对角点（判断真假眼的关键）
        let diagonals = self.diagonals(pt);



        // 3. 根据位置（角/边/中央）确定所需对角己方棋子数量
        let nb_count = neighbors.len(); // 2 角，3 边，4 中央
        let required_diagonals = match nb_count {
            2 => 1, // 角部：至少一个对角是己方
            3 => 2, // 边上：至少两个对角是己方
            4 => 3, // 中央：至少三个对角是己方
            _ => return Some(EyeType::False),
        };

        // 4. 判断眼型
        let my_diagonals = diagonals
            .iter()
            .filter(|&&d| self.get(d) == Some(color))
            .count();

        if my_diagonals >= required_diagonals {
            Some(EyeType::Real)
        } else {
            Some(EyeType::False)
        }
    }

    /// 分析连通块的眼位和死活状态
    pub fn analyze_group(&self, pt: Point) -> Option<EyeAnalysis> {
        let color = self.get(pt)?;
        let group = self.get_block(pt);

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
            let opp_group = temp.get_block(lib);
            if temp.count_liberties(&opp_group).len() == 0 {
                // 对方落子自杀，检查我方
                temp.set(lib, color); // 恢复后模拟我方
                let my_group = temp.get_block(lib);
                if temp.count_liberties(&my_group).len() == 0 {
                    return true; // 双活
                }
            }
        }
        false
    }

    /// 检查落子合法性
    pub fn is_legal(&self, mv: &Move, ko_point: Option<Point>, allow_suicide: bool) -> bool {
        if mv.is_pass() {
            return true; // Pass 总是合法的
        }

        let pt = mv.point.unwrap();

        // 1. 位置必须为空
        if !self.get(pt).is_none() {
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
                let group = temp_board.get_block(nb);
                if temp_board.count_liberties(&group).len() == 0 {
                    captured = true;
                    break;
                }
            }
        }

        // 4. 检查自己的气
        let my_group = temp_board.get_block(pt);
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

    /// 执行着法，返回: (被提掉的棋子, 新的劫点)
    pub fn apply_move(
        &mut self,
        mv: &Move,
        ko_point: Option<Point>,
        allow_suicide: bool,
    ) -> Option<(Vec<Point>, Option<Point>)> {
        if !self.is_legal(mv, ko_point, allow_suicide) {
            // todo:返回错误类型
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
        let captured = self.remove_dead_groups(mv);

        // 3. 计算劫点 (简单劫：提单子且自己被提后也是单子)
        let new_ko = if captured.len() == 1 {
            let my_group = self.get_block(pt);
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

    /// 打印横坐标ABCD..
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

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut result = String::new();
        self.write_coord(&mut result);
        // 棋盘内容
        for y in 0..self.size {
            result.push_str(&format!("{:2} ", y + 1));
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
            result.push_str(&format!(" {}\n", y + 1));
        }
        self.write_coord(&mut result);
        write!(f, "{}", result)
    }
}
