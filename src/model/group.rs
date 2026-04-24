use std::collections::HashSet;

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