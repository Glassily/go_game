/// 棋局数据
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub black_name: String,
    pub white_name: String,
    pub komi: f32,      // 贴目，通常 6.5 或 7.5
    pub handicap: u8,   // 让子数，0 表示无让子
    pub result: String, // e.g., "B+R", "W+1.5", "Draw"
    pub date: String,   // e.g., "2024-10-01"
    pub board_size: u8, // 通常 19
    pub rules: String,  // "Japanese", "Chinese", "AGA"
}

impl GameInfo {
    pub fn new() -> Self {
        Self {
            black_name: String::new(),
            white_name: String::new(),
            komi: 6.5,
            handicap: 0,
            result: String::new(),
            date: String::new(),
            board_size: 19,
            rules: "Japanese".to_string(),
        }
    }
}

impl Default for GameInfo {
    fn default() -> Self {
        Self::new()
    }
}