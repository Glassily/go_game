use crate::sgf::property::Property;
use crate::sgf::tree::{GameTree, TreeError};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Eof,
    InvalidChar(char, usize),
    UnterminatedValue(usize),
    InvalidProperty(String, usize),
    TreeError(TreeError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::Eof => write!(f, "Unexpected EOF"),
            ParseError::InvalidChar(c, p) => write!(f, "Invalid '{}' at {}", c, p),
            ParseError::UnterminatedValue(p) => write!(f, "Unterminated value at {}", p),
            ParseError::InvalidProperty(n, p) => write!(f, "Unknown property '{}' at {}", n, p),
            ParseError::TreeError(e) => write!(f, "Tree error: {}", e),
        }
    }
}
impl std::error::Error for ParseError {}
impl From<TreeError> for ParseError {
    fn from(e: TreeError) -> Self {
        ParseError::TreeError(e)
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;

pub struct SgfParser {
    input: Vec<char>,
    pos: usize,
    tree: GameTree,
    board_size: u8,
    strict: bool,
}

impl SgfParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            tree: GameTree::new(),
            board_size: 19,
            strict: false,
        }
    }

    /// Enable strict parsing: unknown property names are treated as errors.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn parse(mut self) -> Result<GameTree> {
        self.skip_ws();
        self.parse_collection()?;
        self.skip_ws();
        if self.pos < self.input.len() {
            return Err(ParseError::InvalidChar(self.input[self.pos], self.pos));
        }
        Ok(self.tree)
    }

    fn parse_collection(&mut self) -> Result<()> {
        self.expect('(')?;
        self.parse_game_tree()?;
        self.expect(')')?;
        Ok(())
    }

    fn parse_game_tree(&mut self) -> Result<()> {
        let mut parent_stack = vec![None];
        loop {
            self.skip_ws();
            if self.pos >= self.input.len() {
                break;
            }
            match self.input[self.pos] {
                ';' => {
                    self.pos += 1;
                    let props = self.parse_properties()?;
                    let parent = *parent_stack.last().unwrap();
                    let idx = self.tree.add_node(parent, props)?;

                    if self.tree.get_root().is_none() {
                        self.tree.root_index = Some(idx);
                        // 解析 SZ
                        if let Some(sz) = self.tree.get_node(idx).unwrap().get_first(Property::SZ) {
                            let sz_val = sz.split(':').next().unwrap_or(sz);
                            self.board_size = sz_val.parse().unwrap_or(19);
                        }
                    }
                    *parent_stack.last_mut().unwrap() = Some(idx);
                }
                '(' => {
                    self.pos += 1;
                    parent_stack.push(*parent_stack.last().unwrap());
                    self.parse_game_tree()?;
                    self.expect(')')?;
                    parent_stack.pop();
                }
                ')' => break,
                _ => return Err(ParseError::InvalidChar(self.input[self.pos], self.pos)),
            }
        }
        Ok(())
    }

    fn parse_properties(&mut self) -> Result<HashMap<Property, Vec<String>>> {
        let mut props = HashMap::new();
        loop {
            self.skip_ws();
            if self.pos >= self.input.len() {
                break;
            }
            let c = self.input[self.pos];
            if c == ';' || c == '(' || c == ')' || c.is_whitespace() {
                break;
            }

            let name = self.parse_prop_name()?;
            let prop = Property::from_str(&name);

            // 如果开启严格模式，未知属性（Other）视为错误
            if self.strict && !prop.is_known() {
                return Err(ParseError::InvalidProperty(name.clone(), self.pos));
            }

            let mut values = Vec::new();
            while self.pos < self.input.len() && self.input[self.pos] == '[' {
                values.push(self.parse_prop_value()?);
                self.skip_ws();
            }
            props.insert(prop, values);
        }
        Ok(props)
    }

    fn parse_prop_name(&mut self) -> Result<String> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_uppercase() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ParseError::InvalidChar(
                self.input.get(self.pos).copied().unwrap_or('\0'),
                self.pos,
            ));
        }
        Ok(self.input[start..self.pos].iter().collect())
    }

    fn parse_prop_value(&mut self) -> Result<String> {
        self.expect('[')?;
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
                return Ok(Self::unescape(raw));
            }
            self.pos += 1;
        }
        Err(ParseError::UnterminatedValue(start))
    }

    fn unescape(raw: &[char]) -> String {
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
                        '\n' => {} // SGF: ignore escaped newline
                        c => {
                            out.push('\\');
                            out.push(c);
                        }
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

    fn expect(&mut self, ch: char) -> Result<()> {
        if self.pos >= self.input.len() {
            return Err(ParseError::Eof);
        }
        if self.input[self.pos] == ch {
            self.pos += 1;
            Ok(())
        } else {
            Err(ParseError::InvalidChar(self.input[self.pos], self.pos))
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
}
