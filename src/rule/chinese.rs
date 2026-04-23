use super::scoring::ScoringRule;
use crate::{
    model::{Board, Color, Move, Point},
    rule::legality::{KoRule, LegalityRule},
};
use std::collections::{HashSet, VecDeque};

/// 中国规则（数子法），贴3¾子（即7.5目）
/// 简化实现：移除死子后，数黑棋棋子 + 黑棋围的空，白棋同理。
/// 注意：双活棋中的空按均分处理（此处简化，均分给双方）。
pub struct ChineseRule {
    /// 贴子数，默认3.75
    pub komi: f64,
}

impl Default for ChineseRule {
    fn default() -> Self {
        Self { komi: 3.75 }
    }
}

impl LegalityRule for ChineseRule {
    /// 劫规则，全局禁同
    fn ko_rule(&self) -> KoRule {
        KoRule::PositionalSuperKo
    }

    /// 检查落子合法性
    fn is_legal(
        &self,
        board: &Board,
        mv: &Move,
        last_ko_point: Option<Point>,
        // 历史局面列表用于超级劫检测
        history: Option<&[Board]>,
    ) -> bool {
        if !board.is_legal(mv, last_ko_point, self.allow_suicide()) {
            return false;
        }
        // 超级劫：检查全局局面是否重复
        if let KoRule::PositionalSuperKo = self.ko_rule() {
            if let Some(hist) = history {
                let mut temp = board.clone();
                // 模拟落子后的局面
                temp.set(mv.point.unwrap(), mv.color);
                // 移除无气棋子
                temp.remove_dead_groups(mv);
                if hist.contains(&temp) {
                    return false;
                }
            }
        }
        true
    }
}

impl ScoringRule for ChineseRule {
    fn score(
        &self,
        board: &Board,
        _captures: (usize, usize),
        dead_stones: &HashSet<Point>,
        _last_move: Option<Move>,
    ) -> (f64, f64) {
        // 1. 复制棋盘并移除死子
        let mut temp_board = board.clone();
        for &pt in dead_stones {
            temp_board.remove(pt);
        }

        let size = temp_board.size as usize;
        let mut black_area = 0usize;
        let mut white_area = 0usize;

        // 2. 数子 + 数空（简单洪水填充法）
        let mut visited = HashSet::new();

        for y in 0..size {
            for x in 0..size {
                let pt = Point {
                    x: x as u8,
                    y: y as u8,
                };
                if visited.contains(&pt) {
                    continue;
                }
                visited.insert(pt);

                // 如果有点上有棋子，直接计数
                if let Some(color) = temp_board.get(pt) {
                    match color {
                        Color::Black => black_area += 1,
                        Color::White => white_area += 1,
                    }
                    continue;
                }

                // 空点：做 BFS 找出连通空域，并判断该空域被哪方包围
                let mut queue = VecDeque::new();
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

                // 根据相邻棋子决定归属
                let area_size = region.len();
                if adjacent_black && !adjacent_white {
                    black_area += area_size;
                } else if adjacent_white && !adjacent_black {
                    white_area += area_size;
                } else if adjacent_black && adjacent_white {
                    // 双活：均分（整数除法可能丢失，这里按浮点处理）
                    black_area += area_size / 2;
                    white_area += area_size - area_size / 2;
                }
                // 若都不相邻（不可能，边界外不会出现）
            }
        }

        // 3. 加上提子（数子法不提子，因为子空皆地，提子已反映在对方棋子减少中）
        //    但为了与贴目对齐，中国规则中提子不影响胜负，因为已经包含在围空中。
        //    这里不再额外加 captures。

        let black_score = black_area as f64;
        let white_score = white_area as f64 + self.komi;

        (black_score, white_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Board;

    #[test]
    fn test_chinese_rule_empty() {
        let board = Board::new(9);
        let rule = ChineseRule::default();
        let dead = HashSet::new();
        let (black, white) = rule.score(&board, (0, 0), &dead, None);
        // 9x9 共81个点，黑贴3.75子 => 黑: 81/2 - 3.75? 不对，空棋盘双方空域均分？
        // 实际空棋盘双方无子，所有空点相邻双方，应均分。黑得分 = 40.5，白得分 = 40.5 + 3.75 = 44.25
        assert!((black - 40.5).abs() < 0.1);
        assert!((white - 44.25).abs() < 0.1);
    }
}
