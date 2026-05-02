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

/// 解析sgf
pub fn parse(sgf: &str) -> Result<GameTree, ParseError> {
    SgfParser::new(sgf).parse()
}

/// 解析sgf(严格模式，遇到未知属性会报错)
pub fn parse_with_strict(sgf: &str) -> Result<GameTree, ParseError> {
    SgfParser::new(sgf).strict().parse()
}

/// 导出sgf
pub fn export(tree: &GameTree) -> String {
    SgfExporter::new(tree).export()
}

/// 验证GameTree
pub fn validate(tree: &GameTree) -> ValidationResult {
    SgfValidator::new().validate(tree)
}

/// 验证GameTree(严格模式，遇到未知属性会报错)
pub fn validate_with_strict(tree: &GameTree) -> ValidationResult {
    SgfValidator::new().strict().validate(tree)
}
