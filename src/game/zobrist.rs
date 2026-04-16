use crate::model::*;

use std::hash::{Hash, Hasher};

#[warn(unused)]
// ============== 局面哈希（Zobrist Hashing） ==============
/// 用于高效检测重复局面（劫、循环劫等）
#[derive(Debug, Clone)]
pub struct ZobristHash {
    // 随机数表: [x][y][color] + 当前轮次
    table: Vec<Vec<[u64; 3]>>, // 2: Black/White + Empty(占位)
    ko_hash: u64,
}

impl ZobristHash {
    pub fn new(size: u8, seed: u64) -> Self {
        use std::collections::hash_map::DefaultHasher;

        let mut table = vec![vec![[0u64; 3]; size as usize]; size as usize];

        // 伪随机生成（实际使用建议用真随机或预计算表）
        for y in 0..size {
            for x in 0..size {
                for i in 0..3 {
                    let mut hasher = DefaultHasher::new();
                    (seed
                        .wrapping_mul(31)
                        .wrapping_add(x as u64)
                        .wrapping_mul(37)
                        .wrapping_add(y as u64)
                        .wrapping_mul(41)
                        .wrapping_add(i as u64))
                    .hash(&mut hasher);
                    table[y as usize][x as usize][i] = hasher.finish();
                }
            }
        }

        Self {
            table,
            ko_hash: seed.wrapping_add(0x9e3779b97f4a7c15), // 轮次影响
        }
    }

    /// 获取单个位置的哈希贡献
    fn get_piece_hash(&self, pt: Point, color: Option<Color>) -> u64 {
        let idx = match color {
            None => 2, // Empty
            Some(Color::Black) => 0,
            Some(Color::White) => 1,
        };
        self.table[pt.y as usize][pt.x as usize][idx]
    }

    /// 计算完整棋盘哈希
    pub fn compute(&self, board: &Board, current_color: Color) -> u64 {
        let mut hash: u64 = self.ko_hash;
        if current_color == Color::White {
            hash = hash.wrapping_add(0x5555555555555555);
        }

        for y in 0..board.size {
            for x in 0..board.size {
                let pt = Point { x, y };
                if let Some(color) = board.get(pt) {
                    hash ^= self.get_piece_hash(pt, Some(color));
                }
            }
        }
        hash
    }

    /// 增量更新哈希（落子/提子时调用，避免重复计算）
    pub fn update(&self, current: u64, pt: Point, old: Option<Color>, new: Option<Color>) -> u64 {
        current ^ self.get_piece_hash(pt, old) ^ self.get_piece_hash(pt, new)
    }
}

/// 局面历史用于检测循环
#[derive(Debug, Clone)]
pub struct GameHistory {
    /// 哈希值历史（用于快速比较）
    hashes: Vec<u64>,
    /// 完整局面历史（用于精确验证，可选）
    boards: Vec<Board>,
    /// 最大保存历史长度
    max_history: usize,
}

impl GameHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            hashes: Vec::with_capacity(max_history),
            boards: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// 记录新局面
    pub fn record(&mut self, board: &Board, current_color: Color, zobrist: &ZobristHash) {
        let hash = zobrist.compute(board, current_color);

        self.hashes.push(hash);

        // 可选：保存完整局面用于调试或精确验证
        self.boards.push(board.clone());

        // 限制历史长度
        if self.hashes.len() > self.max_history {
            self.hashes.remove(0);
            // if !self.boards.is_empty() { self.boards.remove(0); }
        }
    }

    /// 检测是否形成循环（用于判无胜负）
    pub fn detect_cycle(&self, current_hash: u64, min_cycle_len: usize) -> Option<usize> {
        // 从后向前查找重复哈希
        for (i, &h) in self.hashes.iter().rev().enumerate() {
            if h == current_hash && i + 1 >= min_cycle_len {
                return Some(i + 1); // 返回循环长度
            }
        }
        None
    }

    /// 可选：精确验证循环（避免哈希碰撞误判）
    pub fn verify_cycle(&self, current_board: &Board, current_color: Color, zobrist: &ZobristHash) -> bool {
        let current_hash = zobrist.compute(current_board, current_color);
        for (i, &h) in self.hashes.iter().rev().enumerate() {
            if h == current_hash {
                // 可能是循环，进行精确比较
                if let Some(prev_board) = self.boards.get(self.boards.len() - 1 - i) {
                    if prev_board == current_board {
                        return true; // 确认循环
                    }
                }
            }
        }
        false
    }

}
