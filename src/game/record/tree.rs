use crate::{game::record::node::{NodeProperties, TreeNode}, model::Move};

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


