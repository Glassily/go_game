use crate::model::Point;

/// 眼的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeType {
    /// 真眼：所有对角点都被同色占据或边界
    Real,
    /// 假眼：至少一个对角点被对方占据或为空
    False,
    /// 半眼：需要补一手才能成真眼
    Half,
}

/// 棋子状态（用于死活分析）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupStatus {
    /// 活棋：两只或以上真眼
    Alive,
    /// 死棋：无法做出两只眼
    Dead,
    /// 双活：双方共享气，互不能吃
    Seki,
    /// 未定：需要进一步分析
    Uncertain,
    /// 劫活/劫杀：依赖劫争结果
    Ko,
}

/// 眼位分析结果
#[derive(Debug, Clone)]
pub struct EyeAnalysis {
    /// 棋块所有的眼及类型
    pub eyes: Vec<(Point, EyeType)>,
    /// 真眼数量
    pub real_eye_count: usize,
    /// 棋块状态
    pub status: GroupStatus,
}
