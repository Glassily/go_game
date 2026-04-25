use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use crate::model::{Color, Point};

/// 块群（由多个同色连通块通过共享气连接而成）
#[derive(Debug, Clone)]
pub struct GroupSet {
    /// 块群的颜色
    pub color: Color,
    /// 块群包含的所有棋子点
    pub points: HashSet<Point>,
    // 块群的所有气（外部空点）
    pub liberties: HashSet<Point>,
}

/// 空区域（连通空点集）
#[derive(Debug, Clone)]
pub struct EmptyRegion {
    /// 区域内的空点
    pub points: HashSet<Point>,
    /// 边界棋子的颜色集合      
    pub border_colors: HashSet<Color>,
    /// 是否接触棋盘边界
    pub touches_edge: bool,
}

impl GroupSet {
    pub fn to_string_gtp(&self, board_size: u8) -> String {
        let mut points: Vec<&Point> = self.points.iter().collect();
        points.sort();
        let mut liberties: Vec<&Point> = self.liberties.iter().collect();
        liberties.sort();

        format!(
            "GroupSet {{\n  color: {:?},\n  points: [{}],\n  liberties: [{}]\n}}",
            self.color,
            points
                .iter()
                .map(|p| format!("{}", p.to_gtp(board_size)))
                .collect::<Vec<_>>()
                .join(", "),
            liberties
                .iter()
                .map(|p| format!("{}", p.to_gtp(board_size)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}


impl EmptyRegion {
    pub fn to_string_gtp(&self, board_size: u8) -> String {
        let mut points: Vec<&Point> = self.points.iter().collect();
        points.sort();
        let mut border_colors: Vec<&Color> = self.border_colors.iter().collect();
        border_colors.sort();

        format!(
            "EmptyRegion {{\n  points: [{}],\n  border_colors: {:?},\n  touches_edge: {}\n}}",
            points
                .iter()
                .map(|p| format!("{}", p.to_gtp(board_size)))
                .collect::<Vec<_>>()
                .join(", "),
            border_colors,
            self.touches_edge
        )
    }
}

impl PartialEq for GroupSet {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color && self.points == other.points
    }
}

impl Eq for GroupSet {}

impl PartialEq for EmptyRegion {
    fn eq(&self, other: &Self) -> bool {
        self.points == other.points
    }
}

impl Eq for EmptyRegion {}

impl Display for GroupSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut points: Vec<&Point> = self.points.iter().collect();
        points.sort();
        let mut liberties: Vec<&Point> = self.liberties.iter().collect();
        liberties.sort();

        write!(
            f,
            "GroupSet {{\n  color: {:?},\n  points: [{}],\n  liberties: [{}]\n}}",
            self.color,
            points
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", "),
            liberties
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Display for EmptyRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut points: Vec<&Point> = self.points.iter().collect();
        points.sort();
        let mut border_colors: Vec<&Color> = self.border_colors.iter().collect();
        border_colors.sort();

        write!(
            f,
            "EmptyRegion {{\n  points: [{}],\n  border_colors: {:?},\n  touches_edge: {}\n}}",
            points
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", "),
            border_colors,
            self.touches_edge
        )
    }
}
