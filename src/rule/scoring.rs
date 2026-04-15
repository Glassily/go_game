use std::collections::HashSet;
use crate::model::{Board, Color, Point, Move};

/// 胜负判定规则
pub trait ScoringRule {
    /// 计算双方最终分数（已考虑贴目）
    ///
    /// # Arguments
    /// * `board` - 棋盘状态（未移除死子）
    /// * `captures` - (黑棋提白子数, 白棋提黑子数)
    /// * `dead_stones` - 双方一致同意的死子点集
    /// * `last_move` - 最后一步棋（可用于判定虚手等）
    ///
    /// # Returns
    /// (黑方分数, 白方分数)
    fn score(
        &self,
        board: &Board,
        captures: (usize, usize),
        dead_stones: &HashSet<Point>,
        last_move: Option<Move>,
    ) -> (f64, f64);

    /// 根据分数返回胜者
    fn winner(&self, black_score: f64, white_score: f64) -> Option<Color> {
        if black_score > white_score {
            Some(Color::Black)
        } else if white_score > black_score {
            Some(Color::White)
        } else {
            None // 平局
        }
    }

    /// 便利方法：直接返回胜者
    fn determine_winner(
        &self,
        board: &Board,
        captures: (usize, usize),
        dead_stones: &HashSet<Point>,
        last_move: Option<Move>,
    ) -> Option<Color> {
        let (black, white) = self.score(board, captures, dead_stones, last_move);
        self.winner(black, white)
    }
}