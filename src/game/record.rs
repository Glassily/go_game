use std::{collections::HashMap, usize};

use crate::model::{Color, Move, Point};

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
    pub raw_sgf_props: Vec<(String, Vec<String>)>,
}

/// 树节点
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

    /// 获取节点引用
    pub fn node(&self, idx: usize) -> Option<&TreeNode> {
        self.nodes.get(idx)
    }

    /// 获取节点可变引用
    pub fn node_mut(&mut self, idx: usize) -> Option<&mut TreeNode> {
        self.nodes.get_mut(idx)
    }

    /// 添加子节点（变招）
    pub fn add_child_node(
        &mut self,
        parent_idx: usize,
        move_data: Option<Move>,
        props: NodeProperties,
    ) -> Option<usize> {
        if parent_idx >= self.nodes.len() {
            return None;
        }
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
        Some(new_idx)
    }

    /// 获取指定节点的子节点迭代器（同时给出索引）
    pub fn get_children(&self, idx: usize) -> impl Iterator<Item = (usize, &TreeNode)> + '_ {
        self.nodes[idx]
            .children
            .iter()
            .filter_map(|&child_idx| self.node(child_idx).map(|node| (child_idx, node)))
    }

    /// 获取指定节点的父节点
    pub fn get_parent(&self, idx: usize) -> Option<&TreeNode> {
        self.nodes[idx]
            .parent_index
            .and_then(|parent_idx| self.node(parent_idx))
    }

    /// 获取指定节点的兄弟节点（同父节点的其他子节点）
    pub fn get_siblings(&self, idx: usize) -> Vec<&TreeNode> {
        if let Some(parent_idx) = self.nodes[idx].parent_index {
            self.nodes[parent_idx]
                .children
                .iter()
                .filter_map(|&sibling_idx| {
                    if sibling_idx != idx {
                        self.node(sibling_idx)
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

    /// 获取指定节点的棋盘状态
    pub fn board_state(&self, idx: usize) -> Vec<(Point, Color)> {
        let mut board = Vec::new();
        for node_idx in self.path_to_root(idx) {
            if let Some(node) = self.node(node_idx) {
                if let Some(Move { point, color }) = &node.move_data {
                    if let Some(p) = point {
                        board.push((*p, *color));
                    }
                }
                for (pt, color) in &node.props.setup {
                    board.push((*pt, *color));
                }
            }
        }
        board
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
        self.current_path
            .last()
            .and_then(|&idx| self.tree.node(idx))
    }

    /// 获取当前节点的可变引用
    pub fn current_node_mut(&mut self) -> Option<&mut TreeNode> {
        self.current_path
            .last()
            .and_then(|&idx| self.tree.node_mut(idx))
    }

    /// 获取当前棋盘状态
    pub fn current_board_state(&self) -> Vec<(Point, Color)> {
        if let Some(current) = self.current_path.last() {
            self.tree.board_state(*current)
        } else {
            vec![]
        }
    }

    /// 获取当前节点的所有子节点，返回一个迭代器（同时给出索引）
    pub fn current_children(&self) -> impl Iterator<Item = (usize, &TreeNode)> {
        self.current_path
            .last()
            .map(|&idx| self.tree.get_children(idx))
            .into_iter()
            .flatten()
    }

    /// 前进到指定子节点
    pub fn move_to_child(&mut self, child_idx: usize) -> bool {
        if let Some(current) = self.current_path.last() {
            if let Some(node) = self.tree.node(*current) {
                if node.children.contains(&child_idx) {
                    self.current_path.push(child_idx);

                    return true;
                }
            }
        }
        false
    }

    /// 落子，返回新节点索引或 `None`
    pub fn play_move(&mut self, mv: Move) -> Option<usize> {
        if let Some(current) = self.current_path.last() {
            let props = NodeProperties::default();
            let new_idx = self.tree.add_child_node(*current, Some(mv), props)?;
            self.current_path.push(new_idx);
            Some(new_idx)
        } else {
            None
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
            if let Some(node) = self.tree.node(idx) {
                if let Some(mv) = &node.move_data {
                    turn = mv.color.opposite();
                }
            }
        }
        Some(turn)
    }
}

impl Default for GameRecord {
    fn default() -> Self {
        Self::new()
    }
}
