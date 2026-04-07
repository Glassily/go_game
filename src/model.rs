use std::{collections::HashMap, fmt::Display};

/// 棋盘坐标 (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u8,
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

/// 棋子颜色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
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

/// 一手棋 (None 表示 Pass)
///
/// ⚠️ 注意: `color` 字段表示该着法的执行方（由 SGF 的 B[]/W[] 决定），
/// 而非游戏状态的"当前轮到谁"。业务逻辑中请通过 GoGameRecord 维护回合状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub color: Color,
    pub point: Option<Point>,
}

impl Move {
    /// 创建 Pass 着法
    pub fn pass(color: Color) -> Self {
        Self { color, point: None }
    }

    /// 创建落子着法（带边界验证）
    pub fn new(color: Color, point: Point, board_size: u8) -> Option<Self> {
        if point.is_valid(board_size) {
            Some(Self {
                color,
                point: Some(point),
            })
        } else {
            None
        }
    }

    /// 是否为 Pass
    pub fn is_pass(&self) -> bool {
        self.point.is_none()
    }
}

/// 节点属性 (对应 SGF 的 C, LB, AB, AW, 自定义属性等)
#[derive(Debug, Clone, Default)]
pub struct NodeProperties {
    /// 评论
    pub comment: String,
    /// 标注，如 "A1" -> "正解"
    pub labels: HashMap<Point, String>,
    /// 让子/编辑摆子
    pub setup: Vec<(Point, Color)>,
    /// 自定义注解
    pub annotations: Vec<String>,
    /// 保留原始 SGF 属性防丢失（保证无损往返）
    pub raw_sgf_props: HashMap<String, Vec<String>>,
}

/// 树节点 (索引化设计，无循环引用与借用问题)
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub parent_index: Option<usize>,
    pub children: Vec<usize>,
    pub move_data: Option<Move>,
    pub props: NodeProperties,
}

/// 棋谱树
#[derive(Debug, Clone)]
pub struct GameTree {
    pub nodes: Vec<TreeNode>,
    pub root_index: usize,
}

impl GameTree {
    pub fn new(root_props: NodeProperties) -> Self {
        let root = TreeNode {
            parent_index: None,
            children: vec![],
            move_data: None,
            props: root_props,
        };
        Self {
            nodes: vec![root],
            root_index: 0,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&TreeNode> {
        self.nodes.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut TreeNode> {
        self.nodes.get_mut(idx)
    }

    /// 添加子节点（变招）
    pub fn add_child(
        &mut self,
        parent_idx: usize,
        move_data: Option<Move>,
        props: NodeProperties,
    ) -> usize {
        //创建子节点索引，按加入nodes的顺序创建
        let new_idx = self.nodes.len();
        let child = TreeNode {
            parent_index: Some(parent_idx),
            children: vec![],
            move_data,
            props,
        };
        //将子节点加入nodes
        self.nodes.push(child);
        //在父节点添加子节点索引
        self.nodes[parent_idx].children.push(new_idx);
        new_idx
    }

    /// 获取指定节点的子节点迭代器
    pub fn children(&self, idx: usize) -> impl Iterator<Item = &TreeNode> + '_ {
        self.nodes[idx]
            .children
            .iter()
            .filter_map(|&child_idx| self.get(child_idx))
    }
}

impl Default for GameTree {
    fn default() -> Self {
        // 手动实现 Default，避免 root_index=0 但 nodes 为空的非法状态
        Self::new(NodeProperties::default())
    }
}

/// 棋局数据
#[derive(Debug, Clone, Default)]
pub struct GameInfo {
    pub black_name: String,
    pub white_name: String,
    pub komi: f32,
    pub handicap: u8,
    pub result: String, // e.g., "B+R", "W+1.5", "Draw"
    pub date: String,   // e.g., "2024-10-01"
    pub board_size: u8, // 通常 19
    pub rules: String,  // "Japanese", "Chinese", "AGA"
}

/// 完整打谱记录
#[derive(Debug, Clone)]
pub struct GoGameRecord {
    pub info: GameInfo,
    pub tree: GameTree,
    /// 当前路径：从根节点到当前位置的索引序列
    pub current_path: Vec<usize>,
}

impl GoGameRecord {
    pub fn new(info: GameInfo) -> Self {
        Self {
            info,
            tree: GameTree::default(),
            current_path: vec![0],
        }
    }

    pub fn current_node(&self) -> Option<&TreeNode> {
        self.current_path.last().and_then(|&idx| self.tree.get(idx))
    }

    pub fn current_node_mut(&mut self) -> Option<&mut TreeNode> {
        self.current_path
            .last()
            .and_then(|&idx| self.tree.get_mut(idx))
    }

    /// 前进到指定子节点（切换变招）
    pub fn move_to_child(&mut self, child_idx: usize) -> bool {
        if let Some(current) = self.current_path.last() {
            if let Some(node) = self.tree.get(*current) {
                if node.children.contains(&child_idx) {
                    self.current_path.push(child_idx);
                    return true;
                }
            }
        }
        false
    }

    pub fn move_to_parent(&mut self) -> bool {
        if self.current_path.len() > 1 {
            self.current_path.pop();
            true
        } else {
            false
        }
    }

    pub fn reset_to_root(&mut self) {
        self.current_path = vec![self.tree.root_index];
    }

    /// 获取当前应落子颜色（根据已有着法推断）
    pub fn current_turn(&self) -> Option<Color> {
        let mut turn = if self.info.handicap > 1 {
            Color::White
        } else {
            Color::Black
        };
        for &idx in &self.current_path {
            if let Some(node) = self.tree.get(idx) {
                if let Some(mv) = &node.move_data {
                    turn = mv.color.opposite();
                }
            }
        }
        Some(turn)
    }
}

impl Default for GoGameRecord {
    fn default() -> Self {
        Self::new(GameInfo::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sgf_coord_roundtrip() {
        for c in "abcdefghjklmnopqrstuvwxyz".chars() {
            if let Some(val) = Point::from_sgf_coord_char(c) {
                let pt = Point { x: val, y: 0 };
                let result = pt.to_sgf();
                assert_eq!(result.chars().next(), Some(c));
            }
        }
    }

    #[test]
    fn test_gametree_default_safe() {
        let tree = GameTree::default();
        assert!(!tree.nodes.is_empty());
        assert_eq!(tree.root_index, 0);
    }

    #[test]
    fn test_point_validation() {
        assert!(Point::new(0, 0, 19).is_some());
        assert!(Point::new(18, 18, 19).is_some());
        assert!(Point::new(19, 0, 19).is_none());
        assert!(Point::from_sgf("aa", 19).is_some());
        assert!(Point::from_sgf("tt", 19).is_none()); // t=18, 19x19 棋盘最大索引 18
    }
}
