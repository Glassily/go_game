/// 劫的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KoType {
    /// 简单劫（单子提单子）
    Simple,
    /// 缓气劫
    ApproachKo,
    /// 多子劫
    MultiKo,
    /// 超级劫（任何重复局面禁止）
    SuperKo,
    /// 长生等特殊循环
    EternalLife,
}