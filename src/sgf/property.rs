use std::fmt;
use std::borrow::Cow;

/// SGF FF4 标准属性枚举（按功能分组）。
/// 未知属性使用 `Other(String)` 存储以保持可扩展性。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Property {
    // === 游戏设置 ===
    GM, FF, SZ, KM, HA, RU,
    
    // === 玩家信息 ===
    PB, PW, BR, WR, BT, WT,
    
    // === 对局信息 ===
    DT, EV, RO, PC, RE, GN, US,
    
    // === 棋步相关 ===
    B, W, TB, TW,
    
    // === 摆子/让子 ===
    AB, AE, AW, PL,
    
    // === 注释与标记 ===
    C, DM, GW, GB, GC, HO, UC, BM, TE, DO, IT,
    
    // === 视觉标记 ===
    LB, SL, CR, SQ, TR, MA, LN, AR,
    
    // === 变体与分支 ===
    V, N, MN,
    
    // === 时间与计时 ===
    TM, BL, WL, OB, OW, OT, ON,
    
    // === 其他 ===
    AN, AP, CP, SO, FG, PM, VW, DD, ST, SU, IY, KO, IP, AS, SE,

    /// 非标准或未知属性名
    Other(String),
}

impl Property {
    /// 从字符串解析属性名（大写）
    /// 已知属性返回对应枚举，未知属性以 `Other(String)` 返回，保持解析兼容性。
    pub fn from_str(s: &str) -> Option<Self> {
        use Property::*;
        match s {
            "GM" => Some(GM), "FF" => Some(FF), "SZ" => Some(SZ), "KM" => Some(KM),
            "HA" => Some(HA), "RU" => Some(RU), "PB" => Some(PB), "PW" => Some(PW),
            "BR" => Some(BR), "WR" => Some(WR), "BT" => Some(BT), "WT" => Some(WT),
            "DT" => Some(DT), "EV" => Some(EV), "RO" => Some(RO), "PC" => Some(PC),
            "RE" => Some(RE), "GN" => Some(GN), "US" => Some(US), "B" => Some(B),
            "W" => Some(W), "TB" => Some(TB), "TW" => Some(TW), "AB" => Some(AB),
            "AE" => Some(AE), "AW" => Some(AW), "PL" => Some(PL), "C" => Some(C),
            "DM" => Some(DM), "GW" => Some(GW), "GB" => Some(GB), "GC" => Some(GC),
            "HO" => Some(HO), "UC" => Some(UC), "BM" => Some(BM), "TE" => Some(TE),
            "DO" => Some(DO), "IT" => Some(IT), "LB" => Some(LB), "SL" => Some(SL),
            "CR" => Some(CR), "SQ" => Some(SQ), "TR" => Some(TR), "MA" => Some(MA),
            "LN" => Some(LN), "AR" => Some(AR), "V" => Some(V), "N" => Some(N),
            "MN" => Some(MN), "TM" => Some(TM), "BL" => Some(BL), "WL" => Some(WL),
            "OB" => Some(OB), "OW" => Some(OW), "OT" => Some(OT), "ON" => Some(ON),
            "AN" => Some(AN), "AP" => Some(AP), "CP" => Some(CP), "SO" => Some(SO),
            "FG" => Some(FG), "PM" => Some(PM), "VW" => Some(VW), "DD" => Some(DD),
            "ST" => Some(ST), "SU" => Some(SU), "IY" => Some(IY), "KO" => Some(KO),
            "IP" => Some(IP), "AS" => Some(AS), "SE" => Some(SE),
            _ => Some(Other(s.to_string())),
        }
    }

    /// 返回属性名。如果是已知属性，返回静态字符串；否则返回一个动态字符串。
    pub fn name(&self) -> Cow<'_, str> {
        use Property::*;
        match self {
            GM => Cow::Borrowed("GM"), FF => Cow::Borrowed("FF"), SZ => Cow::Borrowed("SZ"), KM => Cow::Borrowed("KM"), HA => Cow::Borrowed("HA"), RU => Cow::Borrowed("RU"),
            PB => Cow::Borrowed("PB"), PW => Cow::Borrowed("PW"), BR => Cow::Borrowed("BR"), WR => Cow::Borrowed("WR"), BT => Cow::Borrowed("BT"), WT => Cow::Borrowed("WT"),
            DT => Cow::Borrowed("DT"), EV => Cow::Borrowed("EV"), RO => Cow::Borrowed("RO"), PC => Cow::Borrowed("PC"), RE => Cow::Borrowed("RE"), GN => Cow::Borrowed("GN"),
            US => Cow::Borrowed("US"), B => Cow::Borrowed("B"), W => Cow::Borrowed("W"), TB => Cow::Borrowed("TB"), TW => Cow::Borrowed("TW"), AB => Cow::Borrowed("AB"),
            AE => Cow::Borrowed("AE"), AW => Cow::Borrowed("AW"), PL => Cow::Borrowed("PL"), C => Cow::Borrowed("C"), DM => Cow::Borrowed("DM"), GW => Cow::Borrowed("GW"),
            GB => Cow::Borrowed("GB"), GC => Cow::Borrowed("GC"), HO => Cow::Borrowed("HO"), UC => Cow::Borrowed("UC"), BM => Cow::Borrowed("BM"), TE => Cow::Borrowed("TE"),
            DO => Cow::Borrowed("DO"), IT => Cow::Borrowed("IT"), LB => Cow::Borrowed("LB"), SL => Cow::Borrowed("SL"), CR => Cow::Borrowed("CR"), SQ => Cow::Borrowed("SQ"),
            TR => Cow::Borrowed("TR"), MA => Cow::Borrowed("MA"), LN => Cow::Borrowed("LN"), AR => Cow::Borrowed("AR"), V => Cow::Borrowed("V"), N => Cow::Borrowed("N"),
            MN => Cow::Borrowed("MN"), TM => Cow::Borrowed("TM"), BL => Cow::Borrowed("BL"), WL => Cow::Borrowed("WL"), OB => Cow::Borrowed("OB"), OW => Cow::Borrowed("OW"),
            OT => Cow::Borrowed("OT"), ON => Cow::Borrowed("ON"), AN => Cow::Borrowed("AN"), AP => Cow::Borrowed("AP"), CP => Cow::Borrowed("CP"), SO => Cow::Borrowed("SO"),
            FG => Cow::Borrowed("FG"), PM => Cow::Borrowed("PM"), VW => Cow::Borrowed("VW"), DD => Cow::Borrowed("DD"), ST => Cow::Borrowed("ST"), SU => Cow::Borrowed("SU"),
            IY => Cow::Borrowed("IY"), KO => Cow::Borrowed("KO"), IP => Cow::Borrowed("IP"), AS => Cow::Borrowed("AS"), SE => Cow::Borrowed("SE"),
            Other(s) => Cow::Owned(s.clone()),
        }
    }
    
    /// 判断是否为"着手"属性（产生棋步）
    pub fn is_move(self) -> bool {
        matches!(self, Property::B | Property::W)
    }
    
    /// 判断是否为"摆子"属性
    pub fn is_setup(self) -> bool {
        matches!(self, Property::AB | Property::AW | Property::AE)
    }
    
    /// 判断是否为"坐标"属性（值包含棋盘坐标）
    pub fn has_coord(self) -> bool {
        matches!(self, 
            Property::B | Property::W | 
            Property::AB | Property::AW | Property::AE |
            Property::LB | Property::SL | Property::CR | 
            Property::SQ | Property::TR | Property::MA |
            Property::AR | Property::LN
        )
    }

    /// 是否为已知（标准枚举）属性
    pub fn is_known(&self) -> bool {
        !matches!(self, Property::Other(_))
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Property::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{}", self.name()),
        }
    }
}
