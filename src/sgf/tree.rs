use crate::sgf::property::Property;
use std::collections::{HashMap, VecDeque};

/// 游戏树节点结构
///
/// 包含节点的属性数据以及在树中的位置信息
#[derive(Debug, Clone)]
pub struct Node {
    /// 节点属性数据
    pub data: HashMap<Property, Vec<String>>,
    /// 父节点索引
    pub parent_index: Option<usize>,
    /// 子节点索引列表
    pub children: Vec<usize>,
    /// 是否已删除
    pub deleted: bool,
}

impl Node {
    /// 创建新节点
    pub fn new(data: HashMap<Property, Vec<String>>) -> Self {
        Self {
            data,
            parent_index: None,
            children: Vec::new(),
            deleted: false,
        }
    }

    /// 获取指定属性的所有值
    pub fn get(&self, prop: Property) -> Option<&Vec<String>> {
        self.data.get(&prop)
    }

    /// 获取指定属性的第一个值
    pub fn get_first(&self, prop: Property) -> Option<&String> {
        self.get(prop)?.first()
    }

    /// 设置属性值
    pub fn set(&mut self, prop: Property, values: Vec<String>) {
        self.data.insert(prop, values);
    }

    /// 添加属性值
    pub fn add_value(&mut self, prop: Property, value: String) {
        self.data.entry(prop).or_default().push(value);
    }

    /// 检查是否包含指定属性
    pub fn contains(&self, prop: Property) -> bool {
        self.data.contains_key(&prop)
    }
}

/// 游戏树错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeError {
    /// 无效的索引
    InvalidIndex(usize),
    /// 父节点不存在
    ParentNotFound(usize),
    /// 子节点不存在
    ChildNotFound(usize, usize),
    /// 根节点已存在
    RootAlreadyExists,
    /// 非法修改根节点
    InvalidRootChange,
}

impl std::fmt::Display for TreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TreeError::InvalidIndex(i) => write!(f, "Invalid index: {}", i),
            TreeError::ParentNotFound(p) => write!(f, "Parent not found: {}", p),
            TreeError::ChildNotFound(p, c) => write!(f, "Child {} not under {}", c, p),
            TreeError::RootAlreadyExists => write!(f, "Root already exists"),
            TreeError::InvalidRootChange => write!(f, "Cannot set non-root as root"),
        }
    }
}
impl std::error::Error for TreeError {}

/// 游戏树结构
///
/// 表示 SGF 文件中的完整游戏树，包含所有节点
#[derive(Debug, Clone)]
pub struct GameTree {
    /// 所有节点
    pub nodes: Vec<Node>,
    /// 根节点索引
    pub root_index: Option<usize>,
}

impl GameTree {
    /// 创建新的空游戏树
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_index: None,
        }
    }

    /// 添加新节点
    ///
    /// # 参数
    /// - `parent`: 父节点索引，None 表示作为根节点
    /// - `data`: 节点属性数据
    pub fn add_node(
        &mut self,
        parent: Option<usize>,
        data: HashMap<Property, Vec<String>>,
    ) -> Result<usize, TreeError> {
        if let Some(p) = parent {
            if p >= self.nodes.len() {
                return Err(TreeError::ParentNotFound(p));
            }
        }
        let idx = self.nodes.len();
        self.nodes.push(Node {
            data,
            parent_index: parent,
            children: Vec::new(),
            deleted: false,
        });
        if let Some(p) = parent {
            self.nodes[p].children.push(idx);
        } else if self.root_index.is_none() {
            self.root_index = Some(idx);
        }
        Ok(idx)
    }

    /// 获取指定索引的节点（不可变引用）
    pub fn get_node(&self, idx: usize) -> Option<&Node> {
        self.nodes.get(idx)
    }

    /// 获取指定索引的节点（可变引用）
    pub fn get_node_mut(&mut self, idx: usize) -> Option<&mut Node> {
        self.nodes.get_mut(idx)
    }

    /// 获取父节点索引
    pub fn get_parent(&self, idx: usize) -> Option<usize> {
        self.nodes.get(idx)?.parent_index
    }

    /// 获取子节点索引列表
    pub fn get_children(&self, idx: usize) -> &[usize] {
        self.nodes
            .get(idx)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    /// 获取根节点索引
    pub fn get_root(&self) -> Option<usize> {
        self.root_index
    }

    /// 删除子树
    ///
    /// 将指定节点及其所有后代标记为已删除，并断开与父节点的连接
    pub fn remove_subtree(&mut self, idx: usize) -> Result<(), TreeError> {
        if idx >= self.nodes.len() {
            return Err(TreeError::InvalidIndex(idx));
        }
        if let Some(p) = self.nodes[idx].parent_index {
            if p >= self.nodes.len() {
                return Err(TreeError::ParentNotFound(p));
            }
            self.nodes[p].children.retain(|&c| c != idx);
        } else {
            if Some(idx) == self.root_index {
                self.root_index = None;
            }
        }

        let mut stack = vec![idx];
        while let Some(i) = stack.pop() {
            if i >= self.nodes.len() {
                return Err(TreeError::InvalidIndex(i));
            }
            self.nodes[i].deleted = true;
            for &c in &self.nodes[i].children {
                stack.push(c);
            }
            self.nodes[i].children.clear();
            self.nodes[i].parent_index = None;
        }
        Ok(())
    }

    /// 返回前序遍历迭代器
    ///
    /// 前序遍历：先访问节点，再访问子节点
    pub fn preorder_iter(&self) -> PreorderIter<'_> {
        PreorderIter {
            tree: self,
            stack: self.root_index.map(|r| vec![r]).unwrap_or_default(),
        }
    }

    /// 返回广度优先遍历迭代器
    ///
    /// 广度优先遍历：按层级访问节点
    pub fn bfs_iter(&self) -> BfsIter<'_> {
        BfsIter {
            tree: self,
            queue: self
                .root_index
                .map(|r| {
                    let mut q = VecDeque::new();
                    q.push_back(r);
                    q
                })
                .unwrap_or_default(),
        }
    }
}

impl Default for GameTree {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<Property, Vec<String>>> for GameTree {
    fn from(root_data: HashMap<Property, Vec<String>>) -> Self {
        let mut t = Self::new();
        let idx = t.nodes.len();
        t.nodes.push(Node::new(root_data));
        t.root_index = Some(idx);
        t
    }
}

/// 前序遍历迭代器
pub struct PreorderIter<'a> {
    tree: &'a GameTree,
    stack: Vec<usize>,
}

impl<'a> Iterator for PreorderIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.stack.pop()?;
        for &c in self.tree.get_children(idx).iter().rev() {
            self.stack.push(c);
        }
        Some(idx)
    }
}

/// 广度优先遍历迭代器
pub struct BfsIter<'a> {
    tree: &'a GameTree,
    queue: VecDeque<usize>,
}

impl<'a> Iterator for BfsIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.queue.pop_front()?;
        for &c in self.tree.get_children(idx) {
            self.queue.push_back(c);
        }
        Some(idx)
    }
}
