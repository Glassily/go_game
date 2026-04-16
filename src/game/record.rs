use std::collections::HashMap;

use crate::model::{Board, Color, Move, Point};

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

    /// 获取指定节点的父节点
    pub fn parent(&self, idx: usize) -> Option<&TreeNode> {
        self.nodes[idx]
            .parent_index
            .and_then(|parent_idx| self.get(parent_idx))
    }

    /// 获取指定节点的兄弟节点（同父节点的其他子节点）
    pub fn siblings(&self, idx: usize) -> Vec<&TreeNode> {
        if let Some(parent_idx) = self.nodes[idx].parent_index {
            self.nodes[parent_idx]
                .children
                .iter()
                .filter_map(|&sibling_idx| {
                    if sibling_idx != idx {
                        self.get(sibling_idx)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![] // 根节点没有兄弟
        }
    }

    /// 获取从根节点到当前节点的索引序列
    pub fn path_to_root(&self, idx: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current_idx = Some(idx);
        while let Some(idx) = current_idx {
            path.push(idx);
            current_idx = self.nodes[idx].parent_index;
        }
        path.reverse();
        path
    }
}

impl Default for GameTree {
    fn default() -> Self {
        // 手动实现 Default，避免 root_index=0 但 nodes 为空的非法状态
        Self::new(NodeProperties::default())
    }
}

/// 棋局数据
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub black_name: String,
    pub white_name: String,
    pub komi: f32,      // 贴目，通常 6.5 或 7.5
    pub handicap: u8,   // 让子数，0 表示无让子
    pub result: String, // e.g., "B+R", "W+1.5", "Draw"
    pub date: String,   // e.g., "2024-10-01"
    pub board_size: u8, // 通常 19
    pub rules: String,  // "Japanese", "Chinese", "AGA"
}

impl GameInfo {
    pub fn new() -> Self {
        Self {
            black_name: String::new(),
            white_name: String::new(),
            komi: 6.5,
            handicap: 0,
            result: String::new(),
            date: String::new(),
            board_size: 19,
            rules: "Japanese".to_string(),
        }
    }
}

impl Default for GameInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// 围棋棋谱记录
#[derive(Debug, Clone)]
pub struct GameRecord {
    pub info: GameInfo,
    pub tree: GameTree,
    /// 当前路径：从根节点到当前位置的索引序列
    pub current_path: Vec<usize>,
}

impl GameRecord {
    pub fn new() -> Self {
        Self {
            info: GameInfo::new(),
            tree: GameTree::default(),
            current_path: vec![0],
        }
    }

    /// 获取当前节点（根据 current_path 的最后一个索引）
    pub fn current_node(&self) -> Option<&TreeNode> {
        self.current_path.last().and_then(|&idx| self.tree.get(idx))
    }

    /// 获取当前节点的可变引用
    pub fn current_node_mut(&mut self) -> Option<&mut TreeNode> {
        self.current_path
            .last()
            .and_then(|&idx| self.tree.get_mut(idx))
    }

    /// 获取当前节点的所有子节点，返回一个迭代器
    pub fn current_children(&self) -> impl Iterator<Item = &TreeNode> {
        self.current_path
            .last()
            .and_then(|&idx| Some(self.tree.children(idx)))
            .into_iter()
            .flatten()
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

    /// 落子（在当前节点添加子节点并前进）
    pub fn play_move(&mut self, mv: Move) -> bool {
        if let Some(current) = self.current_path.last() {
            let props = NodeProperties::default();
            let new_idx = self.tree.add_child(*current, Some(mv), props);
            self.current_path.push(new_idx);
            true
        } else {
            false
        }
    }

    /// 后退到父节点
    pub fn move_to_parent(&mut self) -> bool {
        if self.current_path.len() > 1 {
            self.current_path.pop();
            true
        } else {
            false
        }
    }

    /// 后退到根节点
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

    /// 计算当前节点对应的棋盘状态（从根节点开始逐步应用着法和设置）
    pub fn current_board(&self) -> Board {
        let mut board = Board::new(self.info.board_size);
        // 应用根节点的设置（如 AB/AW）
        if let Some(root) = self.tree.get(self.tree.root_index) {
            for (pt, color) in &root.props.setup {
                board.set(*pt, *color);
            }
        }
        // 从根节点开始，按路径应用着法和设置
        for &idx in &self.current_path {
            if let Some(node) = self.tree.get(idx) {
                // 应用当前节点的设置（如 AB/AW）
                for (pt, color) in &node.props.setup {
                    board.set(*pt, *color);
                }
                // 应用当前节点的着法
                if let Some(mv) = &node.move_data {
                    if let Some(pt) = mv.point {
                        board.set(pt, mv.color);
                    }
                }
            }
        }
        board
    }
}

impl Default for GameRecord {
    fn default() -> Self {
        Self::new()
    }
}
