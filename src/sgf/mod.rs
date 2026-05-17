//! SGF 模块
//!
//! 提供 SGF（Smart Game Format）格式的解析、导出和验证功能

mod exporter;
mod parser;
mod property;
mod tree;
mod validator;

pub use exporter::SgfExporter;
pub use parser::{ParseError, SgfParser};
pub use property::Property;
pub use tree::{GameTree, Node, TreeError};
pub use validator::{SgfValidator, ValidationError, ValidationResult};

/// 解析 SGF 格式字符串
///
/// # 示例
/// ```
/// use go_game::parse;
///
/// let sgf = "(;FF[4]SZ[19]KM[6.5];B[pd];W[dd])";
/// let tree = parse(sgf).unwrap();
/// ```
pub fn parse(sgf: &str) -> Result<GameTree, ParseError> {
    SgfParser::new(sgf).parse()
}

/// 解析 SGF（严格模式）
///
/// 严格模式下，未知属性名将被视为错误
pub fn parse_with_strict(sgf: &str) -> Result<GameTree, ParseError> {
    SgfParser::new(sgf).strict().parse()
}

/// 导出游戏树为 SGF 格式字符串
pub fn export(tree: &GameTree) -> String {
    SgfExporter::new(tree).export()
}

/// 验证游戏树的有效性
pub fn validate(tree: &GameTree) -> ValidationResult {
    SgfValidator::new().validate(tree)
}

/// 验证游戏树（严格模式）
///
/// 严格模式下，未知属性将被报告为警告
pub fn validate_with_strict(tree: &GameTree) -> ValidationResult {
    SgfValidator::new().strict().validate(tree)
}
