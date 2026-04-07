use go_game::{model::*, sgf::SgfParser};

fn main() {
    let mut record = GoGameRecord {
        info: GameInfo {
            board_size: 19,
            komi: 6.5,
            ..Default::default()
        },
        tree: GameTree::new(NodeProperties::default()),
        current_path: vec![0],
    };

    // 添加分支
    let b1 = record.tree.add_child(
        0,
        Move {
            color: Color::Black,
            point: Some(Point { x: 3, y: 3 }),
        },
        NodeProperties::default(),
    );
    let w1 = record.tree.add_child(
        b1,
        Move {
            color: Color::White,
            point: Some(Point { x: 15, y: 15 }),
        },
        NodeProperties::default(),
    );

    // 切换变招
    let b2 = record.tree.add_child(
        0,
        Move {
            color: Color::Black,
            point: Some(Point { x: 4, y: 4 }),
        },
        NodeProperties {
            comment: "变招1".into(),
            ..Default::default()
        },
    );

    record.move_to_child(b2);
    assert_eq!(record.current_node().unwrap().props.comment, "变招1");

    let sgf = r#"
(;GM[1]FF[4]SZ[19]KM[6.5]PB[AlphaGo]PW[Lee Sedol]RE[B+R]
;B[pd];W[dp];B[pp];W[dd]
  (;B[fq];W[cn])
  (;B[cq];W[dq])
)
"#;

    match SgfParser::new(sgf).parse() {
        Ok(record) => {
            println!(
                "🏁 {} vs {}",
                record.info.black_name, record.info.white_name
            );
            println!(
                "📐 {}x{}, Komi: {}",
                record.info.board_size, record.info.board_size, record.info.komi
            );
            println!("🌲 节点总数: {}", record.tree.nodes.len());
            println!("📍 当前路径: {:?}", record.current_path);
        }
        Err(e) => eprintln!("❌ SGF 解析失败: {}", e),
    }

}
