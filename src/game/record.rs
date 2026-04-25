use std::{collections::HashMap, usize};

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

    /// 添加子节点
    pub fn add_child_node(
        &mut self,
        parent_idx: usize,
        move_data: Option<Move>,
        props: NodeProperties,
    ) -> Option<usize> {
        if parent_idx >= self.nodes.len() {
            return None;
        }
        //创建子节点索引
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
    pub fn get_children(&self, idx: usize) -> impl Iterator<Item = (usize, &TreeNode)> {
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

    /// 获取指定节点的兄弟节点（同时给出索引）
    pub fn get_siblings(&self, idx: usize) -> impl Iterator<Item = (usize, &TreeNode)> {
        let parent_idx_opt = self.nodes.get(idx).and_then(|node| node.parent_index);
        parent_idx_opt.into_iter().flat_map(move |parent_idx| {
            self.nodes[parent_idx]
                .children
                .iter()
                .filter_map(move |&sibling_idx| {
                    if sibling_idx != idx {
                        self.node(sibling_idx).map(|node| (sibling_idx, node))
                    } else {
                        None
                    }
                })
        })
    }

    /// 获取从根节点到指定节点的索引序列
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

    /// 获取当前节点的引用
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

    /// 移除当前节点及子树
    pub fn remove_current_node(&mut self) -> bool {
        todo!()
    }

    /// 获取当前棋盘状态
    pub fn current_board(&self) -> Board {
        let mut setup = Vec::new();
        for node_idx in self
            .tree
            .path_to_root(*self.current_path.last().unwrap_or(&0))
        {
            if let Some(node) = self.tree.node(node_idx) {
                if let Some(Move { point, color }) = &node.move_data {
                    if let Some(p) = point {
                        setup.push((*p, *color));
                    }
                }
                for (pt, color) in &node.props.setup {
                    setup.push((*pt, *color));
                }
            }
        }
        //setup
        Board::from_setup(self.info.board_size, setup)
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

    /// 获取当前节点的所有子节点，返回一个迭代器（同时给出索引）
    pub fn current_children(&self) -> impl Iterator<Item = (usize, &TreeNode)> {
        self.current_path
            .last()
            .map(|&idx| self.tree.get_children(idx))
            .into_iter()
            .flatten()
    }

    /// 获取当前节点的父节点
    pub fn current_parent_node(&self) -> Option<&TreeNode> {
        if let Some(idx) = self.current_path.last() {
            self.tree.get_parent(*idx)
        } else {
            None
        }
    }

    /// 获得当前节点的所有兄弟节点
    pub fn current_siblings(&self) -> impl Iterator<Item = (usize, &TreeNode)> {
        self.current_path
            .last()
            .map(|&idx| self.tree.get_siblings(idx))
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

    /// 前进到第一个子节点
    pub fn move_to_first_child(&mut self) -> bool {
        let first_child_idx = self
            .current_path
            .last()
            .and_then(|&current_idx| self.tree.get_children(current_idx).next())
            .map(|(idx, _)| idx);

        if let Some(child_idx) = first_child_idx {
            self.move_to_child(child_idx)
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

    /// 添加评论
    pub fn add_comment(&mut self, comment: String) -> bool {
        if let Some(node) = self.current_node_mut() {
            node.props.comment = comment;
            true
        } else {
            false
        }
    }

    /// 添加标注
    pub fn add_label(&mut self, point: Point, label: String) -> bool {
        if let Some(node) = self.current_node_mut() {
            node.props.labels.insert(point, label);
            true
        } else {
            false
        }
    }
}

impl Default for GameRecord {
    fn default() -> Self {
        Self::new()
    }
}
