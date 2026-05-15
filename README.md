# Go Game - 围棋SGF编辑器

一个使用Rust编写的围棋棋谱编辑器，支持SGF格式的解析、导出和验证。

## 功能特性

- **棋盘绘制** - 支持2-25路围棋棋盘显示
- **SGF解析** - 完整支持SGF FF4格式
- **SGF导出** - 将棋谱导出为标准SGF格式
- **SGF验证** - 验证棋谱格式合法性（支持严格模式）
- **图形界面** - 基于egui的现代化界面
- **棋谱浏览** - 支持前进、后退、首步、末步导航
- **编辑功能** - 支持编辑模式自由落子
- **信息编辑** - 可编辑棋局基本信息（黑方、白方、日期、贴目、结果）
- **主题切换** - 支持明暗主题切换

## 项目结构

```
src/
├── main.rs          # 程序入口
├── gui.rs           # 图形界面
├── lib.rs           # 库入口与公共API
├── record.rs        # 棋谱记录管理
├── board/           # 棋盘模块
│   ├── board.rs     # 棋盘逻辑与合法性检查
│   └── mod.rs
├── model/           # 数据模型
│   ├── color.rs     # 棋子颜色
│   ├── move.rs      # 落子
│   ├── point.rs     # 点位坐标
│   └── mod.rs
└── sgf/             # SGF格式处理
    ├── parser.rs    # 解析器
    ├── exporter.rs  # 导出器
    ├── validator.rs # 验证器
    ├── property.rs  # 属性处理
    ├── tree.rs      # 棋谱树结构
    └── mod.rs
```

## 依赖

- `egui` / `eframe` - 图形界面框架
- `chardetng` - 字符编码检测
- `encoding_rs` - 字符编码转换
- `rfd` - 原生文件对话框

## 快速开始

### 运行GUI编辑器

```bash
cargo run
```

### 作为库使用

```rust
use go_game::{parse, export, validate, Board, Color, Move, Point};

// 解析SGF
let sgf = "(;FF[4]SZ[19]KM[6.5];B[pd];W[dd])";
let tree = parse(sgf).unwrap();

// 导出SGF
let exported = export(&tree);

// 验证棋谱
let result = validate(&tree);
assert!(result.is_valid());

// 操作棋盘
let mut board = Board::new(19);
board.set(Point::new(3, 3), Color::Black);
```

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| ← | 上一步 |
| → | 下一步 |
| Home | 跳至首步 |
| End | 跳至末步 |
| Ctrl+Z | 撤销 |
| Ctrl+Y / Ctrl+Shift+Z | 重做 |

## License

MIT