use std::collections::{HashSet, VecDeque};

use crate::model::{Board, Color, Point};

/// 眼的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeType {
    /// 真眼：所有对角点都被同色占据或边界
    Real,
    /// 假眼：至少一个对角点被对方占据或为空
    False,
    /// 半眼：需要补一手才能成真眼
    Half,
}

/// 棋子状态（用于死活分析）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupStatus {
    /// 活棋：两只或以上真眼
    Alive,
    /// 死棋：无法做出两只眼
    Dead,
    /// 双活：双方共享气，互不能吃
    Seki,
    /// 未定：需要进一步分析
    Uncertain,
    /// 劫活/劫杀：依赖劫争结果
    Ko,
}

/// 眼位分析结果
#[derive(Debug, Clone)]
pub struct EyeAnalysis {
    /// 棋块所有的眼及类型
    pub eyes: Vec<(Point, EyeType)>,
    /// 真眼数量
    pub real_eye_count: usize,
    /// 棋块状态
    pub status: GroupStatus,
}

/// 棋盘与眼有关的实现
impl Board {
    // ============== 眼位判断核心算法 ==============

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
                None => return false,   // 空点，不受控制
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
        if liberties == 0 {
            GroupStatus::Dead
        } else if liberties <= 2 {
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
            if temp.count_liberties(&opp_group) == 0 {
                // 对方落子自杀，检查我方
                temp.set(lib, color); // 恢复后模拟我方
                let my_group = temp.get_group(lib);
                if temp.count_liberties(&my_group) == 0 {
                    return true; // 双活
                }
            }
        }
        false
    }

    // ============== 目数计算（简化版数子法） ==============

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
