use crate::sgf::property::Property;
use crate::sgf::tree::GameTree;
use std::fmt::Write;

pub struct SgfExporter<'a> {
    tree: &'a GameTree,
}

impl<'a> SgfExporter<'a> {
    pub fn new(tree: &'a GameTree) -> Self {
        Self { tree }
    }

    pub fn export(&self) -> String {
        let mut out = String::new();
        self.write_collection(&mut out).unwrap();
        out
    }

    fn write_collection(&self, f: &mut String) -> std::fmt::Result {
        f.push('(');
        if let Some(root) = self.tree.get_root() {
            self.write_node_sequence(root, f)?;
        }
        f.push(')');
        Ok(())
    }

    fn write_node_sequence(&self, idx: usize, f: &mut String) -> std::fmt::Result {
        let node = self.tree.get_node(idx).unwrap();
        f.push(';');
        self.write_properties(node, f)?;
        let children = self.tree.get_children(idx);
        if children.len() == 1 {
            self.write_node_sequence(children[0], f)?;
        } else if children.len() > 1 {
            for &child in children {
                f.push('(');
                self.write_node_sequence(child, f)?;
                f.push(')');
            }
        }
        Ok(())
    }

    fn write_properties(&self, node: &crate::sgf::tree::Node, f: &mut String) -> std::fmt::Result {
        let order = [
            Property::GM,
            Property::FF,
            Property::SZ,
            Property::KM,
            Property::HA,
            Property::PB,
            Property::PW,
            Property::RE,
            Property::DT,
            Property::RU,
            Property::B,
            Property::W,
            Property::AB,
            Property::AW,
            Property::AE,
            Property::C,
        ];
        for prop in &order {
            if let Some(vals) = node.get(prop.clone()) {
                self.write_prop(f, &prop.to_string(), vals)?;
            }
        }
        for (prop, vals) in &node.data {
            if !order.iter().any(|p| p == prop) {
                self.write_prop(f, &prop.to_string(), vals)?;
            }
        }
        Ok(())
    }

    fn write_prop(&self, f: &mut String, name: &str, values: &[String]) -> std::fmt::Result {
        for v in values {
            write!(f, "{}[{}]", name, Self::escape(v))?;
        }
        Ok(())
    }

    fn escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        for c in s.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                ']' => out.push_str("\\]"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                c => out.push(c),
            }
        }
        out
    }
}
