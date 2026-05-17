//! Go Game Library - 围棋 SGF 解析库
//!
//! 提供围棋棋谱的解析、导出、验证和编辑功能
//!
//! # 主要功能
//! - SGF FF4 格式棋谱解析和导出
//! - 围棋棋盘和着法管理
//! - 棋谱验证（检查着法合法性、劫规则等）
//!
//! # 示例
//! ```
//! use go_game::{parse, export, validate};
//!
//! let sgf = "(;FF[4]SZ[19]KM[6.5];B[pd];W[dd])";
//! let tree = parse(sgf).unwrap();
//! let exported = export(&tree);
//! let result = validate(&tree);
//! assert!(result.is_valid());
//! ```

pub mod board;
pub mod model;
pub mod record;
pub mod sgf;

pub use board::{Board, IllegalMoveError};
pub use model::{Color, Move, Point};
pub use record::{GameInfo, GoRecord, NodeInfo};
pub use sgf::{
    GameTree, Node, ParseError, Property, SgfExporter, SgfParser, SgfValidator, TreeError,
    ValidationError, ValidationResult,
};
pub use sgf::{export, parse, validate, validate_with_strict};
