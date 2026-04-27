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

    /// 删除以idx为根的整棵子树，并更新全部索引。
    /// 返回旧索引到新索引的映射。若idx无效或是根节点则返回None。
    pub fn remove_subtree(&mut self, idx: usize) -> Option<HashMap<usize, usize>> {
        // 不允许删除根节点
        if idx >= self.nodes.len() || idx == self.root_index {
            return None;
        }

        // 1. 收集要删除的索引（BFS）
        let mut to_delete = vec![idx];
        let mut i = 0;
        while i < to_delete.len() {
            let node_idx = to_delete[i];
            if let Some(node) = self.nodes.get(node_idx) {
                to_delete.extend(node.children.iter().copied());
            }
            i += 1;
        }
        // 转为 HashSet 方便查找
        let del_set: std::collections::HashSet<usize> = to_delete.into_iter().collect();

        // 2. 从父节点移除该子节点索引
        if let Some(parent_idx) = self.nodes[idx].parent_index {
            if let Some(parent) = self.nodes.get_mut(parent_idx) {
                parent.children.retain(|&c| c != idx);
            }
        }

        // 3. 构建新旧映射并压缩 nodes
        let mut old_to_new = vec![None; self.nodes.len()];
        let mut new_nodes = Vec::with_capacity(self.nodes.len() - del_set.len());

        for (old_idx, node) in self.nodes.iter().enumerate() {
            if !del_set.contains(&old_idx) {
                let new_idx = new_nodes.len();
                old_to_new[old_idx] = Some(new_idx);
                new_nodes.push(node.clone());
            }
        }

        // 4. 更新保留节点中的索引
        for node in &mut new_nodes {
            if let Some(pidx) = node.parent_index {
                node.parent_index = old_to_new[pidx]; // 若父节点被删则变 None，但父不可能被删（因只删子树）
            }
            node.children = node
                .children
                .iter()
                .filter_map(|&c| old_to_new[c])
                .collect();
        }

        // 5. 更新根索引（根未被删除，必然存在）
        self.root_index =
            old_to_new[self.root_index].expect("root was deleted, but deletion is forbidden");

        self.nodes = new_nodes;

        // 6. 构造并返回映射
        let map: HashMap<usize, usize> = old_to_new
            .into_iter()
            .enumerate()
            .filter_map(|(old, new)| new.map(|n| (old, n)))
            .collect();
        Some(map)
    }

    /// 获取子节点迭代器，idx无效返回None
    pub fn get_children(&self, idx: usize) -> Option<impl Iterator<Item = (usize, &TreeNode)>> {
        let node = self.node(idx)?;
        Some(
            node.children
                .iter()
                .filter_map(move |&cidx| self.node(cidx).map(|n| (cidx, n)))
        )
    }

    /// 获取父节点引用，idx效返回None
    pub fn get_parent(&self, idx: usize) -> Option<&TreeNode> {
        let parent_idx = self.node(idx)?.parent_index?;
        self.node(parent_idx)
    }

    /// 获取兄弟节点迭代器，idx无效返回None
    pub fn get_siblings(&self, idx: usize) -> Option<impl Iterator<Item = (usize, &TreeNode)>> {
        let parent_idx = self.node(idx)?.parent_index?;
        Some(
            self.nodes[parent_idx]
                .children
                .iter()
                .filter(move |&&sib| sib != idx)
                .filter_map(|&sib| self.node(sib).map(|n| (sib, n)))
        )
    }

    /// 获取从根节点到指定节点的索引序列
    pub fn path_to_root(&self, idx: usize) -> Option<Vec<usize>> {
        let mut path = Vec::new();
        let mut current_idx = Some(idx);
        while let Some(idx) = current_idx {
            path.push(idx);
            current_idx = self.node(idx)?.parent_index;
        }
        path.reverse();
        Some(path)
    }

    /// 从根开始深度优先遍历
    pub fn iter_depth_first(&self) -> DepthFirstIter<'_> {
        DepthFirstIter::new(self, self.root_index)
    }
}

impl Default for GameTree {
    fn default() -> Self {
        // 手动实现 Default，避免 root_index=0 但 nodes 为空的非法状态
        Self::new(NodeProperties::default())
    }
}

pub struct DepthFirstIter<'a> {
    tree: &'a GameTree,
    stack: Vec<usize>,
}

impl<'a> DepthFirstIter<'a> {
    fn new(tree: &'a GameTree, start: usize) -> Self {
        let mut stack = Vec::new();
        if tree.node(start).is_some() {
            stack.push(start);
        }
        Self { tree, stack }
    }
}

impl<'a> Iterator for DepthFirstIter<'a> {
    type Item = (usize, &'a TreeNode);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(idx) = self.stack.pop() {
            if let Some(node) = self.tree.node(idx) {
                // 逆序压入未删除子节点，保证先子后兄弟
                let children: Vec<usize> = node.children.iter()
                    .copied()
                    .filter(|&c| self.tree.node(c).is_some())
                    .collect();
                for &child in children.iter().rev() {
                    self.stack.push(child);
                }
                return Some((idx, node));
            }
        }
        None
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
    pub starting_player: Option<Color>,
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
            starting_player: None,
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
        if let Some(&cur) = self.current_path.last() {
            if let Some(map) = self.tree.remove_subtree(cur) {
                // 将 current_path 中已删除的节点移除，并将保留的索引映射到新索引
                self.current_path.retain(|idx| map.contains_key(idx));
                for idx in self.current_path.iter_mut() {
                    *idx = *map.get(idx).unwrap_or(idx);
                }
                return true;
            }
        }
        false
    }

    /// 获取当前棋盘状态
    pub fn current_board(&self) -> Board {
        let mut setup = Vec::new();
        for node_idx in self
            .tree
            .path_to_root(*self.current_path.last().unwrap_or(&0))
            .unwrap()
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
    pub fn current_turn(&self) -> Color {
        // 优先使用设定
        if let Some(sp) = self.info.starting_player {
            return sp;
        }
        
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
        turn
    }

    /// 获取当前节点的所有子节点，返回一个迭代器（同时给出索引）
    pub fn current_children(&self) -> impl Iterator<Item = (usize, &TreeNode)> {
        self.current_path
            .last()
            .map(|&idx| self.tree.get_children(idx).unwrap())
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
            .map(|&idx| self.tree.get_siblings(idx).unwrap())
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
            .and_then(|&current_idx| self.tree.get_children(current_idx).unwrap().next())
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
