use crate::sgf::property::Property;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct Node {
    pub data: HashMap<Property, Vec<String>>,
    pub parent_index: Option<usize>,
    pub children: Vec<usize>,
    pub deleted: bool,
}

impl Node {
    pub fn new(data: HashMap<Property, Vec<String>>) -> Self {
        Self {
            data,
            parent_index: None,
            children: Vec::new(),
            deleted: false,
        }
    }

    pub fn get(&self, prop: Property) -> Option<&Vec<String>> {
        self.data.get(&prop)
    }
    pub fn get_first(&self, prop: Property) -> Option<&String> {
        self.get(prop)?.first()
    }
    pub fn set(&mut self, prop: Property, values: Vec<String>) {
        self.data.insert(prop, values);
    }
    pub fn add_value(&mut self, prop: Property, value: String) {
        self.data.entry(prop).or_default().push(value);
    }
    pub fn contains(&self, prop: Property) -> bool {
        self.data.contains_key(&prop)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeError {
    InvalidIndex(usize),
    ParentNotFound(usize),
    ChildNotFound(usize, usize),
    RootAlreadyExists,
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

#[derive(Debug, Clone)]
pub struct GameTree {
    pub nodes: Vec<Node>,
    pub root_index: Option<usize>,
}

impl GameTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_index: None,
        }
    }

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

    pub fn get_node(&self, idx: usize) -> Option<&Node> {
        self.nodes.get(idx)
    }
    pub fn get_node_mut(&mut self, idx: usize) -> Option<&mut Node> {
        self.nodes.get_mut(idx)
    }
    pub fn get_parent(&self, idx: usize) -> Option<usize> {
        self.nodes.get(idx)?.parent_index
    }
    pub fn get_children(&self, idx: usize) -> &[usize] {
        self.nodes
            .get(idx)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }
    pub fn get_root(&self) -> Option<usize> {
        self.root_index
    }

    /// Mark a node and all its descendants as deleted and remove the link from its parent.
    pub fn remove_subtree(&mut self, idx: usize) -> Result<(), TreeError> {
        if idx >= self.nodes.len() {
            return Err(TreeError::InvalidIndex(idx));
        }
        // detach from parent
        if let Some(p) = self.nodes[idx].parent_index {
            if p >= self.nodes.len() {
                return Err(TreeError::ParentNotFound(p));
            }
            self.nodes[p].children.retain(|&c| c != idx);
        } else {
            // removing root
            if Some(idx) == self.root_index {
                self.root_index = None;
            }
        }

        // mark subtree deleted (DFS)
        let mut stack = vec![idx];
        while let Some(i) = stack.pop() {
            if i >= self.nodes.len() {
                return Err(TreeError::InvalidIndex(i));
            }
            self.nodes[i].deleted = true;
            for &c in &self.nodes[i].children {
                stack.push(c);
            }
            // clear children to avoid future traversal from this node
            self.nodes[i].children.clear();
            self.nodes[i].parent_index = None;
        }
        Ok(())
    }

    pub fn preorder_iter(&self) -> PreorderIter<'_> {
        PreorderIter {
            tree: self,
            stack: self.root_index.map(|r| vec![r]).unwrap_or_default(),
        }
    }

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
