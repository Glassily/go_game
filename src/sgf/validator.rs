use crate::model::{Color as ModelColor, Point};
use crate::sgf::property::Property;
use crate::sgf::tree::GameTree;
use crate::{Board, IllegalMoveError, Move};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    InvalidBoardSize {
        value: String,
        node_idx: usize,
    },
    CoordinateOutOfBounds {
        coord: String,
        board_size: u8,
        node_idx: usize,
    },
    PointOccupied {
        coord: String,
        node_idx: usize,
    },
    TurnOrderViolation {
        expected: ModelColor,
        actual: ModelColor,
        node_idx: usize,
    },
    HandicapFirstMoveMustBeWhite {
        node_idx: usize,
    },
    DoublePassWithoutResult {
        node_idx: usize,
    },
    InvalidPropertyValue {
        prop: String,
        value: String,
        node_idx: usize,
    },
    MissingRequiredProperty {
        prop: String,
        node_idx: usize,
    },
    NonStandardBoardSize {
        value: u8,
        node_idx: usize,
    },
    UnknownProperty {
        prop: String,
        node_idx: usize,
    },
    DuplicateProperty {
        prop: String,
        node_idx: usize,
    },
    SetupOverwritesStone {
        coord: String,
        node_idx: usize,
    },
    EmptyTree,
    InvalidNodeReference {
        index: usize,
        context: String,
    },
}

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}
impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

pub struct SgfValidator {
    strict: bool,
}
impl SgfValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn validate(&self, tree: &GameTree) -> ValidationResult {
        let mut result = ValidationResult::default();
        let root_idx = match tree.get_root() {
            Some(idx) => idx,
            None => {
                result.errors.push(ValidationError::EmptyTree);
                return result;
            }
        };

        let (board_size, handicap) = match self.parse_root_metadata(tree, root_idx, &mut result) {
            Ok(v) => v,
            Err(_) => return result,
        };

        let initial_board = Board::new(board_size);
        // 应用根节点摆子
        let mut board =
            self.apply_setup(&initial_board, tree.get_node(root_idx).unwrap(), board_size);
        board = self.apply_handicap(&board, handicap);

        let initial_next_turn = if handicap > 1 {
            ModelColor::White
        } else {
            ModelColor::Black
        };
        self.validate_node(
            tree,
            root_idx,
            &board,
            true,
            board_size,
            handicap,
            initial_next_turn,
            None,
            false,
            &mut result,
        );
        result
    }

    fn parse_root_metadata(
        &self,
        tree: &GameTree,
        root_idx: usize,
        result: &mut ValidationResult,
    ) -> Result<(u8, u8), ()> {
        let node = tree.get_node(root_idx).ok_or(())?;
        let board_size = match node.get_first(Property::SZ) {
            Some(s) => {
                let sz = s.split(':').next().unwrap_or(s);
                match sz.parse::<u8>() {
                    Ok(sz) if (2..=26).contains(&sz) => sz,
                    _ => {
                        result.errors.push(ValidationError::InvalidBoardSize {
                            value: s.clone(),
                            node_idx: root_idx,
                        });
                        return Err(());
                    }
                }
            }
            None => 19,
        };
        if ![9, 13, 19].contains(&board_size) {
            result.warnings.push(ValidationError::NonStandardBoardSize {
                value: board_size,
                node_idx: root_idx,
            });
        }
        let handicap = node
            .get_first(Property::HA)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        Ok((board_size, handicap))
    }

    fn apply_setup(&self, board: &Board, node: &crate::sgf::tree::Node, size: u8) -> Board {
        let mut b = board.clone();
        for (prop, values) in &node.data {
            if *prop == Property::AB || *prop == Property::AW {
                let color = if *prop == Property::AB {
                    ModelColor::Black
                } else {
                    ModelColor::White
                };
                for coord in values {
                    if let Some(pt) = Point::from_sgf(coord, size) {
                        b.set(pt, color);
                    }
                }
            } else if *prop == Property::AE {
                for coord in values {
                    if let Some(pt) = Point::from_sgf(coord, size) {
                        b.remove(pt);
                    }
                }
            }
        }
        b
    }

    fn apply_handicap(&self, board: &Board, handicap: u8) -> Board {
        // 让子位置预设 (简化: 星位)
        if handicap <= 1 {
            return board.clone();
        }
        let mut b = board.clone();
        let size = board.size;
        let star_points = self.get_handicap_points(handicap, size);
        for pt in star_points {
            if pt.is_valid(size) {
                b.set(pt, ModelColor::Black);
            }
        }
        b
    }

    fn get_handicap_points(&self, handicap: u8, size: u8) -> Vec<Point> {
        // 简化: 返回星位坐标
        let mut pts = Vec::new();
        let stars = [
            (3, 3),
            (3, size - 4),
            (size - 4, 3),
            (size - 4, size - 4),
            (size / 2, 3),
            (size / 2, size - 4),
            (3, size / 2),
            (size - 4, size / 2),
            (size / 2, size / 2),
        ];
        for &(x, y) in &stars[..(handicap as usize).min(stars.len())] {
            pts.push(Point { x, y });
        }
        pts
    }

    fn validate_node(
        &self,
        tree: &GameTree,
        idx: usize,
        board: &Board,
        is_root: bool,
        board_size: u8,
        handicap: u8,
        mut next_turn: ModelColor,
        mut ko_point: Option<Point>,
        mut last_was_pass: bool,
        result: &mut ValidationResult,
    ) {
        let node = match tree.get_node(idx) {
            Some(n) => n,
            None => {
                result.errors.push(ValidationError::InvalidNodeReference {
                    index: idx,
                    context: "validation".into(),
                });
                return;
            }
        };
    
        let mut current_board = board.clone();
        if is_root && handicap > 1 {
            next_turn = ModelColor::White;
        }
    
        let mut seen = HashSet::new();
        for (prop, values) in &node.data {
            if !seen.insert(prop.clone()) {
                result.warnings.push(ValidationError::DuplicateProperty {
                    prop: prop.to_string(),
                    node_idx: idx,
                });
            }
            if self.strict && !prop.is_known() {
                result.warnings.push(ValidationError::UnknownProperty {
                    prop: prop.to_string(),
                    node_idx: idx,
                });
            }
    
            match *prop {
                Property::B | Property::W => {
                    let expected = if *prop == Property::B {
                        ModelColor::Black
                    } else {
                        ModelColor::White
                    };
                    if expected != next_turn {
                        if !(handicap > 1
                            && is_root
                            && expected == ModelColor::Black
                            && next_turn == ModelColor::White)
                        {
                            result.errors.push(ValidationError::TurnOrderViolation {
                                expected: next_turn,
                                actual: expected,
                                node_idx: idx,
                            });
                        }
                    }
    
                    let coord = values.first().map(|s| s.as_str()).unwrap_or("");
                    if coord.is_empty() {
                        if last_was_pass {
                            result
                                .errors
                                .push(ValidationError::DoublePassWithoutResult { node_idx: idx });
                        }
                        last_was_pass = true;
                        ko_point = None; // pass 清除劫点
                    } else {
                        last_was_pass = false;
                        match Point::from_sgf(coord, board_size) {
                            Some(pt) => {
                                // 直接使用 apply_move，由其完成所有合法性检查
                                let mv = Move::new(expected, pt);
                                match current_board.apply_move(&mv, ko_point, false) {
                                    Ok((_captured, new_ko)) => {
                                        // 着法合法，棋盘已更新，劫点更新
                                        ko_point = new_ko;
                                    }
                                    Err(e) => {
                                        // 将棋步违规映射为具体验证错误
                                        match e {
                                            IllegalMoveError::OutOfBounds => {
                                                result.errors.push(
                                                    ValidationError::CoordinateOutOfBounds {
                                                        coord: coord.into(),
                                                        board_size,
                                                        node_idx: idx,
                                                    },
                                                );
                                            }
                                            IllegalMoveError::Occupied => {
                                                result.errors.push(ValidationError::PointOccupied {
                                                    coord: coord.into(),
                                                    node_idx: idx,
                                                });
                                            }
                                            // 劫、自杀、无效着法统一归为非法着法值
                                            IllegalMoveError::KoViolation
                                            | IllegalMoveError::Suicide
                                            | IllegalMoveError::InvalidMove => {
                                                result.errors.push(
                                                    ValidationError::InvalidPropertyValue {
                                                        prop: prop.to_string(),
                                                        value: coord.into(),
                                                        node_idx: idx,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            None => {
                                result.errors.push(ValidationError::InvalidPropertyValue {
                                    prop: prop.to_string(),
                                    value: coord.into(),
                                    node_idx: idx,
                                });
                            }
                        }
                    }
                    next_turn = expected.opposite();
                }
                Property::AB | Property::AW | Property::AE => {
                    for coord in values {
                        if let Some(pt) = Point::from_sgf(coord, board_size) {
                            if !pt.is_valid(board_size) {
                                result.errors.push(ValidationError::CoordinateOutOfBounds {
                                    coord: coord.clone(),
                                    board_size,
                                    node_idx: idx,
                                });
                            }
                        }
                    }
                }
                Property::KM => {
                    for v in values {
                        if v.parse::<f32>().is_err()
                            || v.parse::<f32>().unwrap() < -100.0
                            || v.parse::<f32>().unwrap() > 100.0
                        {
                            result.errors.push(ValidationError::InvalidPropertyValue {
                                prop: "KM".into(),
                                value: v.clone(),
                                node_idx: idx,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    
        for &child in tree.get_children(idx) {
            self.validate_node(
                tree,
                child,
                &current_board,
                false,
                board_size,
                handicap,
                next_turn,
                ko_point,
                last_was_pass,
                result,
            );
        }
    }
}
impl Default for SgfValidator {
    fn default() -> Self {
        Self::new()
    }
}
