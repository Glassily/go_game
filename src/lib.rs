//! Go SGF Parser - 围棋棋谱解析库
//!
//! 支持: 解析/导出/验证 SGF FF4 格式棋谱
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
pub use record::{GameInfo, GoRecord};
pub use sgf::{
    GameTree, Node, ParseError, Property, SgfExporter, SgfParser, SgfValidator, TreeError,
    ValidationError, ValidationResult,
};
pub use sgf::{export, parse, validate, validate_with_strict};
