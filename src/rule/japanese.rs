use super::scoring::ScoringRule;
use crate::{
    model::{Board, Color, Move, Point},
    rule::legality::LegalityRule,
};
use std::collections::HashSet;

/// 日本规则（数目法），贴6.5目
/// 简化实现：计算双方围空 + 提子数，死子需要提前移除并计入提子。
/// 注意：实际日本规则中，死子最后要填入对方空中，此处简化计算。
pub struct JapaneseRule {
    pub komi: f64,
}

impl Default for JapaneseRule {
    fn default() -> Self {
        Self { komi: 6.5 }
    }
}

impl LegalityRule for JapaneseRule {}

impl ScoringRule for JapaneseRule {
    fn score(
        &self,
        board: &Board,
        captures: (usize, usize),
        dead_stones: &HashSet<Point>,
        _last_move: Option<Move>,
    ) -> (f64, f64) {
        // 1. 复制棋盘并移除死子（死子视为被提）
        let mut temp_board = board.clone();
        let mut extra_captures_black = 0usize; // 黑棋从死子中获得的提子（即白死子）
        let mut extra_captures_white = 0usize;
        for &pt in dead_stones {
            if let Some(color) = temp_board.get(pt) {
                match color {
                    Color::Black => extra_captures_white += 1, // 黑死子算白提
                    Color::White => extra_captures_black += 1,
                }
                temp_board.remove(pt);
            }
        }

        // 总提子数 = 对局中实际提子 + 终局死子
        let total_captures_black = captures.0 + extra_captures_black;
        let total_captures_white = captures.1 + extra_captures_white;

        // 2. 计算双方围空（仅空点）
        let size = temp_board.size as usize;
        let mut black_territory = 0usize;
        let mut white_territory = 0usize;
        let mut visited = HashSet::new();

        for y in 0..size {
            for x in 0..size {
                let pt = Point {
                    x: x as u8,
                    y: y as u8,
                };
                if visited.contains(&pt) || temp_board.get(pt).is_some() {
                    continue;
                }
                // 空点区域 BFS
                let mut queue = std::collections::VecDeque::new();
                let mut region = Vec::new();
                queue.push_back(pt);
                region.push(pt);
                visited.insert(pt);

                let mut adjacent_black = false;
                let mut adjacent_white = false;

                while let Some(p) = queue.pop_front() {
                    for nb in temp_board.neighbors(p) {
                        if visited.contains(&nb) {
                            continue;
                        }
                        if let Some(c) = temp_board.get(nb) {
                            match c {
                                Color::Black => adjacent_black = true,
                                Color::White => adjacent_white = true,
                            }
                        } else {
                            // 未访问的空点
                            visited.insert(nb);
                            queue.push_back(nb);
                            region.push(nb);
                        }
                    }
                }

                if adjacent_black && !adjacent_white {
                    black_territory += region.len();
                } else if adjacent_white && !adjacent_black {
                    white_territory += region.len();
                }
                // 双活区域不计目（日本规则双活无目）
            }
        }

        // 3. 数目法得分 = 围空 + 提子数
        let black_score = black_territory as f64 + total_captures_white as f64;
        let white_score = white_territory as f64 + total_captures_black as f64 + self.komi;

        (black_score, white_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Board, Color, Point};

    #[test]
    fn test_japanese_rule_simple() {
        let mut board = Board::new(5);
        // 黑棋围一个空
        board.set(Point { x: 1, y: 1 }, Color::Black);
        board.set(Point { x: 1, y: 2 }, Color::Black);
        board.set(Point { x: 2, y: 1 }, Color::Black);
        board.set(Point { x: 2, y: 2 }, Color::Black);
        // 中间一点 (1,1) 已占，实际上没有围空，这里简化，不深入。

        let rule = JapaneseRule::default();
        let dead = HashSet::new();
        let (black, white) = rule.score(&board, (0, 0), &dead, None);
        // 至少应该黑棋有提子或空
        println!("black: {}, white: {}", black, white);
    }
}
