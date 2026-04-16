use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::game::record::{GameRecord, NodeProperties};
use crate::model::{Color, Move, Point};

#[derive(Debug)]
pub enum SgfError {
    Eof,
    InvalidChar(char, usize),
    UnterminatedValue(usize),
    InvalidCoord(String, usize),
    InvalidTreeStructure(usize),
    ParseError(String),
}

impl fmt::Display for SgfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SgfError::Eof => write!(f, "Unexpected end of input"),
            SgfError::InvalidChar(c, pos) => write!(f, "Invalid character '{}' at pos {}", c, pos),
            SgfError::UnterminatedValue(pos) => {
                write!(f, "Unterminated property value at pos {}", pos)
            }
            SgfError::InvalidCoord(s, pos) => {
                write!(f, "Invalid coordinate '{}' at pos {}", s, pos)
            }
            SgfError::InvalidTreeStructure(pos) => {
                write!(f, "Invalid tree structure at pos {}", pos)
            }
            SgfError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for SgfError {}

pub struct SgfParser {
    input: Vec<char>,
    pos: usize, // 当前解析位置
    board_size: u8,
    record: GameRecord,
}

impl SgfParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            board_size: 19, // SGF 默认
            record: GameRecord::default(),
        }
    }

    /// 公开入口：解析完整 SGF 字符串
    pub fn parse(mut self) -> Result<GameRecord, SgfError> {
        self.skip_whitespace();
        self.parse_collection()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(SgfError::InvalidChar(self.input[self.pos], self.pos));
        }
        Ok(self.record)
    }

    // 核心递归下降

    fn parse_collection(&mut self) -> Result<(), SgfError> {
        self.expect_char('(')?;
        self.parse_game_tree()?;
        self.expect_char(')')?;
        Ok(())
    }

    fn parse_game_tree(&mut self) -> Result<(), SgfError> {
        let parent_idx = if let Some(&last) = self.record.current_path.last() {
            last
        } else {
            0 // 根节点
        };
        self.parse_node_sequence(parent_idx)?;
        Ok(())
    }

    fn parse_node_sequence(&mut self, mut parent_idx: usize) -> Result<(), SgfError> {
        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                break;
            }

            match self.input[self.pos] {
                ';' => {
                    self.pos += 1;
                    let props = self.parse_properties()?;
                    let is_root = self.record.tree.nodes.len() == 1;

                    // 创建新节点
                    let (move_data, node_props) = self.apply_props(&props, is_root)?;

                    if is_root {
                        self.merge_root_props(&node_props);
                        self.record.tree.nodes[0].move_data = move_data;
                    } else {
                        let new_idx = self
                            .record
                            .tree
                            .add_child(parent_idx, move_data, node_props);
                        parent_idx = new_idx;
                        // 同步 current_path（仅主线）
                        if self.record.current_path.last() == Some(&parent_idx) {
                            // 已在路径中，无需操作
                        } else {
                            self.record.current_path.push(new_idx);
                        }
                    }
                }
                '(' => {
                    // 分支解析时保存
                    let path_snapshot = self.record.current_path.clone();
                    let parent_snapshot = parent_idx;

                    self.pos += 1;
                    self.parse_game_tree()?;
                    self.expect_char(')')?;
                    // 恢复状态到分支起点
                    self.record.current_path = path_snapshot;
                    parent_idx = parent_snapshot;
                }
                ')' => break,
                _ => return Err(SgfError::InvalidChar(self.input[self.pos], self.pos)),
            }
        }
        Ok(())
    }

    // 属性解析

    /// 解析连续属性，直到遇到分隔符 (;()或空白)
    fn parse_properties(&mut self) -> Result<Vec<(String, Vec<String>)>, SgfError> {
        let mut props = Vec::new();
        self.skip_whitespace();

        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            if c == ';' || c == '(' || c == ')' || c.is_whitespace() {
                break;
            }

            let prop_name = self.parse_prop_name()?;
            let mut values = Vec::new();

            // SGF 允许同一属性多次出现: AB[aa][ab]
            while self.pos < self.input.len() && self.input[self.pos] == '[' {
                let val = self.parse_prop_value()?;
                values.push(val);
                self.skip_whitespace();
            }
            props.push((prop_name, values));
            self.skip_whitespace();
        }
        Ok(props)
    }

    /// 解析属性名称（连续大写字母）
    fn parse_prop_name(&mut self) -> Result<String, SgfError> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_uppercase() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(SgfError::InvalidChar(
                self.input.get(self.pos).copied().unwrap_or('\0'),
                self.pos,
            ));
        }
        Ok(self.input[start..self.pos].iter().collect())
    }

    /// 解析属性值，处理转义和多行
    fn parse_prop_value(&mut self) -> Result<String, SgfError> {
        self.expect_char('[')?;
        let start = self.pos;
        let mut escaped = false;

        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == ']' {
                let raw = &self.input[start..self.pos];
                self.pos += 1;
                return Ok(self.unescape_value(raw));
            }
            self.pos += 1;
        }
        Err(SgfError::UnterminatedValue(start))
    }

    fn unescape_value(&self, raw: &[char]) -> String {
        let mut out = String::with_capacity(raw.len());
        let mut i = 0;
        while i < raw.len() {
            match raw[i] {
                '\\' if i + 1 < raw.len() => {
                    match raw[i + 1] {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        ']' => out.push(']'),
                        '\\' => out.push('\\'),
                        '\n' => {} // 忽略换行转义（SGF 规范）
                        c => out.push(c),
                    }
                    i += 2;
                }
                c => {
                    out.push(c);
                    i += 1;
                }
            }
        }
        out
    }

    /// 属性映射到数据模型
    fn apply_props(
        &mut self,
        props: &[(String, Vec<String>)],
        is_root: bool,
    ) -> Result<(Option<Move>, NodeProperties), SgfError> {
        let mut node_props = NodeProperties::default();
        let mut move_data = None;
        let mut raw_props = HashMap::new();

        for (name, values) in props {
            match name.as_str() {
                "B" | "W" => {
                    let color = if name == "B" {
                        Color::Black
                    } else {
                        Color::White
                    };
                    let pt = if values.is_empty() || values[0].is_empty() {
                        None // Pass
                    } else {
                        Some(self.parse_sgf_point(&values[0])?)
                    };
                    move_data = Some(Move { color, point: pt });
                }
                "C" => node_props.comment = values.first().cloned().unwrap_or_default(),
                "LB" => {
                    for v in values {
                        if let Some((coord, label)) = v.split_once(':') {
                            if let Ok(pt) = self.parse_sgf_point(coord) {
                                node_props.labels.insert(pt, label.to_string());
                            }
                        }
                    }
                }
                "AB" | "AW" => {
                    let color = if name == "AB" {
                        Color::Black
                    } else {
                        Color::White
                    };
                    for v in values {
                        let pt = self.parse_sgf_point(v)?;
                        node_props.setup.push((pt, color));
                    }
                }
                // 根节点元数据
                "SZ" => {
                    if is_root {
                        let s = values
                            .first()
                            .map(|s| s.split(':').next().unwrap_or(s))
                            .unwrap_or("19");
                        self.board_size = s.parse().unwrap_or(19);
                        self.record.info.board_size = self.board_size;
                    }
                }
                "KM" => {
                    if is_root {
                        self.record.info.komi =
                            values.first().and_then(|s| s.parse().ok()).unwrap_or(6.5);
                    }
                }
                "HA" => {
                    if is_root {
                        self.record.info.handicap =
                            values.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                    }
                }
                "PB" => {
                    if is_root {
                        self.record.info.black_name = values.first().cloned().unwrap_or_default();
                    }
                }
                "PW" => {
                    if is_root {
                        self.record.info.white_name = values.first().cloned().unwrap_or_default();
                    }
                }
                "RE" => {
                    if is_root {
                        self.record.info.result = values.first().cloned().unwrap_or_default();
                    }
                }
                "DT" => {
                    if is_root {
                        self.record.info.date = values.first().cloned().unwrap_or_default();
                    }
                }
                "RU" => {
                    if is_root {
                        self.record.info.rules = values.first().cloned().unwrap_or_default();
                    }
                }
                _ => {
                    // 保留未识别属性，保证无损导出
                    raw_props.insert(name.clone(), values.clone());
                }
            }
        }
        node_props.raw_sgf_props = raw_props;
        Ok((move_data, node_props))
    }

    /// 合并根节点属性而非覆盖
    fn merge_root_props(&mut self, new_props: &NodeProperties) {
        let root = self.record.tree.get_mut(0).unwrap();
        if !new_props.comment.is_empty() {
            root.props.comment.clone_from(&new_props.comment);
        }
        root.props.labels.extend(new_props.labels.clone());
        root.props.setup.extend(new_props.setup.clone());
        root.props.annotations.extend(new_props.annotations.clone());
        root.props
            .raw_sgf_props
            .extend(new_props.raw_sgf_props.clone());
    }

    fn parse_sgf_point(&self, s: &str) -> Result<Point, SgfError> {
        Point::from_sgf(s, self.board_size)
            .ok_or_else(|| SgfError::InvalidCoord(s.to_string(), self.pos))
    }

    // 辅助方法

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), SgfError> {
        if self.pos >= self.input.len() {
            return Err(SgfError::Eof);
        }
        if self.input[self.pos] == expected {
            self.pos += 1;
            Ok(())
        } else {
            Err(SgfError::InvalidChar(self.input[self.pos], self.pos))
        }
    }
}

// 验证/导出逻辑

#[derive(Debug, Clone, PartialEq)]
pub enum SgfValidationError {
    // 阻断性错误 (棋谱无法对弈或渲染)
    InvalidBoardSize(u8),
    CoordinateOutOfBounds(Point, u8),
    PointAlreadyOccupied(Point),
    TurnOrderViolation {
        expected: Color,
        actual: Color,
        node_idx: usize,
    },
    HandicapFirstMoveMustBeWhite(usize),
    DoublePassWithoutResult(usize),
    // 警告 (不阻断运行，但建议修复)
    Warning(String, usize),
}

impl fmt::Display for SgfValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SgfValidationError::InvalidBoardSize(s) => write!(f, "Invalid board size: {}", s),
            SgfValidationError::CoordinateOutOfBounds(p, s) => {
                write!(f, "Coordinate {:?} out of bounds for {}x{}", p, s, s)
            }
            SgfValidationError::PointAlreadyOccupied(p) => {
                write!(f, "Point {:?} is already occupied", p)
            }
            SgfValidationError::TurnOrderViolation {
                expected,
                actual,
                node_idx,
            } => {
                write!(
                    f,
                    "Turn mismatch at node {}: expected {}, got {}",
                    node_idx, expected, actual
                )
            }
            SgfValidationError::HandicapFirstMoveMustBeWhite(idx) => {
                write!(
                    f,
                    "Handicap game: first move should be White at node {}",
                    idx
                )
            }
            SgfValidationError::DoublePassWithoutResult(idx) => {
                write!(
                    f,
                    "Consecutive passes without result marker at node {}",
                    idx
                )
            }
            SgfValidationError::Warning(msg, idx) => write!(f, "Warning at node {}: {}", idx, msg),
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<SgfValidationError>,
    pub warnings: Vec<SgfValidationError>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// 轻量级虚拟棋盘 (仅用于校验，不计算气/提子)
#[derive(Clone)]
struct ValidationBoard {
    grid: Vec<Option<Color>>,
    size: u8,
    next_turn: Color,
    last_was_pass: bool,
}

impl ValidationBoard {
    fn new(size: u8, handicap: u8) -> Self {
        Self {
            grid: vec![None; (size * size) as usize],
            size,
            next_turn: if handicap > 1 {
                Color::White
            } else {
                Color::Black
            },
            last_was_pass: false,
        }
    }
    fn idx(&self, p: Point) -> usize {
        (p.y * self.size + p.x) as usize
    }
    fn is_empty(&self, p: &Point) -> bool {
        p.x < self.size && p.y < self.size && self.grid[self.idx(*p)].is_none()
    }
    fn place(&mut self, p: &Point, c: Color) {
        let idx = self.idx(*p);
        self.grid[idx] = Some(c);
    }
}

/// 公开校验入口
pub fn validate_sgf(record: &GameRecord) -> ValidationResult {
    let mut res = ValidationResult::default();

    // 1. 基础元数据校验
    if record.info.board_size < 2 || record.info.board_size > 26 {
        res.errors
            .push(SgfValidationError::InvalidBoardSize(record.info.board_size));
        return res;
    }
    if ![9, 13, 19].contains(&record.info.board_size) {
        res.warnings.push(SgfValidationError::Warning(
            "非标准棋盘尺寸 (推荐 9/13/19)".into(),
            0,
        ));
    }

    // 2. 递归校验整棵树
    let mut board = ValidationBoard::new(record.info.board_size, record.info.handicap);
    validate_tree(record, record.tree.root_index, &mut board, true, &mut res);
    res
}

fn validate_tree(
    record: &GameRecord,
    idx: usize,
    board: &mut ValidationBoard,
    is_root: bool,
    res: &mut ValidationResult,
) {
    let node = &record.tree.nodes[idx];

    // 1.处理摆子/让子 (AB/AW) - 不改变回合顺序
    for &(pt, color) in &node.props.setup {
        if pt.x >= record.info.board_size || pt.y >= record.info.board_size {
            res.errors.push(SgfValidationError::CoordinateOutOfBounds(
                pt,
                record.info.board_size,
            ));
        } else if !board.is_empty(&pt) {
            res.warnings.push(SgfValidationError::Warning(
                format!("覆盖交叉点 {:?}", pt),
                idx,
            ));
        } else {
            board.place(&pt, color);
        }
    }

    // 2.处理着手 (B/W)
    if let Some(mv) = &node.move_data {
        // 校验回合顺序
        if mv.color != board.next_turn {
            if is_root && record.info.handicap > 1 {
                res.errors
                    .push(SgfValidationError::HandicapFirstMoveMustBeWhite(idx));
            } else {
                res.errors.push(SgfValidationError::TurnOrderViolation {
                    expected: board.next_turn,
                    actual: mv.color,
                    node_idx: idx,
                });
            }
        }

        // 校验坐标与占位
        if let Some(pt) = mv.point {
            if pt.x >= record.info.board_size || pt.y >= record.info.board_size {
                res.errors.push(SgfValidationError::CoordinateOutOfBounds(
                    pt,
                    record.info.board_size,
                ));
            } else if !board.is_empty(&pt) {
                res.errors
                    .push(SgfValidationError::PointAlreadyOccupied(pt));
            } else {
                board.place(&pt, mv.color);
            }
        } else {
            // Pass 处理
            if board.last_was_pass && record.info.result.is_empty() {
                res.errors
                    .push(SgfValidationError::DoublePassWithoutResult(idx));
            }
            board.last_was_pass = true;
        }

        // 切换回合
        board.next_turn = mv.color.opposite();
        board.last_was_pass = mv.point.is_none();
    } else if !is_root {
        // 非根节点无着手数据 (通常是纯注释/摆子节点)
        board.last_was_pass = false;
    }

    // 3.递归处理子节点 (分支独立状态)
    if !node.children.is_empty() {
        for &child_idx in &node.children {
            let mut child_board = board.clone(); // 19x19 Vec 克隆 < 1μs
            validate_tree(record, child_idx, &mut child_board, false, res);
        }
    }
}

impl Point {}

impl GameRecord {
    /// 导出为 SGF 字符串
    pub fn to_sgf(&self) -> String {
        let mut out = String::new();
        self.write_to_sgf(&mut out).unwrap();
        out
    }

    /// 流式写入 SGF（推荐用于大文件或网络传输）
    pub fn write_to_sgf(&self, f: &mut impl Write) -> Result<(), fmt::Error> {
        write!(f, "(")?;
        self.write_node_sequence(self.tree.root_index, f)?;
        write!(f, ")")?;
        Ok(())
    }

    /// 递归写入节点序列（主线直线展开，分支包裹括号）
    fn write_node_sequence(&self, idx: usize, f: &mut impl Write) -> Result<(), fmt::Error> {
        let node = &self.tree.nodes[idx];
        write!(f, ";")?;
        self.write_node_properties(idx, f)?;

        if !node.children.is_empty() {
            // 主线：第一个子节点继续当前序列
            self.write_node_sequence(node.children[0], f)?;

            // 分支：后续子节点作为独立变招包裹在 () 中
            for &child_idx in node.children.iter().skip(1) {
                write!(f, "(")?;
                self.write_node_sequence(child_idx, f)?;
                write!(f, ")")?;
            }
        }
        Ok(())
    }

    /// 写入节点属性
    fn write_node_properties(&self, idx: usize, f: &mut impl Write) -> Result<(), fmt::Error> {
        let node = &self.tree.nodes[idx];
        let is_root = idx == self.tree.root_index;

        // 1. 根节点元数据（按 SGF 惯例顺序）
        if is_root {
            self.write_prop(f, "GM", &["1"])?;
            self.write_prop(f, "FF", &["4"])?;
            self.write_prop(f, "SZ", &[&self.info.board_size.to_string()])?;
            self.write_prop(f, "KM", &[&format_komi(self.info.komi)])?;
            self.write_prop_if_not_empty(f, "PB", &self.info.black_name)?;
            self.write_prop_if_not_empty(f, "PW", &self.info.white_name)?;
            self.write_prop_if_not_empty(f, "RE", &self.info.result)?;
            self.write_prop_if_not_empty(f, "DT", &self.info.date)?;
            self.write_prop_if_not_empty(f, "RU", &self.info.rules)?;
        }

        // 2. 着手信息
        if let Some(mv) = node.move_data {
            let prop = if mv.color == Color::Black { "B" } else { "W" };
            let coord = match mv.point {
                Some(p) => p.to_sgf(),
                None => String::new(), // Pass 记为空 []
            };
            self.write_prop(f, prop, &[&coord])?;
        }

        // 3. 注释与标记
        if !node.props.comment.is_empty() {
            self.write_prop(f, "C", &[&node.props.comment])?;
        }

        // 批量输出 LB 属性
        if !node.props.labels.is_empty() {
            let lbs: Vec<String> = node
                .props
                .labels
                .iter()
                .map(|(pt, label)| format!("{}:{}", pt.to_sgf(), label))
                .collect();
            let refs: Vec<&str> = lbs.iter().map(|s| s.as_str()).collect();
            self.write_prop(f, "LB", &refs)?;
        }

        // 批量输出 AB/AW
        if !node.props.setup.is_empty() {
            let mut ab = Vec::new();
            let mut aw = Vec::new();
            for (pt, color) in &node.props.setup {
                if *color == Color::Black {
                    ab.push(pt.to_sgf());
                } else {
                    aw.push(pt.to_sgf());
                }
            }
            if !ab.is_empty() {
                let refs: Vec<&str> = ab.iter().map(|s| s.as_str()).collect();
                self.write_prop(f, "AB", &refs)?;
            }
            if !aw.is_empty() {
                let refs: Vec<&str> = aw.iter().map(|s| s.as_str()).collect();
                self.write_prop(f, "AW", &refs)?;
            }
        }

        // 4. 保留原始未知属性（保证无损往返）
        for (name, values) in &node.props.raw_sgf_props {
            let escaped: Vec<String> = values.iter().map(|v| escape_sgf(v)).collect();
            let refs: Vec<&str> = escaped.iter().map(|s| s.as_str()).collect();
            self.write_prop(f, name, &refs)?;
        }

        Ok(())
    }

    /// 辅助：写入标准属性格式 `NAME[val1][val2]`
    fn write_prop(
        &self,
        f: &mut impl Write,
        name: &str,
        values: &[&str],
    ) -> Result<(), fmt::Error> {
        write!(f, "{}", name)?;
        for v in values {
            write!(f, "[{}]", escape_sgf(v))?;
        }
        Ok(())
    }

    /// 辅助：仅当非空时写入
    fn write_prop_if_not_empty(
        &self,
        f: &mut impl Write,
        name: &str,
        val: &str,
    ) -> Result<(), fmt::Error> {
        if !val.is_empty() {
            self.write_prop(f, name, &[val])?;
        }
        Ok(())
    }
}

/// SGF 合规数值格式化（6.5 -> "6.5", 0 -> "0"）
fn format_komi(k: f32) -> String {
    if k.fract() == 0.0 {
        format!("{}", k as i32)
    } else {
        format!("{:.1}", k)
    }
}

/// SGF 值转义（FF4 规范）
fn escape_sgf(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ']' => out.push_str("\\]"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::game::record::GameTree;

    use super::*;

    #[test]
    fn test_gametree_default_safe() {
        let tree = GameTree::default();
        assert!(!tree.nodes.is_empty());
        assert_eq!(tree.root_index, 0);
    }

    #[test]
    fn test_parse_branching_path_recovery() {
        let sgf = "(;FF[4]SZ[9];B[aa];W[bb](;B[cc])(;B[dd]))";
        let record = SgfParser::new(sgf).parse().unwrap();
        //let w_bb_idx = record.tree.root_index + 2;
        //assert_eq!(record.tree.get(w_bb_idx).unwrap().children.len(), 2);
        let board = record.current_board();
        println!("{}", board);
    }

    #[test]
    fn test_sgf_roundtrip() {
        let original = "(;FF[4]SZ[19]KM[6.5];B[pd];W[dd]C[好棋])";
        let record = SgfParser::new(original).parse().unwrap();
        let exported = record.to_sgf();
        let record2 = SgfParser::new(&exported).parse().unwrap();
        assert_eq!(record.info.komi, record2.info.komi);
        assert_eq!(record.tree.nodes.len(), record2.tree.nodes.len());
    }
}
