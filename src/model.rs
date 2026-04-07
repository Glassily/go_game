use std::{collections::HashMap, fmt::Display};

/// 棋盘坐标 (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl Point {
    /// SGF 坐标转换 (a-t 跳过 i)
    pub fn from_sgf(c: char) -> Option<u8> {
        match c {
            //0..=7
            'a'..='h' => Some(c as u8 - b'a'),
            //8..=18
            'j'..='z' => Some(c as u8 - b'j' + 8), //跳过i
            _ => None,
        }
    }

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

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Black => write!(f, "B"),
            Color::White => write!(f, "W"),
        }
    }
}

/// 一手棋 (None 表示 Pass)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub color: Color,
    pub point: Option<Point>,
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
    /// 保留原始 SGF 属性防丢失
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
#[derive(Debug, Clone, Default)]
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
        move_data: Move,
        props: NodeProperties,
    ) -> usize {
        //创建子节点索引，按加入nodes的顺序创建
        let new_idx = self.nodes.len();
        let child = TreeNode {
            parent_index: Some(parent_idx),
            children: vec![],
            move_data: Some(move_data),
            props,
        };
        //将子节点加入nodes
        self.nodes.push(child);
        //在父节点添加子节点索引
        self.nodes[parent_idx].children.push(new_idx);
        new_idx
    }
}

/// 棋局数据
#[derive(Debug, Clone, Default)]
pub struct GameInfo{
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
    pub fn current_node(&self) -> Option<&TreeNode> {
        self.current_path.last().and_then(|&idx| self.tree.get(idx))
    }

    pub fn current_node_mut(&mut self) -> Option<&mut TreeNode> {
        self.current_path.last().and_then(|&idx| self.tree.get_mut(idx))
    }

    /// 前进到指定子节点（切换变招）
    pub fn move_to_child(&mut self, child_idx: usize) -> bool {
        if let Some(current) = self.current_path.last() {
            let node = &self.tree.nodes[*current];
            if node.children.contains(&child_idx) {
                self.current_path.push(child_idx);
                return true;
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
}

