pub mod ko;
pub mod record;
pub mod sgf;
pub mod state;
pub mod zobrist;

use crate::game::{record::GameRecord, state::GameState};

/// 游戏状态管理器
#[derive(Debug, Clone)]
pub struct Game {
    pub record: GameRecord,
    pub state: GameState,
}

impl Game {}
