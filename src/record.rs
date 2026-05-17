use crate::board::{Board, IllegalMoveError};
use crate::model::{Color, Move, Point};
use crate::sgf::{GameTree, Property};

/// 围棋对局记录结构
///
/// 包含完整的游戏树、当前节点位置、棋盘状态、提子计数等信息
pub struct GoRecord {
    /// 游戏树（包含所有节点和着法）
    pub tree: GameTree,
    /// 当前节点索引
    pub current: Option<usize>,
    /// 棋盘状态
    pub board: Board,
    /// 黑方提子数（白方被吃的子数）
    pub black_captures: usize,
    /// 白方提子数（黑方被吃的子数）
    pub white_captures: usize,
    /// 历史版本栈（用于撤销）
    history: Vec<GameTree>,
    /// 未来版本栈（用于重做）
    future: Vec<GameTree>,
    /// 当前劫点位置
    ko_point: Option<Point>,
}

impl Default for GoRecord {
    fn default() -> Self {
        let mut root_data = std::collections::HashMap::new();
        root_data.insert(Property::GM, vec!["1".to_string()]);
        root_data.insert(Property::FF, vec!["4".to_string()]);
        root_data.insert(Property::SZ, vec!["19".to_string()]);
        root_data.insert(Property::RU, vec!["Japanese".to_string()]);
        root_data.insert(Property::KM, vec!["6.5".to_string()]);
        let tree = GameTree::from(root_data);

        Self {
            tree,
            current: None,
            board: Board::new(19),
            black_captures: 0,
            white_captures: 0,
            history: Vec::new(),
            future: Vec::new(),
            ko_point: None,
        }
    }
}

impl GoRecord {
    /// 创建指定大小的空对局记录
    pub fn new(size: u8) -> Self {
        let mut root_data = std::collections::HashMap::new();
        root_data.insert(Property::GM, vec!["1".to_string()]);
        root_data.insert(Property::FF, vec!["4".to_string()]);
        root_data.insert(Property::SZ, vec![size.to_string()]);
        let tree = GameTree::from(root_data);

        Self {
            tree,
            current: None,
            board: Board::new(size),
            black_captures: 0,
            white_captures: 0,
            history: Vec::new(),
            future: Vec::new(),
            ko_point: None,
        }
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
        if let Some(c) = self.current {
            self.current = self.tree.get_parent(c);
            self.rebuild_board_to(self.current);
        }
    }

    /// 移动到下一个节点（主变体）
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

    /// 移动到第一个节点
    pub fn go_first(&mut self) {
        self.current = self.tree.get_root();
        self.rebuild_board_to(self.current);
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
        self.current = cur;
        self.rebuild_board_to(self.current);
    }

    /// 移动到指定节点
    pub fn go_to(&mut self, idx: usize) {
        self.current = Some(idx);
        self.rebuild_board_to(self.current);
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
    /// 返回所有子节点的着法位置和颜色
    pub fn get_variation_moves(&self) -> Vec<(Color, Point)> {
        let mut moves = Vec::new();
        if let Some(c) = self.current {
            for &child in self.tree.get_children(c) {
                if let Some(node) = self.tree.get_node(child) {
                    let size = self.board.size;
                    if let Some(v) = node.get(&Property::B) {
                        if let Some(s) = v.first() {
                            if let Some(pt) = Point::from_sgf(s, size) {
                                moves.push((Color::Black, pt));
                            }
                        }
                    }
                    if let Some(v) = node.get(&Property::W) {
                        if let Some(s) = v.first() {
                            if let Some(pt) = Point::from_sgf(s, size) {
                                moves.push((Color::White, pt));
                            }
                        }
                    }
                }
            }
        } else if let Some(r) = self.tree.get_root() {
            for &child in self.tree.get_children(r) {
                if let Some(node) = self.tree.get_node(child) {
                    let size = self.board.size;
                    if let Some(v) = node.get(&Property::B) {
                        if let Some(s) = v.first() {
                            if let Some(pt) = Point::from_sgf(s, size) {
                                moves.push((Color::Black, pt));
                            }
                        }
                    }
                    if let Some(v) = node.get(&Property::W) {
                        if let Some(s) = v.first() {
                            if let Some(pt) = Point::from_sgf(s, size) {
                                moves.push((Color::White, pt));
                            }
                        }
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

        for &i in &path {
            if let Some(node) = self.tree.get_node(i) {
                if let Some(v) = node.get(&Property::B) {
                    if let Some(s) = v.first() {
                        if let Some(mv) = property_str_to_move(s, Color::Black, size) {
                            let (captured, ko) = self.board.apply_move_uncheck(&mv);
                            self.black_captures += captured.len() as usize;
                            self.ko_point = ko;
                        }
                    }
                }
                if let Some(v) = node.get(&Property::W) {
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

    /// 保存当前状态快照
    fn push_snapshot(&mut self) {
        self.history.push(self.tree.clone());
        self.future.clear();
    }

    /// 撤销操作
    pub fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.future.push(self.tree.clone());
            self.tree = prev;
            self.current = self.tree.get_root();
            self.rebuild_board_to(self.current);
        }
    }

    /// 重做操作
    pub fn redo(&mut self) {
        if let Some(next) = self.future.pop() {
            self.history.push(self.tree.clone());
            self.tree = next;
            self.current = self.tree.get_root();
            self.rebuild_board_to(self.current);
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
        if let Err(e) = self.board.is_legal(&mv, self.ko_point, false) {
            return Err(e);
        }

        let (captured, ko) = self.board.apply_move_uncheck(&mv);
        match mv.color {
            Color::Black => self.black_captures += captured.len() as usize,
            Color::White => self.white_captures += captured.len() as usize,
        }
        self.ko_point = ko;

        let prop = match mv.color {
            Color::Black => Property::B,
            Color::White => Property::W,
        };
        let pt_str = mv.point.map(|p| p.to_sgf()).unwrap_or_default();

        // 检查该着法是否已作为子节点存在
        if let Some(cur) = self.current {
            for &child in self.tree.get_children(cur) {
                if let Some(node) = self.tree.get_node(child) {
                    if let Some(v) = node.get(&prop) {
                        if v.first().map(|s| s.as_str()) == Some(&pt_str) {
                            // 该着法已存在，导航到该分支
                            self.rebuild_board_to(Some(child));
                            self.current = Some(child);
                            return Ok(());
                        }
                    }
                }
            }
        } else if let Some(root) = self.tree.get_root() {
            for &child in self.tree.get_children(root) {
                if let Some(node) = self.tree.get_node(child) {
                    if let Some(v) = node.get(&prop) {
                        if v.first().map(|s| s.as_str()) == Some(&pt_str) {
                            // 该着法已存在，导航到该分支
                            self.rebuild_board_to(Some(child));
                            self.current = Some(child);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // 着法不存在，创建新节点
        self.push_snapshot();
        let mut map = std::collections::HashMap::new();
        map.insert(prop, vec![pt_str]);
        let parent = self.current.or(self.tree.get_root());
        let _ = self.tree.add_node(parent, map);
        self.current = Some(self.tree.nodes.len() - 1);
        Ok(())
    }

    /// 获取下一步该谁走
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
        if count % 2 == 0 {
            Color::Black
        } else {
            Color::White
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

    /// 获取当前节点的手数
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

    /// 获取主变体总手数
    pub fn total_moves(&self) -> usize {
        self.mainline().len()
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
            if let Some(node) = self.tree.get_node(root) {
                info.black = node.get(&Property::PB).and_then(|v| v.first().cloned());
                info.white = node.get(&Property::PW).and_then(|v| v.first().cloned());
                info.black_rank = node.get(&Property::BR).and_then(|v| v.first().cloned());
                info.white_rank = node.get(&Property::WR).and_then(|v| v.first().cloned());
                info.event = node.get(&Property::EV).and_then(|v| v.first().cloned());
                info.round = node.get(&Property::RO).and_then(|v| v.first().cloned());
                info.place = node.get(&Property::PC).and_then(|v| v.first().cloned());
                info.date = node.get(&Property::DT).and_then(|v| v.first().cloned());
                info.komi = node.get(&Property::KM).and_then(|v| v.first().cloned());
                info.result = node.get(&Property::RE).and_then(|v| v.first().cloned());
                info.game_name = node.get(&Property::GN).and_then(|v| v.first().cloned());
                info.rules = node.get(&Property::RU).and_then(|v| v.first().cloned());
                info.handicap = node.get(&Property::HA).and_then(|v| v.first().cloned());
                info.black_team = node.get(&Property::BT).and_then(|v| v.first().cloned());
                info.white_team = node.get(&Property::WT).and_then(|v| v.first().cloned());
                info.user = node.get(&Property::US).and_then(|v| v.first().cloned());
            }
        }
        info
    }

    /// 设置对局信息
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
                if let Some(ref v) = info.black_rank {
                    node.set(Property::BR, vec![v.clone()]);
                }
                if let Some(ref v) = info.white_rank {
                    node.set(Property::WR, vec![v.clone()]);
                }
                if let Some(ref v) = info.event {
                    node.set(Property::EV, vec![v.clone()]);
                }
                if let Some(ref v) = info.round {
                    node.set(Property::RO, vec![v.clone()]);
                }
                if let Some(ref v) = info.place {
                    node.set(Property::PC, vec![v.clone()]);
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
                if let Some(ref v) = info.game_name {
                    node.set(Property::GN, vec![v.clone()]);
                }
                if let Some(ref v) = info.rules {
                    node.set(Property::RU, vec![v.clone()]);
                }
                if let Some(ref v) = info.handicap {
                    node.set(Property::HA, vec![v.clone()]);
                }
                if let Some(ref v) = info.black_team {
                    node.set(Property::BT, vec![v.clone()]);
                }
                if let Some(ref v) = info.white_team {
                    node.set(Property::WT, vec![v.clone()]);
                }
                if let Some(ref v) = info.user {
                    node.set(Property::US, vec![v.clone()]);
                }
            }
        }
    }

    /// 加载 SGF 游戏树
    pub fn load_sgf(&mut self, tree: GameTree) {
        self.push_snapshot();
        self.tree = tree;
        self.current = self.tree.get_root();
        if let Some(idx) = self.current {
            if let Some(node) = self.tree.get_node(idx) {
                if let Some(sz) = node.get_first(Property::SZ) {
                    let sz_val = sz.split(':').next().unwrap_or(sz);
                    if let Ok(board_size) = sz_val.parse::<u8>() {
                        self.board = Board::new(board_size);
                    }
                }
            }
        }
        self.rebuild_board_to(self.current);
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
}

/// 将 SGF 属性字符串转换为着法
fn property_str_to_move(s: &str, color: Color, board_size: u8) -> Option<Move> {
    if s.is_empty() {
        Some(Move::pass(color))
    } else {
        Point::from_sgf(s, board_size).map(|pt| Move::new(color, pt))
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
