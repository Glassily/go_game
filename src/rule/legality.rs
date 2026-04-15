use crate::model::*;

/// 劫规则（不同规则体系）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KoRule {
    /// 简单劫：仅禁止立即回提（日韩规则，默认）
    Simple,
    /// 位置超级劫：任何重复局面禁止（中国规则）          
    PositionalSuperKo,
    /// 情境超级劫：局面+轮次重复禁止（应氏规则）
    SituationalSuperKo,
    /// 无劫规则（允许立即回提）
    NoKo,
}

/// 合法性规则，默认不允许自杀，简单劫规则
///
/// 检查落子合法性
pub trait LegalityRule {
    /// 是否允许自杀，默认不允许自杀
    fn allow_suicide(&self) -> bool {
        false
    }

    /// 劫规则类型，默认简单劫规则
    fn ko_rule(&self) -> KoRule {
        KoRule::Simple
    }

    /// 检查落子是否合法（包括劫规则）
    ///
    /// 默认实现简单劫
    fn is_legal(
        &self,
        board: &Board,
        mv: &Move,
        last_ko_point: Option<Point>,
        // 历史局面列表用于超级劫检测
        _history: Option<&[Board]>,
    ) -> bool {
        if !board.is_legal(mv, last_ko_point, self.allow_suicide()) {
            return false;
        }
        true
    }
}
