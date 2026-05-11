use std::fmt;

/// SGF FF4 标准属性枚举（按功能分组）。
/// 未知属性使用 `Other(String)` 存储以保持可扩展性。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Property {
    // === 游戏设置 ===
    /// GM: Game type（游戏类型）。1 表示围棋，其他值可用于不同棋类。必需属性。
    GM,
    /// FF: File format（文件格式版本）。例如 4 表示 FF[4]，当前主流版本。必需属性。
    FF,
    /// SZ: Size of board（棋盘尺寸）。正整数值，如 19 表示 19x19 棋盘。默认通常为 19。
    SZ,
    /// KM: Komi（贴目/贴子）。浮点数，可由白方获得的补偿分数，如 6.5。
    KM,
    /// HA: Handicap（让子数）。大于等于 0 的整数。0 表示分先，2 表示让 2 子，依此类推。
    HA,
    /// RU: Rules（对局规则）。字符串，如 "Japanese"、"Chinese"、"AGA" 等。
    RU,
    
    // === 玩家信息 ===
    /// PB: Player Black（黑方棋手姓名）。字符串。
    PB,
    /// PW: Player White（白方棋手姓名）。字符串。
    PW,
    /// BR: Black Rank（黑方段位/等级）。字符串，如 "9p"、"1d"。
    BR,
    /// WR: White Rank（白方段位/等级）。字符串，如 "5d"。
    WR,
    /// BT: Black Team（黑方所属队伍/组织）。字符串。
    BT,
    /// WT: White Team（白方所属队伍/组织）。字符串。
    WT,
    
    // === 对局信息 ===
    /// DT: Date（对局日期）。字符串，格式常为 "YYYY-MM-DD"。
    DT,
    /// EV: Event（赛事名称）。字符串，如 "第25届富士通杯"。
    EV,
    /// RO: Round（轮次/回合）。字符串，如 "半决赛" 或 "Round 3"。
    RO,
    /// PC: Place（对局地点）。字符串，如地点名称或城市。
    PC,
    /// RE: Result（对局结果）。字符串，如 "B+R"（黑中盘胜）、"W+3.5"（白胜3.5目）。
    RE,
    /// GN: Game Name（对局名称/棋谱标题）。字符串。
    GN,
    /// US: User（录入者或上传者）。字符串，表示创建或编辑该 SGF 文件的用户。
    US,
    
    // === 棋步相关 ===
    /// B: Black move（黑方落子）。坐标字符串（如 "pd"）或空串表示放弃一手（pass）。
    B,
    /// W: White move（白方落子）。坐标字符串或空串（pass）。
    W,
    /// TB: Territory Black（黑方领地标记）。常用于显示软件中的黑方控制区域或死子标记。
    TB,
    /// TW: Territory White（白方领地标记）。同 TB，用于白方。
    TW,
    
    // === 摆子/让子 ===
    /// AB: Add Black（添加黑子）。坐标列表，用于在初始棋盘上放置黑子（如让子、习题局面）。
    AB,
    /// AE: Add Empty（清空交叉点）。坐标列表，用于指定棋盘上必须为空的点。
    AE,
    /// AW: Add White（添加白子）。坐标列表，用于在初始棋盘上放置白子。
    AW,
    /// PL: Player to play（当前轮到谁下）。值为 "B" 或 "W"，指定下一步由哪方走。
    PL,
    
    // === 注释与标记 ===
    /// C: Comment（注释）。节点的文字说明，可包含多行文本。
    C,
    /// DM: Double move / Even position（双先/均势标记）。通常表示局部两分或双方先后手等局面评价。
    DM,
    /// GW: Good for White（白方有利标记）。形势判断中表示白方优势的图形标记。
    GW,
    /// GB: Good for Black（黑方有利标记）。形势判断中表示黑方优势的图形标记。
    GB,
    /// GC: Good for Both / Good for Colored?（双方有利/形势评价）。可能用于表示两分或特殊评价，具体取决于软件。
    GC,
    /// HO: Hotspot（热点标记）。对局中需要注意的关键位置提示。
    HO,
    /// UC: Unclear position（形势不明标记）。表示该局面下局势复杂、难以判断。
    UC,
    /// BM: Bad move（恶手/问题手标记）。通常带有数值 1（恶手）或 2（疑问手）。
    BM,
    /// TE: Tesuji（手筋/妙手标记）。数值通常为 1 表示普通妙手，2 表示极佳妙手。
    TE,
    /// DO: Doubtful move（疑问手标记）。表示值得商榷的着手。
    DO,
    /// IT: Interesting move（有趣的一手标记）。表示值得注意的、新颖或有趣的着手。
    IT,
    
    // === 视觉标记 ===
    /// LB: Label（文本标签）。在棋盘上为指定交叉点添加字母、数字等文本标签，如 "A"、"1"。
    LB,
    /// SL: Selected（选中/高亮棋子）。标记被选中的棋子或交叉点，常用于习题或分析。
    SL,
    /// CR: Circle（圆形标记）。在棋盘交叉点上绘制圆圈。
    CR,
    /// SQ: Square（方形标记）。在棋盘交叉点上绘制方形。
    SQ,
    /// TR: Triangle（三角形标记）。在棋盘交叉点上绘制三角形。
    TR,
    /// MA: Mark / Cross（叉号标记）。在棋盘交叉点上绘制 "×" 形标记。
    MA,
    /// LN: Line（连线）。连接两个或多个交叉点，绘制直线或折线。
    LN,
    /// AR: Arrow（箭头）。绘制从一点指向另一点的箭头，用于表示发展方向或攻击路线。
    AR,
    
    // === 变体与分支 ===
    /// V: Variation（变体/分支）。在当前位置提供一种变化图，可递归嵌套。
    V,
    /// N: Node name（节点名称）。给当前节点命名，便于引用或在导航中显示。
    N,
    /// MN: Move number（设置手数）。用于在节点上强制显示特定的手数，常用于分支或编辑局面。
    MN,
    
    // === 时间与计时 ===
    /// TM: Time limit（基本时限）。基本用时，单位为秒。
    TM,
    /// BL: Black time left（黑方剩余时间）。单位为秒。
    BL,
    /// WL: White time left（白方剩余时间）。单位为秒。
    WL,
    /// OB: Overtime Black（黑方加时/读秒次数？）。读秒阶段黑方剩余的读秒次数或其他加时参数。
    OB,
    /// OW: Overtime White（白方加时/读秒次数？）。对应白方。
    OW,
    /// OT: Overtime type（加时/读秒类型）。字符串，如 "simple"（简单读秒）、"Canadian"（加拿大读秒）等。
    OT,
    /// ON: Overtime parameters（加时参数）。具体数值，如读秒步数或每步时限，根据 OT 类型不同含义各异。
    ON,
    
    // === 其他 ===
    /// AN: Annotation（注解者/来源注释）。提供棋谱注解的作者或分析者姓名。
    AN,
    /// AP: Application（生成软件）。标识创建或编辑该 SGF 的应用程序名称和版本，如 "Sabaki:0.52.0"。
    AP,
    /// CP: Copyright（版权信息）。字符串，声明棋谱版权。
    CP,
    /// SO: Source（来源）。棋谱的原始出处或提供者。
    SO,
    /// FG: Figure（图形/图例）。可能用于表示分谱、图示编号或打印布局控制。
    FG,
    /// PM: Print mode（打印模式）。控制棋谱打印输出样式，具体实现依赖软件。
    PM,
    /// VW: View（视角/可视区域）。指定棋盘显示的区域限制，常用于局部变化图。
    VW,
    /// DD: Double digit（两位数坐标）？非标准属性，可能用于表示坐标采用两位数格式。
    DD,
    /// ST: Style（样式）。控制显示样式的参数，如棋盘皮肤、线条粗细等，扩展属性。
    ST,
    /// SU: Setup（局面设置标记）。指示当前节点包含初始局面设置（AB/AW/AE），方便工具识别。
    SU,
    /// IY: Invert Y（翻转 Y 轴）。若值为真，则坐标 Y 轴方向取反，用于兼容不同坐标系习惯。
    IY,
    /// KO: Ko（劫）。标记劫争状态或禁止提劫的约束，具体实现因软件而异。
    KO,
    /// IP: Initial position（初始位置）。可能是对局起始局面参考，或者与隐藏/显示相关。
    IP,
    /// AS: ASCII display（文本棋盘显示）？扩展属性，可能用于纯文本棋盘表示的样式定义。
    AS,
    /// SE: Setup empty? / Special effect（特殊效果）？非标准，可能用于标记空点或特殊显示效果。
    SE,

    /// 非标准或未知属性名（用于兼容未来扩展或私有属性）
    Other(String),
}

impl Property {
    /// 从字符串解析属性名（大写）
    /// 已知属性返回对应枚举，未知属性以 `Other(String)` 返回，保持解析兼容性。
    pub fn from_str(s: &str) -> Self {
        use Property::*;
        match s {
            "GM" => GM, "FF" => FF, "SZ" => SZ, "KM" => KM,
            "HA" => HA, "RU" => RU, "PB" => PB, "PW" => PW,
            "BR" => BR, "WR" => WR, "BT" => BT, "WT" => WT,
            "DT" => DT, "EV" => EV, "RO" => RO, "PC" => PC,
            "RE" => RE, "GN" => GN, "US" => US, "B"  => B,
            "W"  => W,  "TB" => TB, "TW" => TW, "AB" => AB,
            "AE" => AE, "AW" => AW, "PL" => PL, "C"  => C,
            "DM" => DM, "GW" => GW, "GB" => GB, "GC" => GC,
            "HO" => HO, "UC" => UC, "BM" => BM, "TE" => TE,
            "DO" => DO, "IT" => IT, "LB" => LB, "SL" => SL,
            "CR" => CR, "SQ" => SQ, "TR" => TR, "MA" => MA,
            "LN" => LN, "AR" => AR, "V"  => V,  "N"  => N,
            "MN" => MN, "TM" => TM, "BL" => BL, "WL" => WL,
            "OB" => OB, "OW" => OW, "OT" => OT, "ON" => ON,
            "AN" => AN, "AP" => AP, "CP" => CP, "SO" => SO,
            "FG" => FG, "PM" => PM, "VW" => VW, "DD" => DD,
            "ST" => ST, "SU" => SU, "IY" => IY, "KO" => KO,
            "IP" => IP, "AS" => AS, "SE" => SE,
            _ => Other(s.to_string()),
        }
    }

    /// 返回属性名。
    pub fn name(&self) -> &str {
        use Property::*;
        match self {
            GM => "GM", FF => "FF", SZ => "SZ", KM => "KM",
            HA => "HA", RU => "RU", PB => "PB", PW => "PW",
            BR => "BR", WR => "WR", BT => "BT", WT => "WT",
            DT => "DT", EV => "EV", RO => "RO", PC => "PC",
            RE => "RE", GN => "GN", US => "US", B  => "B",
            W  => "W",  TB => "TB", TW => "TW", AB => "AB",
            AE => "AE", AW => "AW", PL => "PL", C  => "C",
            DM => "DM", GW => "GW", GB => "GB", GC => "GC",
            HO => "HO", UC => "UC", BM => "BM", TE => "TE",
            DO => "DO", IT => "IT", LB => "LB", SL => "SL",
            CR => "CR", SQ => "SQ", TR => "TR", MA => "MA",
            LN => "LN", AR => "AR", V  => "V",  N  => "N",
            MN => "MN", TM => "TM", BL => "BL", WL => "WL",
            OB => "OB", OW => "OW", OT => "OT", ON => "ON",
            AN => "AN", AP => "AP", CP => "CP", SO => "SO",
            FG => "FG", PM => "PM", VW => "VW", DD => "DD",
            ST => "ST", SU => "SU", IY => "IY", KO => "KO",
            IP => "IP", AS => "AS", SE => "SE",
            Other(s) => s.as_str(),
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
