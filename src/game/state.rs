use crate::{
    game::record::{GameHistory, ZobristHash},
    model::*,
    rule::legality::KoRule,
};


/// 当前游戏局面
#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    /// 当前轮到谁下
    pub current_color: Color,
    /// 最新的一步棋
    pub last_move: Option<Move>,
    /// 当前劫点
    pub ko_point: Option<Point>,
    /// (黑提白子数, 白提黑子数)
    pub captures: (usize, usize),
    pub zobrist: ZobristHash,
    /// 当前局面哈希
    pub current_hash: u64,
    /// 历史记录用于劫检测和历史回溯
    pub history: GameHistory,
    /// 是否允许自杀
    pub allow_suicide: bool,
    /// 劫规则
    pub ko_rule: KoRule,
}

impl GameState {
    pub fn new_with_rules(board_size: u8, ko_rule: KoRule) -> Self {
        let zobrist = ZobristHash::new(board_size, 0x123456789abcdef0);
        let board = Board::new(board_size);
        let current_hash = zobrist.compute(&board, Color::Black);
        Self {
            board,
            current_color: Color::Black,
            last_move: None,
            ko_point: None,
            captures: (0, 0),
            zobrist,
            current_hash,
            history: GameHistory::new(1000),
            allow_suicide: false,
            ko_rule,
        }
    }

    pub fn apply_move(&mut self, mv: Move) -> bool {
        // 检查颜色是否匹配
        if mv.color != self.current_color {
            return false;
        }
        // 调用 board.apply_move，传入当前 ko_point
        if let Some((captured, new_ko)) = self.board.apply_move(&mv, self.ko_point, false) {
            // 更新提子数
            match mv.color {
                Color::Black => self.captures.1 += captured.len(), // 白被提
                Color::White => self.captures.0 += captured.len(), // 黑被提
            }
            // 更新状态
            self.last_move = Some(mv);
            self.ko_point = new_ko;
            self.current_color = mv.color.opposite();
            true
        } else {
            false
        }
    }
}
