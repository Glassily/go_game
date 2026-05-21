use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chardetng::EncodingDetector;
use chardetng::Iso2022JpDetection;

use crate::Node;
use crate::board::{Board, IllegalMoveError};
use crate::model::{Color, Move, Point};
use crate::sgf::{GameTree, ParseError, Property, export, parse};

/// 文件加载错误类型
#[derive(Debug)]
pub enum FileError {
    Io(std::io::Error),
    Parse(ParseError),
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::Io(err)
    }
}

impl From<ParseError> for FileError {
    fn from(err: ParseError) -> Self {
        FileError::Parse(err)
    }
}

/// 围棋对局记录结构
///
/// 包含完整的游戏树、当前节点位置、棋盘状态、提子计数等信息
pub struct GoRecord {
    /// 游戏树（包含所有节点和着法）
    pub tree: GameTree,
    /// 当前节点索引
    current_idx: Option<usize>,
    /// 棋盘状态
    pub board: Board,
    /// 黑方提子数（白方被吃的子数）
    pub black_captures: usize,
    /// 白方提子数（黑方被吃的子数）
    pub white_captures: usize,
    /// 历史版本栈（用于撤销），保存 (树, 当前节点)
    history: Vec<(GameTree, Option<usize>)>,
    /// 未来版本栈（用于重做）
    future: Vec<(GameTree, Option<usize>)>,
    /// 当前劫点位置
    ko_point: Option<Point>,
    /// 让子数（0 表示分先，2+ 表示让子）
    handicap: u8,
}

impl Default for GoRecord {
    fn default() -> Self {
        Self::new(19)
    }
}

impl GoRecord {
    /// 创建指定大小的空对局记录
    pub fn new(size: u8) -> Self {
        let mut root_data = std::collections::HashMap::new();
        root_data.insert(Property::GM, vec!["1".to_string()]);
        root_data.insert(Property::FF, vec!["4".to_string()]);
        root_data.insert(Property::SZ, vec![size.to_string()]);
        root_data.insert(Property::RU, vec!["Japanese".to_string()]);
        root_data.insert(Property::KM, vec!["6.5".to_string()]);
        let tree = GameTree::from(root_data);

        Self {
            tree,
            current_idx: None,
            board: Board::new(size),
            black_captures: 0,
            white_captures: 0,
            history: Vec::new(),
            future: Vec::new(),
            ko_point: None,
            handicap: 0,
        }
    }

    /// 设置让子数
    pub fn handicap(mut self, handicap: u8) -> Self {
        self.set_handicap(handicap);
        self
    }

    /// 设置让子数（内部方法）
    fn set_handicap(&mut self, handicap: u8) {
        self.handicap = handicap;
        let size = self.board.size;
        if handicap > 0 {
            self.board = Self::setup_board(size, handicap);
            self.set_root_property(Property::HA, vec![handicap.to_string()]);
            self.set_root_property(Property::KM, vec!["0.5".to_string()]);
        } else {
            self.board = Board::new(size);
            if let Some(root) = self.tree.get_root() {
                if let Some(node) = self.tree.get_node_mut(root) {
                    node.data.remove(&Property::HA);
                }
            }
        }
        if self.current_idx.is_some() {
            self.rebuild_board_to(self.current_idx);
        }
    }

    /// 根据让子数设置棋盘初始局面
    fn setup_board(size: u8, handicap: u8) -> Board {
        if handicap <= 1 {
            return Board::new(size);
        }
        let mut board = Board::new(size);
        let star_points = Self::get_handicap_points(handicap, size);
        for pt in star_points {
            if pt.is_valid(size) {
                board.set(pt, Color::Black);
            }
        }
        board
    }

    /// 获取让子点坐标（星位）
    fn get_handicap_points(handicap: u8, size: u8) -> Vec<Point> {
        let offset = if size > 9 { 3 } else { 2 };
        match handicap {
            2 => vec![
                Point {
                    x: size - 1 - offset,
                    y: offset,
                },
                Point {
                    x: offset,
                    y: size - 1 - offset,
                },
            ],
            3 => vec![
                Point {
                    x: size - 1 - offset,
                    y: offset,
                },
                Point {
                    x: offset,
                    y: size - 1 - offset,
                },
                Point {
                    x: offset,
                    y: offset,
                },
            ],
            4 => vec![
                Point {
                    x: size - 1 - offset,
                    y: offset,
                },
                Point {
                    x: offset,
                    y: size - 1 - offset,
                },
                Point {
                    x: offset,
                    y: offset,
                },
                Point {
                    x: size - 1 - offset,
                    y: size - 1 - offset,
                },
            ],
            5 => vec![
                Point {
                    x: size - 1 - offset,
                    y: offset,
                },
                Point {
                    x: offset,
                    y: size - 1 - offset,
                },
                Point {
                    x: offset,
                    y: offset,
                },
                Point {
                    x: size - 1 - offset,
                    y: size - 1 - offset,
                },
                Point {
                    x: size / 2,
                    y: size / 2,
                },
            ],
            _ => Vec::new(),
        }
    }

    /// 获取让子数
    pub fn get_handicap(&self) -> u8 {
        self.handicap
    }

    /// 返回当前节点索引
    pub fn current_index(&self) -> Option<usize> {
        self.current_idx
    }

    /// 删除子树
    ///
    /// 删除指定节点及其所有后代。如果当前节点在删除的子树中，
    /// 则将当前位置移动到被删除节点的父节点。
    /// 注意：不能删除根节点。
    pub fn delete_subtree(&mut self, idx: usize) -> Result<(), crate::sgf::TreeError> {
        if Some(idx) == self.tree.get_root() {
            return Err(crate::sgf::TreeError::InvalidRootChange);
        }
        let parent = self.tree.get_parent(idx);
        self.tree.remove_subtree(idx)?;
        if let Some(p) = parent {
            self.current_idx = Some(p);
            self.rebuild_board_to(self.current_idx);
        } else {
            self.current_idx = self.tree.get_root();
            self.rebuild_board_to(self.current_idx);
        }
        Ok(())
    }

    /// 检查指定节点是否为根节点
    pub fn is_root(&self, idx: usize) -> bool {
        Some(idx) == self.tree.get_root()
    }

    /// 设置根节点属性
    pub fn set_root_property(&mut self, prop: Property, values: Vec<String>) {
        if let Some(root) = self.tree.get_root() {
            if let Some(node) = self.tree.get_node_mut(root) {
                node.set(prop, values);
            }
        }
    }

    /// 获取棋盘大小
    pub fn board_size(&self) -> u8 {
        self.board.size
    }

    /// 移动到上一个节点
    pub fn go_prev(&mut self) {
        if let Some(c) = self.current_idx {
            self.current_idx = self.tree.get_parent(c);
            self.rebuild_board_to(self.current_idx);
        }
    }

    /// 移动到下一个节点（主变体）
    pub fn go_next(&mut self) {
        if let Some(c) = self.current_idx {
            let ch = self.tree.get_children(c);
            if !ch.is_empty() {
                self.current_idx = Some(ch[0]);
                self.rebuild_board_to(self.current_idx);
            }
        } else if let Some(r) = self.tree.get_root() {
            self.current_idx = Some(r);
            self.rebuild_board_to(self.current_idx);
        }
    }

    /// 移动到第一个节点
    pub fn go_first(&mut self) {
        self.current_idx = self.tree.get_root();
        self.rebuild_board_to(self.current_idx);
    }

    /// 移动到最后一个节点（主变体末端）
    pub fn go_last(&mut self) {
        let mut cur = self.tree.get_root();
        while let Some(c) = cur {
            let ch = self.tree.get_children(c);
            if ch.is_empty() {
                break;
            }
            cur = Some(ch[0]);
        }
        self.current_idx = cur;
        self.rebuild_board_to(self.current_idx);
    }

    /// 移动到指定节点
    pub fn go_to(&mut self, idx: usize) {
        self.current_idx = Some(idx);
        self.rebuild_board_to(self.current_idx);
    }

    /// 获取主变体路径上的所有节点索引
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

    /// 获取当前节点的子节点着法（变体提示）
    ///
    /// 返回所有子节点的着法颜色和位置（pass 时位置为 None）
    pub fn get_variation_moves(&self) -> Vec<(Color, Option<Point>)> {
        let mut moves = Vec::new();
        let parent = self.current_idx.or(self.tree.get_root());
        if let Some(p) = parent {
            for &child in self.tree.get_children(p) {
                if let Some(node) = self.tree.get_node(child) {
                    if let Some(mv) = node_to_move(node, self.board.size) {
                        moves.push(mv);
                    }
                }
            }
        }
        moves
    }

    /// 重建棋盘到指定节点
    ///
    /// 重置棋盘状态，然后重放从根到目标节点的着法
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

        let mut is_root = true;
        for &i in &path {
            if let Some(node) = self.tree.get_node(i) {
                if is_root && self.handicap > 1 {
                    is_root = false;
                    continue;
                }
                if let Some(v) = node.get(&Property::B) {
                    if let Some(s) = v.first() {
                        if let Some(mv) = property_str_to_move(s, Color::Black, size) {
                            let (captured, ko) = self.board.apply_move_uncheck(&mv);
                            self.black_captures += captured.len();
                            self.ko_point = ko;
                        }
                    }
                }
                if let Some(v) = node.get(&Property::W) {
                    if let Some(s) = v.first() {
                        if let Some(mv) = property_str_to_move(s, Color::White, size) {
                            let (captured, ko) = self.board.apply_move_uncheck(&mv);
                            self.white_captures += captured.len();
                            self.ko_point = ko;
                        }
                    }
                }
            }
        }
    }

    /// 保存当前状态快照
    fn push_snapshot(&mut self) {
        self.history.push((self.tree.clone(), self.current_idx));
        self.future.clear();
    }

    /// 撤销操作
    pub fn undo(&mut self) {
        if let Some((prev_tree, prev_idx)) = self.history.pop() {
            self.future.push((self.tree.clone(), self.current_idx));
            self.tree = prev_tree;
            self.current_idx = prev_idx;
            self.rebuild_board_to(self.current_idx);
        }
    }

    /// 重做操作
    pub fn redo(&mut self) {
        if let Some((next_tree, next_idx)) = self.future.pop() {
            self.history.push((self.tree.clone(), self.current_idx));
            self.tree = next_tree;
            self.current_idx = next_idx;
            self.rebuild_board_to(self.current_idx);
        }
    }

    /// 判断是否可以撤销
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// 判断是否可以重做
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// 添加着法
    ///
    /// 检查着法合法性后添加到游戏树
    /// 如果该着法已作为子节点存在，则导航到该分支而非创建新节点
    pub fn add_move(&mut self, mv: Move) -> Result<(), IllegalMoveError> {
        if self.handicap > 1 {
            let mut count = 0usize;
            if let Some(c) = self.current_idx {
                let mut n = Some(c);
                while let Some(i) = n {
                    if let Some(node) = self.tree.get_node(i) {
                        if node.contains(Property::B) || node.contains(Property::W) {
                            count += 1;
                        }
                    }
                    n = self.tree.get_parent(i);
                }
            } else if let Some(root) = self.tree.get_root() {
                if let Some(node) = self.tree.get_node(root) {
                    if node.contains(Property::B) || node.contains(Property::W) {
                        count += 1;
                    }
                }
            }
            if count == 0 && mv.color != Color::White {
                return Err(IllegalMoveError::InvalidMove);
            }
        }

        self.board.is_legal(&mv, self.ko_point, false)?;

        let prop = match mv.color {
            Color::Black => Property::B,
            Color::White => Property::W,
        };
        let pt_str = mv.point.map(|p| p.to_sgf()).unwrap_or_default();

        let parent = self.current_idx.or(self.tree.get_root());
        if let Some(p) = parent {
            for &child in self.tree.get_children(p) {
                if let Some(node) = self.tree.get_node(child) {
                    if let Some(v) = node.get(&prop) {
                        if v.first().map(|s| s.as_str()) == Some(&pt_str) {
                            self.current_idx = Some(child);
                            self.rebuild_board_to(self.current_idx);
                            return Ok(());
                        }
                    }
                }
            }
        }

        self.push_snapshot();
        let (captured, ko) = self.board.apply_move_uncheck(&mv);
        match mv.color {
            Color::Black => self.black_captures += captured.len(),
            Color::White => self.white_captures += captured.len(),
        }
        self.ko_point = ko;

        let mut map = HashMap::new();
        map.insert(prop, vec![pt_str]);
        let parent = self.current_idx.or(self.tree.get_root());
        let idx = self.tree.add_node(parent, map).unwrap();
        self.current_idx = Some(idx);
        Ok(())
    }

    /// 获取下一步该谁走
    pub fn next_to_move(&self) -> Color {
        let mut count = 0usize;
        if let Some(mut n) = self.current_idx {
            while let Some(p) = self.tree.get_parent(n) {
                if let Some(node) = self.tree.get_node(n) {
                    if node.contains(Property::B) || node.contains(Property::W) {
                        count += 1;
                    }
                }
                n = p;
            }
            if let Some(node) = self.tree.get_node(n) {
                if node.contains(Property::B) || node.contains(Property::W) {
                    count += 1;
                }
            }
        } else if let Some(root) = self.tree.get_root() {
            if let Some(node) = self.tree.get_node(root) {
                if node.contains(Property::B) || node.contains(Property::W) {
                    count += 1;
                }
            }
        }
        if self.handicap > 1 {
            if count == 0 {
                Color::White
            } else if count % 2 == 1 {
                Color::Black
            } else {
                Color::White
            }
        } else {
            if count % 2 == 0 {
                Color::Black
            } else {
                Color::White
            }
        }
    }

    /// 获取节点的深度（根节点深度为 1）
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

    /// 获取当前节点的手数（包含 pass）
    pub fn current_move_number(&self) -> usize {
        let mut cnt = 0usize;
        if let Some(c) = self.current_idx {
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

    /// 获取主变体总手数（只统计含有 B 或 W 的节点数）
    pub fn total_moves(&self) -> usize {
        self.mainline()
            .iter()
            .filter(|&&idx| {
                self.tree
                    .get_node(idx)
                    .map(|n| n.contains(Property::B) || n.contains(Property::W))
                    .unwrap_or(false)
            })
            .count()
    }

    /// 获取当前着法的信息（颜色、位置、手数）
    /// pass 时位置为 None
    pub fn get_current_move_info(&self) -> Option<(Color, Option<Point>, usize)> {
        let idx = self.current_idx?;
        let node = self.tree.get_node(idx)?;
        let move_number = self.current_move_number(); // 此处已经是从根到当前的手数

        if let Some(v) = node.get(&Property::B) {
            let s = v.first()?;
            let pt = if s.is_empty() {
                None
            } else {
                Point::from_sgf(s, self.board.size)
            };
            return Some((Color::Black, pt, move_number));
        }
        if let Some(v) = node.get(&Property::W) {
            let s = v.first()?;
            let pt = if s.is_empty() {
                None
            } else {
                Point::from_sgf(s, self.board.size)
            };
            return Some((Color::White, pt, move_number));
        }
        None
    }

    /// 获取主变体所有着法（颜色、位置、手数，pass 时位置为 None）
    pub fn get_all_moves(&self) -> Vec<(Color, Option<Point>, usize)> {
        let mut moves = Vec::new();
        let mut move_number = 0usize;
        for idx in self.mainline() {
            if let Some(node) = self.tree.get_node(idx) {
                if let Some(v) = node.get(&Property::B) {
                    if let Some(s) = v.first() {
                        move_number += 1;
                        let pt = if s.is_empty() {
                            None
                        } else {
                            Point::from_sgf(s, self.board.size)
                        };
                        moves.push((Color::Black, pt, move_number));
                    }
                }
                if let Some(v) = node.get(&Property::W) {
                    if let Some(s) = v.first() {
                        move_number += 1;
                        let pt = if s.is_empty() {
                            None
                        } else {
                            Point::from_sgf(s, self.board.size)
                        };
                        moves.push((Color::White, pt, move_number));
                    }
                }
            }
        }
        moves
    }

    /// 获取从根到当前位置的所有着法（包括当前位置）
    ///
    /// 返回 (颜色, 位置, 手数)，pass 时位置为 None
    pub fn get_moves_to_current(&self) -> Vec<(Color, Option<Point>, usize)> {
        let mut moves = Vec::new();
        let mut move_number = 0usize;
        if let Some(cur) = self.current_idx {
            let mut path = Vec::new();
            let mut idx = cur;
            while let Some(parent) = self.tree.get_parent(idx) {
                path.push(idx);
                idx = parent;
            }
            path.push(idx); // 根节点
            path.reverse();
            for &node_idx in &path {
                if let Some(node) = self.tree.get_node(node_idx) {
                    if let Some(v) = node.get(&Property::B) {
                        if let Some(s) = v.first() {
                            move_number += 1;
                            let pt = if s.is_empty() {
                                None
                            } else {
                                Point::from_sgf(s, self.board.size)
                            };
                            moves.push((Color::Black, pt, move_number));
                        }
                    }
                    if let Some(v) = node.get(&Property::W) {
                        if let Some(s) = v.first() {
                            move_number += 1;
                            let pt = if s.is_empty() {
                                None
                            } else {
                                Point::from_sgf(s, self.board.size)
                            };
                            moves.push((Color::White, pt, move_number));
                        }
                    }
                }
            }
        }
        moves
    }

    /// 获取所有节点数量
    pub fn node_count(&self) -> usize {
        self.tree.nodes.iter().filter(|n| !n.deleted).count()
    }

    /// 获取指定节点的注释
    pub fn get_comment(&self, idx: usize) -> Option<String> {
        self.tree
            .get_node(idx)
            .and_then(|n| n.get(&Property::C).and_then(|v| v.first().cloned()))
    }

    /// 设置指定节点的注释
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

    /// 获取对局信息
    pub fn get_game_info(&self) -> GameInfo {
        let mut info = GameInfo::default();
        if let Some(root) = self.tree.get_root() {
            let mut idx = root;
            while idx < self.tree.nodes.len() {
                if let Some(node) = self.tree.get_node(idx) {
                    if info.komi.is_none() {
                        info.komi = node.get(&Property::KM).and_then(|v| v.first().cloned());
                    }
                    if info.rules.is_none() {
                        info.rules = node.get(&Property::RU).and_then(|v| v.first().cloned());
                    }
                    if info.black.is_none() {
                        info.black = node.get(&Property::PB).and_then(|v| v.first().cloned());
                    }
                    if info.white.is_none() {
                        info.white = node.get(&Property::PW).and_then(|v| v.first().cloned());
                    }
                    if info.black_rank.is_none() {
                        info.black_rank = node.get(&Property::BR).and_then(|v| v.first().cloned());
                    }
                    if info.white_rank.is_none() {
                        info.white_rank = node.get(&Property::WR).and_then(|v| v.first().cloned());
                    }
                    if info.event.is_none() {
                        info.event = node.get(&Property::EV).and_then(|v| v.first().cloned());
                    }
                    if info.round.is_none() {
                        info.round = node.get(&Property::RO).and_then(|v| v.first().cloned());
                    }
                    if info.place.is_none() {
                        info.place = node.get(&Property::PC).and_then(|v| v.first().cloned());
                    }
                    if info.date.is_none() {
                        info.date = node.get(&Property::DT).and_then(|v| v.first().cloned());
                    }
                    if info.result.is_none() {
                        info.result = node.get(&Property::RE).and_then(|v| v.first().cloned());
                    }
                    if info.game_name.is_none() {
                        info.game_name = node.get(&Property::GN).and_then(|v| v.first().cloned());
                    }
                    if info.handicap.is_none() {
                        info.handicap = node.get(&Property::HA).and_then(|v| v.first().cloned());
                    }
                    if info.black_team.is_none() {
                        info.black_team = node.get(&Property::BT).and_then(|v| v.first().cloned());
                    }
                    if info.white_team.is_none() {
                        info.white_team = node.get(&Property::WT).and_then(|v| v.first().cloned());
                    }
                    if info.user.is_none() {
                        info.user = node.get(&Property::US).and_then(|v| v.first().cloned());
                    }
                }
                if let Some(children) = self.tree.get_children(idx).first() {
                    idx = *children;
                } else {
                    break;
                }
            }
        }
        if info.komi.is_none() || info.komi.as_deref() == Some("") {
            if let Some(ref rules) = info.rules {
                info.komi = Some(default_komi(rules).to_string());
            }
        }
        info
    }

    /// 设置对局信息
    pub fn set_game_info(&mut self, info: &GameInfo) {
        self.push_snapshot();
        if let Some(root) = self.tree.get_root() {
            if let Some(node) = self.tree.get_node_mut(root) {
                set_info_field(node, Property::PB, &info.black);
                set_info_field(node, Property::PW, &info.white);
                set_info_field(node, Property::BR, &info.black_rank);
                set_info_field(node, Property::WR, &info.white_rank);
                set_info_field(node, Property::EV, &info.event);
                set_info_field(node, Property::RO, &info.round);
                set_info_field(node, Property::PC, &info.place);
                set_info_field(node, Property::DT, &info.date);
                set_info_field(node, Property::KM, &info.komi);
                set_info_field(node, Property::RE, &info.result);
                set_info_field(node, Property::GN, &info.game_name);
                set_info_field(node, Property::RU, &info.rules);
                set_info_field(node, Property::HA, &info.handicap);
                set_info_field(node, Property::BT, &info.black_team);
                set_info_field(node, Property::WT, &info.white_team);
                set_info_field(node, Property::US, &info.user);
            }
        }
    }

    /// 加载 SGF 游戏树（清空撤销历史）
    pub fn load_sgf(&mut self, tree: GameTree) {
        self.history.clear();
        self.future.clear();

        self.tree = tree;
        self.current_idx = self.tree.get_root();

        if let Some(idx) = self.current_idx {
            if let Some(node) = self.tree.get_node(idx) {
                let size = node
                    .get_first(Property::SZ)
                    .and_then(|s| {
                        let sz_val = s.split(':').next().unwrap_or(s);
                        sz_val.parse::<u8>().ok()
                    })
                    .unwrap_or(19);

                let handicap = node
                    .get_first(Property::HA)
                    .and_then(|s| s.parse::<u8>().ok())
                    .unwrap_or(0);

                self.board = Self::setup_board(size, handicap);
                self.handicap = handicap;
            }
        }
        self.rebuild_board_to(self.current_idx);
    }

    /// 查找指定位置的着法节点
    pub fn find_move_at_point(&self, pt: Point) -> Option<usize> {
        let sgf_str = pt.to_sgf();
        for (i, node) in self.tree.nodes.iter().enumerate().rev() {
            if node.deleted {
                continue;
            }
            if let Some(v) = node.get(&Property::B) {
                if v.first().map(|s| s == &sgf_str).unwrap_or(false) {
                    return Some(i);
                }
            }
            if let Some(v) = node.get(&Property::W) {
                if v.first().map(|s| s == &sgf_str).unwrap_or(false) {
                    return Some(i);
                }
            }
        }
        None
    }

    /// 获取指定节点的详细信息
    pub fn get_node_info(&self, idx: usize) -> Option<NodeInfo> {
        self.tree.get_node(idx).map(|node| {
            let mut kind = 0u8;
            if node.contains(Property::B) {
                kind = 1;
            }
            if node.contains(Property::W) {
                kind = 2;
            }
            let comment = node.get(&Property::C).and_then(|v| v.first().cloned());
            NodeInfo {
                kind,
                comment,
                depth: self.node_depth(idx),
            }
        })
    }

    /// 获取所有节点的（索引，节点信息）列表
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

    /// 从文件加载 SGF
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), FileError> {
        let bytes = fs::read(path)?;
        let content = Self::decode_sgf_content(&bytes);
        let tree = parse(&content).map_err(FileError::Parse)?;
        self.load_sgf(tree);
        Ok(())
    }

    /// 保存到文件
    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let s = export(&self.tree);
        fs::write(path, s)
    }

    /// 解码 SGF 文件内容
    fn decode_sgf_content(bytes: &[u8]) -> String {
        if bytes.starts_with(&[0xFF, 0xFE]) {
            let (decoded, _, had_errors) = encoding_rs::UTF_16LE.decode(bytes);
            if !had_errors {
                return decoded.into_owned();
            }
        } else if bytes.starts_with(&[0xFE, 0xFF]) {
            let (decoded, _, had_errors) = encoding_rs::UTF_16BE.decode(bytes);
            if !had_errors {
                return decoded.into_owned();
            }
        }

        let mut detector = EncodingDetector::new(Iso2022JpDetection::Allow);
        detector.feed(bytes, true);
        let detected_encoding = detector.guess(None, chardetng::Utf8Detection::Allow);

        let (result, had_errors) = detected_encoding.decode_with_bom_removal(bytes);

        if !had_errors {
            let result_str = result.to_string();
            if !result_str
                .chars()
                .any(|c: char| c.is_control() && c != '\n' && c != '\r' && c != '\t')
            {
                return result_str;
            }
        }

        let encodings_priority = [
            (encoding_rs::GBK, "GBK"),
            (encoding_rs::GB18030, "GB18030"),
            (encoding_rs::BIG5, "BIG5"),
            (encoding_rs::SHIFT_JIS, "SHIFT_JIS"),
            (encoding_rs::EUC_JP, "EUC-JP"),
            (encoding_rs::EUC_KR, "EUC-KR"),
            (encoding_rs::UTF_8, "UTF-8"),
        ];

        for (encoding, _) in encodings_priority {
            let (decoded, _, had_errors) = encoding.decode(bytes);
            if !had_errors {
                let s = decoded.to_string();
                if !s
                    .chars()
                    .any(|c: char| c.is_control() && c != '\n' && c != '\r' && c != '\t')
                {
                    return s;
                }
            }
        }

        String::from_utf8_lossy(bytes).to_string()
    }
}

// ==================== 辅助函数 ====================

/// 从节点中提取着法（颜色，位置），pass 时位置为 None
fn node_to_move(node: &Node, size: u8) -> Option<(Color, Option<Point>)> {
    if let Some(v) = node.get(&Property::B) {
        if let Some(s) = v.first() {
            let pt = if s.is_empty() {
                None
            } else {
                Point::from_sgf(s, size)
            };
            return Some((Color::Black, pt));
        }
    }
    if let Some(v) = node.get(&Property::W) {
        if let Some(s) = v.first() {
            let pt = if s.is_empty() {
                None
            } else {
                Point::from_sgf(s, size)
            };
            return Some((Color::White, pt));
        }
    }
    None
}

/// 将 SGF 属性字符串转换为着法（pass 也返回 Some）
fn property_str_to_move(s: &str, color: Color, board_size: u8) -> Option<Move> {
    if s.is_empty() {
        Some(Move::pass(color))
    } else {
        Point::from_sgf(s, board_size).map(|pt| Move::new(color, pt))
    }
}

/// 辅助：设置信息字段（存在则设置，不存在则删除）
fn set_info_field(node: &mut Node, prop: Property, val: &Option<String>) {
    if let Some(v) = val {
        if v.is_empty() {
            node.data.remove(&prop);
        } else {
            node.set(prop, vec![v.clone()]);
        }
    } else {
        // None 表示删除该属性
        node.data.remove(&prop);
    }
}

/// 根据规则获取默认贴目值
pub fn default_komi(rules: &str) -> &'static str {
    match rules {
        "Japanese" | "japanese" => "6.5",
        "Chinese" | "chinese" => "7.5",
        "AGA" | "aga" => "7.0",
        "New Zealand" | "new zealand" => "6.5",
        _ => "6.5",
    }
}

/// 节点信息结构
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// 节点类型：0=无着法，1=黑，2=白
    pub kind: u8,
    /// 注释内容
    pub comment: Option<String>,
    /// 节点深度
    pub depth: usize,
}

/// 对局信息结构
#[derive(Debug, Clone, Default)]
pub struct GameInfo {
    /// 黑方棋手
    pub black: Option<String>,
    /// 白方棋手
    pub white: Option<String>,
    /// 黑方段位
    pub black_rank: Option<String>,
    /// 白方段位
    pub white_rank: Option<String>,
    /// 赛事名称
    pub event: Option<String>,
    /// 轮次
    pub round: Option<String>,
    /// 对局地点
    pub place: Option<String>,
    /// 对局日期
    pub date: Option<String>,
    /// 贴目
    pub komi: Option<String>,
    /// 对局结果
    pub result: Option<String>,
    /// 棋谱名称
    pub game_name: Option<String>,
    /// 规则
    pub rules: Option<String>,
    /// 让子数
    pub handicap: Option<String>,
    /// 黑方队伍
    pub black_team: Option<String>,
    /// 白方队伍
    pub white_team: Option<String>,
    /// 录入者
    pub user: Option<String>,
}
