mod info;
mod node;
mod sgf;
mod tree;
mod zobrist;

pub use crate::game::record::sgf::*;
pub use crate::game::record::{
    info::GameInfo,
    node::{NodeProperties, TreeNode},
    tree::GameTree,
    zobrist::{GameHistory, ZobristHash},
};

use crate::model::{Board, Color, Move};

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
