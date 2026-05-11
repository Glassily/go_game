use crate::board::{Board, IllegalMoveError};
use crate::model::{Color, Move, Point};
use crate::sgf::{GameTree, Property};

pub struct GoRecord {
    pub tree: GameTree,
    pub current: Option<usize>,
    pub board: Board,
    pub black_captures: usize,
    pub white_captures: usize,
    history: Vec<GameTree>,
    future: Vec<GameTree>,
    ko_point: Option<Point>,
}

impl GoRecord {
    pub fn new(size: u8) -> Self {
        Self {
            tree: GameTree::new(),
            current: None,
            board: Board::new(size),
            black_captures: 0,
            white_captures: 0,
            history: Vec::new(),
            future: Vec::new(),
            ko_point: None,
        }
    }

    pub fn board_size(&self) -> u8 {
        self.board.size
    }

    pub fn go_prev(&mut self) {
        if let Some(c) = self.current {
            self.current = self.tree.get_parent(c);
            self.rebuild_board_to(self.current);
        }
    }

    pub fn go_next(&mut self) {
        if let Some(c) = self.current {
            let ch = self.tree.get_children(c);
            if !ch.is_empty() {
                self.current = Some(ch[0]);
                self.rebuild_board_to(self.current);
            }
        } else if let Some(r) = self.tree.get_root() {
            self.current = Some(r);
            self.rebuild_board_to(self.current);
        }
    }

    pub fn go_first(&mut self) {
        self.current = self.tree.get_root();
        self.rebuild_board_to(self.current);
    }

    pub fn go_last(&mut self) {
        let mut cur = self.tree.get_root();
        while let Some(c) = cur {
            let ch = self.tree.get_children(c);
            if ch.is_empty() {
                break;
            }
            cur = Some(ch[0]);
        }
        self.current = cur;
        self.rebuild_board_to(self.current);
    }

    pub fn go_to(&mut self, idx: usize) {
        self.current = Some(idx);
        self.rebuild_board_to(self.current);
    }

    pub fn mainline(&self) -> Vec<usize> {
        let mut res = Vec::new();
        let mut cur = self.tree.get_root();
        while let Some(c) = cur {
            res.push(c);
            let ch = self.tree.get_children(c);
            if ch.is_empty() {
                break;
            }
            cur = Some(ch[0]);
        }
        res
    }

    pub fn rebuild_board_to(&mut self, idx: Option<usize>) {
        let size = self.board.size;
        self.board = Board::new(size);
        self.black_captures = 0;
        self.white_captures = 0;
        self.ko_point = None;

        if idx.is_none() {
            return;
        }

        let mut path = Vec::new();
        let mut cur = idx;
        while let Some(i) = cur {
            path.push(i);
            cur = self.tree.get_parent(i);
        }
        path.reverse();

        for &i in &path {
            if let Some(node) = self.tree.get_node(i) {
                if let Some(v) = node.get(Property::B) {
                    if let Some(s) = v.first() {
                        if let Some(mv) = property_str_to_move(s, Color::Black, size) {
                            let (captured, ko) = self.board.apply_move_uncheck(&mv);
                            self.black_captures += captured.len() as usize;
                            self.ko_point = ko;
                        }
                    }
                }
                if let Some(v) = node.get(Property::W) {
                    if let Some(s) = v.first() {
                        if let Some(mv) = property_str_to_move(s, Color::White, size) {
                            let (captured, ko) = self.board.apply_move_uncheck(&mv);
                            self.white_captures += captured.len() as usize;
                            self.ko_point = ko;
                        }
                    }
                }
            }
        }
    }

    fn push_snapshot(&mut self) {
        self.history.push(self.tree.clone());
        self.future.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.future.push(self.tree.clone());
            self.tree = prev;
            self.current = self.tree.get_root();
            self.rebuild_board_to(self.current);
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.future.pop() {
            self.history.push(self.tree.clone());
            self.tree = next;
            self.current = self.tree.get_root();
            self.rebuild_board_to(self.current);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn add_move(&mut self, mv: Move) -> Result<(), IllegalMoveError> {
        if let Err(e) = self.board.is_legal(&mv, self.ko_point, false) {
            return Err(e);
        }

        let (captured, ko) = self.board.apply_move_uncheck(&mv);
        match mv.color {
            Color::Black => self.black_captures += captured.len() as usize,
            Color::White => self.white_captures += captured.len() as usize,
        }
        self.ko_point = ko;

        self.push_snapshot();
        let mut map = std::collections::HashMap::new();
        let prop = match mv.color {
            Color::Black => Property::B,
            Color::White => Property::W,
        };
        let pt_str = mv.point.map(|p| p.to_sgf()).unwrap_or_default();
        map.insert(prop, vec![pt_str]);
        let _ = self.tree.add_node(self.current, map);
        self.current = Some(self.tree.nodes.len() - 1);
        Ok(())
    }

    pub fn next_to_move(&self) -> Color {
        let mut count = 0usize;
        if let Some(mut n) = self.current {
            while let Some(p) = self.tree.get_parent(n) {
                if let Some(node) = self.tree.get_node(n) {
                    if node.contains(Property::B) || node.contains(Property::W) {
                        count += 1;
                    }
                }
                n = p;
            }
            if let Some(root) = self.tree.get_root() {
                if let Some(node) = self.tree.get_node(root) {
                    if node.contains(Property::B) || node.contains(Property::W) {
                        count += 1;
                    }
                }
            }
        }
        if count % 2 == 0 {
            Color::Black
        } else {
            Color::White
        }
    }

    pub fn node_depth(&self, idx: usize) -> usize {
        let mut d = 0;
        let mut cur = Some(idx);
        while let Some(i) = cur {
            cur = self.tree.get_parent(i);
            if cur.is_some() {
                d += 1;
            }
        }
        d
    }

    pub fn current_move_number(&self) -> usize {
        let mut cnt = 0usize;
        if let Some(c) = self.current {
            let mut n = Some(c);
            while let Some(i) = n {
                if let Some(node) = self.tree.get_node(i) {
                    if node.contains(Property::B) || node.contains(Property::W) {
                        cnt += 1;
                    }
                }
                n = self.tree.get_parent(i);
            }
        }
        cnt
    }

    pub fn total_moves(&self) -> usize {
        self.mainline().len()
    }

    pub fn node_count(&self) -> usize {
        self.tree.nodes.iter().filter(|n| !n.deleted).count()
    }

    pub fn get_comment(&self, idx: usize) -> Option<String> {
        self.tree
            .get_node(idx)
            .and_then(|n| n.get(Property::C).and_then(|v| v.first().cloned()))
    }

    pub fn set_comment(&mut self, idx: usize, comment: String) {
        self.push_snapshot();
        if let Some(node) = self.tree.get_node_mut(idx) {
            if comment.is_empty() {
                node.data.remove(&Property::C);
            } else {
                node.set(Property::C, vec![comment]);
            }
        }
    }

    pub fn get_game_info(&self) -> GameInfo {
        let mut info = GameInfo::default();
        if let Some(root) = self.tree.get_root() {
            if let Some(node) = self.tree.get_node(root) {
                info.black = node.get(Property::PB).and_then(|v| v.first().cloned());
                info.white = node.get(Property::PW).and_then(|v| v.first().cloned());
                info.date = node.get(Property::DT).and_then(|v| v.first().cloned());
                info.komi = node.get(Property::KM).and_then(|v| v.first().cloned());
                info.result = node.get(Property::RE).and_then(|v| v.first().cloned());
            }
        }
        info
    }

    pub fn set_game_info(&mut self, info: &GameInfo) {
        self.push_snapshot();
        if let Some(root) = self.tree.get_root() {
            if let Some(node) = self.tree.get_node_mut(root) {
                if let Some(ref v) = info.black {
                    node.set(Property::PB, vec![v.clone()]);
                }
                if let Some(ref v) = info.white {
                    node.set(Property::PW, vec![v.clone()]);
                }
                if let Some(ref v) = info.date {
                    node.set(Property::DT, vec![v.clone()]);
                }
                if let Some(ref v) = info.komi {
                    node.set(Property::KM, vec![v.clone()]);
                }
                if let Some(ref v) = info.result {
                    node.set(Property::RE, vec![v.clone()]);
                }
            }
        }
    }

    pub fn new_game(&mut self) {
        self.push_snapshot();
        self.tree = GameTree::new();
        self.current = None;
        self.rebuild_board_to(None);
    }

    pub fn load_sgf(&mut self, tree: GameTree) {
        self.push_snapshot();
        self.tree = tree;
        self.current = self.tree.get_root();
        self.rebuild_board_to(self.current);
    }

    pub fn find_move_at_point(&self, pt: Point) -> Option<usize> {
        let sgf_str = pt.to_sgf();
        for (i, node) in self.tree.nodes.iter().enumerate().rev() {
            if node.deleted {
                continue;
            }
            if let Some(v) = node.get(Property::B) {
                if v.first().map(|s| s == &sgf_str).unwrap_or(false) {
                    return Some(i);
                }
            }
            if let Some(v) = node.get(Property::W) {
                if v.first().map(|s| s == &sgf_str).unwrap_or(false) {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn get_node_info(&self, idx: usize) -> Option<NodeInfo> {
        self.tree.get_node(idx).map(|node| {
            let mut kind = 0u8;
            if node.contains(Property::B) {
                kind = 1;
            }
            if node.contains(Property::W) {
                kind = 2;
            }
            let comment = node.get(Property::C).and_then(|v| v.first().cloned());
            NodeInfo {
                kind,
                comment,
                depth: self.node_depth(idx),
            }
        })
    }

    pub fn all_nodes(&self) -> Vec<(usize, NodeInfo)> {
        self.tree
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                if node.deleted {
                    return None;
                }
                self.get_node_info(i).map(|info| (i, info))
            })
            .collect()
    }
}

fn property_str_to_move(s: &str, color: Color, board_size: u8) -> Option<Move> {
    if s.is_empty() {
        Some(Move::pass(color))
    } else {
        Point::from_sgf(s, board_size).map(|pt| Move::new(color, pt))
    }
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub kind: u8,
    pub comment: Option<String>,
    pub depth: usize,
}

#[derive(Debug, Clone, Default)]
pub struct GameInfo {
    pub black: Option<String>,
    pub white: Option<String>,
    pub date: Option<String>,
    pub komi: Option<String>,
    pub result: Option<String>,
}
