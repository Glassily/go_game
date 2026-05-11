use eframe::egui;
use egui::{Color32, Stroke, Vec2};
use std::collections::HashMap;
use std::fs;

use go_game::board::Board;
use go_game::model::{Color, Move, Point};
use go_game::sgf::{GameTree, Property, export, parse};

pub struct GoApp {
    tree: GameTree,
    current: Option<usize>,
    board: Board,
    edit_mode: bool,
    show_tree: bool,
    show_coords: bool,
    dark_theme: bool,
    // history for undo/redo
    history: Vec<GameTree>,
    future: Vec<GameTree>,
    // capture counts
    black_captures: usize,
    white_captures: usize,
    // comment editing
    comment_edit: String,
    show_comment_panel: bool,
    // context menu state
    context_node: Option<usize>,
    context_pos: egui::Pos2,
    show_context_window: bool,
    // game info
    info_black: String,
    info_white: String,
    info_date: String,
    info_komi: String,
    info_result: String,
    show_info_window: bool,
    // parse / io error display
    show_error_window: bool,
    error_message: String,
}

impl GoApp {
    pub fn new() -> Self {
        let board = Board::new(19);
        Self {
            tree: GameTree::new(),
            current: None,
            board,
            edit_mode: true,
            show_tree: true,
            show_coords: true,
            dark_theme: false,
            history: Vec::new(),
            future: Vec::new(),
            black_captures: 0,
            white_captures: 0,
            comment_edit: String::new(),
            show_comment_panel: true,
            context_node: None,
            context_pos: egui::pos2(0.0, 0.0),
            show_context_window: false,
            info_black: String::new(),
            info_white: String::new(),
            info_date: String::new(),
            info_komi: String::from("6.5"),
            info_result: String::new(),
            show_info_window: false,
            show_error_window: false,
            error_message: String::new(),
        }
    }

    fn go_prev(&mut self) {
        if let Some(c) = self.current {
            self.current = self.tree.get_parent(c);
            self.rebuild_board_to(self.current);
        }
    }

    fn go_next(&mut self) {
        if let Some(c) = self.current {
            let ch = self.tree.get_children(c);
            if !ch.is_empty() {
                self.current = Some(ch[0]);
                self.rebuild_board_to(self.current);
            }
        } else if let Some(r) = self.tree.get_root() {
            self.current = Some(r);
            self.rebuild_board_to(self.current);
        }
    }

    fn go_first(&mut self) {
        self.current = self.tree.get_root();
        self.rebuild_board_to(self.current);
    }

    fn go_last(&mut self) {
        let mut cur = self.tree.get_root();
        while let Some(c) = cur {
            let ch = self.tree.get_children(c);
            if ch.is_empty() {
                break;
            }
            cur = Some(ch[0]);
        }
        self.current = cur;
        self.rebuild_board_to(self.current);
    }

    fn mainline(&self) -> Vec<usize> {
        let mut res = Vec::new();
        let mut cur = self.tree.get_root();
        while let Some(c) = cur {
            res.push(c);
            let ch = self.tree.get_children(c);
            if ch.is_empty() {
                break;
            }
            cur = Some(ch[0]);
        }
        res
    }

    fn rebuild_board_to(&mut self, idx: Option<usize>) {
        // clear board and replay moves from root to idx
        let size = self.board.size;
        self.board = Board::new(size);
        self.black_captures = 0;
        self.white_captures = 0;
        if idx.is_none() {
            return;
        }
        let mut path = Vec::new();
        let mut cur = idx;
        while let Some(i) = cur {
            path.push(i);
            cur = self.tree.get_parent(i);
        }
        path.reverse();
        for &i in &path {
            if let Some(node) = self.tree.get_node(i) {
                if let Some(v) = node.get(Property::B) {
                    if let Some(s) = v.first() {
                        let mv = property_str_to_move(s, Color::Black, size);
                        if let Some(m) = mv {
                            let (captured, _ko) = self.board.apply_move_uncheck(&m);
                            self.black_captures += captured.len();
                        }
                    }
                }
                if let Some(v) = node.get(Property::W) {
                    if let Some(s) = v.first() {
                        let mv = property_str_to_move(s, Color::White, size);
                        if let Some(m) = mv {
                            let (captured, _ko) = self.board.apply_move_uncheck(&m);
                            self.white_captures += captured.len();
                        }
                    }
                }
            }
        }
    }

    fn push_snapshot(&mut self) {
        self.history.push(self.tree.clone());
        self.future.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.future.push(self.tree.clone());
            self.tree = prev;
            // try to set current to root if unavailable
            self.current = self.tree.get_root();
            self.rebuild_board_to(self.current);
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.future.pop() {
            self.history.push(self.tree.clone());
            self.tree = next;
            self.current = self.tree.get_root();
            self.rebuild_board_to(self.current);
        }
    }
}

fn property_str_to_move(s: &str, color: Color, board_size: u8) -> Option<Move> {
    if s.is_empty() {
        Some(Move::pass(color))
    } else {
        Point::from_sgf(s, board_size).map(|pt| Move::new(color, pt))
    }
}

impl eframe::App for GoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // global keyboard shortcuts (use input reader API)
        ctx.input(|input| {
            if input.key_pressed(egui::Key::ArrowLeft) {
                self.go_prev();
            }
            if input.key_pressed(egui::Key::ArrowRight) {
                self.go_next();
            }
            if input.key_pressed(egui::Key::Home) {
                self.go_first();
            }
            if input.key_pressed(egui::Key::End) {
                self.go_last();
            }
            if input.modifiers.ctrl && input.key_pressed(egui::Key::Z) {
                if input.modifiers.shift {
                    self.redo();
                } else {
                    self.undo();
                }
            }
        });
        // top menu + toolbar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.push_snapshot();
                        self.tree = GameTree::new();
                        self.current = None;
                        self.rebuild_board_to(None);
                        ui.close_menu();
                    }
                    if ui.button("Open SGF").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            match fs::read_to_string(&path) {
                                Ok(s) => match parse(&s) {
                                    Ok(t) => {
                                        self.push_snapshot();
                                        self.tree = t;
                                        self.current = self.tree.get_root();
                                        self.rebuild_board_to(self.current);
                                    }
                                    Err(e) => {
                                        self.show_error_window = true;
                                        self.error_message = format!("SGF parse error: {}", e);
                                    }
                                },
                                Err(e) => {
                                    self.show_error_window = true;
                                    self.error_message = format!("Failed to read file: {}", e);
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save As").clicked() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            let s = export(&self.tree);
                            let _ = fs::write(path, s);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo (Ctrl+Z)").clicked() {
                        self.undo();
                        ui.close_menu();
                    }
                    if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                        self.redo();
                        ui.close_menu();
                    }
                    if ui.button("Toggle Edit Mode").clicked() {
                        self.edit_mode = !self.edit_mode;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.dark_theme, "Dark theme").clicked() {
                        if self.dark_theme {
                            ctx.set_visuals(egui::Visuals::dark());
                        } else {
                            ctx.set_visuals(egui::Visuals::light());
                        }
                    }
                    ui.checkbox(&mut self.show_tree, "Show game tree");
                    ui.checkbox(&mut self.show_coords, "Show coordinates");
                    ui.checkbox(&mut self.show_comment_panel, "Show comment panel");
                });

                ui.menu_button("Help", |ui| {
                    ui.label("Go SGF Editor - minimal demo");
                });
            });

            ui.horizontal(|ui| {
                if ui.button("|<").clicked() {
                    self.go_first();
                }
                if ui.button("<").clicked() {
                    self.go_prev();
                }
                if ui.button(">").clicked() {
                    self.go_next();
                }
                if ui.button("|>").clicked() {
                    self.go_last();
                }
                ui.label("Move:");
                let mut cur_idx = 0usize;
                if let Some(c) = self.current {
                    cur_idx = c;
                }
                ui.label(format!("Current: {}", cur_idx));
            });
        });

        // 右侧树面板（固定占比宽度，不可调整），在 CentralPanel 之外定义以保证棋盘计算可用区域不被挤占
        if self.show_tree {
            let win_w = ctx.available_rect().width();
            println!("Available width: {}", win_w);
            let panel_w = (win_w * 0.28).clamp(200.0, 420.0);
            println!("Tree panel width: {}", panel_w);
            egui::SidePanel::right("right_panel")
                .resizable(false)
                .default_width(panel_w)
                .show(ctx, |ui| {
                    ui.label("Game Tree");

                    // collect node data: (index, kind, comment, depth)
                    // kind: 0 none, 1 black, 2 white
                    let node_views: Vec<(usize, u8, Option<String>, usize)> = self
                        .tree
                        .nodes
                        .iter()
                        .enumerate()
                        .filter_map(|(i, node)| {
                            if node.deleted {
                                return None;
                            }
                            let depth = node_depth(&self.tree, i);
                            let mut kind: u8 = 0;
                            if node.contains(Property::B) {
                                kind = 1;
                            }
                            if node.contains(Property::W) {
                                kind = 2;
                            }
                            let comment = node.get(Property::C).and_then(|v| v.first().cloned());
                            Some((i, kind, comment, depth))
                        })
                        .collect();

                    // compute min width so deep trees produce horizontal scrollbar
                    let max_depth = node_views.iter().map(|t| t.3).max().unwrap_or(0) as f32;
                    let indent = 18.0;
                    let required_w = (max_depth + 2.0) * indent + 160.0;

                    egui::ScrollArea::both().show_viewport(ui, |ui, _viewport| {
                        ui.set_min_width(required_w);

                        for (i, kind, comment, depth) in &node_views {
                            ui.horizontal(|ui| {
                                ui.add_space((*depth as f32 * indent) as f32);
                                let dot_size = 12.0;
                                let (rect, resp) = ui.allocate_exact_size(
                                    Vec2::new(dot_size + 8.0, dot_size + 8.0),
                                    egui::Sense::click(),
                                );
                                let painter = ui.painter();
                                let center = rect.center();
                                if self.current == Some(*i) {
                                    painter.rect_filled(
                                        rect.expand(4.0),
                                        4.0,
                                        Color32::from_rgb(200, 230, 255),
                                    );
                                }
                                let _shape = match kind {
                                    1 => painter.circle_filled(
                                        center,
                                        dot_size * 0.45,
                                        Color32::BLACK,
                                    ),
                                    2 => {
                                        let s = painter.circle_filled(
                                            center,
                                            dot_size * 0.45,
                                            Color32::WHITE,
                                        );
                                        painter.circle_stroke(
                                            center,
                                            dot_size * 0.45,
                                            Stroke::new(1.0, Color32::BLACK),
                                        );
                                        s
                                    }
                                    _ => painter.circle_filled(
                                        center,
                                        dot_size * 0.25,
                                        Color32::from_gray(120),
                                    ),
                                };

                                if resp.clicked() {
                                    self.current = Some(*i);
                                    self.comment_edit = comment.clone().unwrap_or_default();
                                    self.rebuild_board_to(self.current);
                                }

                                ui.add_space(6.0);
                                if let Some(c) = comment {
                                    ui.label(c.clone());
                                }
                            });
                        }

                        if self.show_comment_panel {
                            ui.separator();
                            ui.label("Comment");
                            ui.horizontal(|ui| {
                                if ui.button("Save").clicked() {
                                    if let Some(i) = self.current {
                                        self.push_snapshot();
                                        if let Some(node) = self.tree.get_node_mut(i) {
                                            node.set(Property::C, vec![self.comment_edit.clone()]);
                                        }
                                    }
                                }
                                if ui.button("Clear").clicked() {
                                    self.comment_edit.clear();
                                }
                            });
                            ui.add(
                                egui::TextEdit::multiline(&mut self.comment_edit).desired_rows(4),
                            );
                        }
                    });
                });
        }

        // central layout: board in CentralPanel
        egui::CentralPanel::default().show(ctx, |ui| {
            // allocate a centered square inside the central panel
            let avail = ui.available_rect_before_wrap();
            let board_size = avail.width().min(avail.height());
            let center = avail.center();
            let min_pos = center - Vec2::splat(board_size * 0.5);
            let board_rect = egui::Rect::from_min_size(min_pos, Vec2::splat(board_size));

            let response = ui.allocate_rect(board_rect, egui::Sense::click());
            let r = board_rect;
            draw_board(
                ui,
                r,
                &self.board,
                self.show_coords,
                self.tree.get_root(),
                self.current,
            );

            if response.clicked() {
                if self.edit_mode {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if let Some(pt) = screen_pos_to_point(r, pos, self.board.size) {
                            let next_color = next_to_move(&self.tree, self.current);
                            let _mv = Move::new(next_color, pt);
                            self.push_snapshot();
                            let mut map = HashMap::new();
                            let prop = match next_color {
                                Color::Black => Property::B,
                                Color::White => Property::W,
                            };
                            map.insert(prop, vec![pt.to_sgf()]);
                            let _ = self.tree.add_node(self.current, map);
                            self.current = Some(self.tree.nodes.len() - 1);
                            self.rebuild_board_to(self.current);
                        }
                    }
                }
            }

            if response.secondary_clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(pt) = screen_pos_to_point(r, pos, self.board.size) {
                        let mut found: Option<usize> = None;
                        for (i, node) in self.tree.nodes.iter().enumerate().rev() {
                            if node.deleted {
                                continue;
                            }
                            if let Some(v) = node.get(Property::B) {
                                if v.first().map(|s| s == &pt.to_sgf()).unwrap_or(false) {
                                    found = Some(i);
                                    break;
                                }
                            }
                            if let Some(v) = node.get(Property::W) {
                                if v.first().map(|s| s == &pt.to_sgf()).unwrap_or(false) {
                                    found = Some(i);
                                    break;
                                }
                            }
                        }
                        self.context_node = found;
                        self.context_pos = pos;
                        self.show_context_window = true;
                    }
                }
            }
        });

        // bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mainline = self.mainline();
                let total = mainline.len();
                let mut cur_move = 0usize;
                if let Some(c) = self.current {
                    let mut n = Some(c);
                    let mut cnt = 0usize;
                    while let Some(i) = n {
                        if let Some(node) = self.tree.get_node(i) {
                            if node.contains(Property::B) || node.contains(Property::W) {
                                cnt += 1;
                            }
                        }
                        n = self.tree.get_parent(i);
                    }
                    cur_move = cnt;
                }
                ui.label(format!("Move: {}/{}", cur_move, total));
                ui.separator();
                ui.label(format!(
                    "Next: {}",
                    if next_to_move(&self.tree, self.current) == Color::Black {
                        "Black"
                    } else {
                        "White"
                    }
                ));
                ui.separator();
                ui.label(format!(
                    "Captures - Black: {}  White: {}",
                    self.black_captures, self.white_captures
                ));
                ui.separator();
                ui.label(format!(
                    "Nodes: {}",
                    self.tree.nodes.iter().filter(|n| !n.deleted).count()
                ));
                ui.separator();
                ui.label(format!(
                    "Edit: {}",
                    if self.edit_mode { "ON" } else { "OFF" }
                ));
                ui.separator();
                if ui.button("Comment").clicked() {
                    self.show_comment_panel = true;
                }
                ui.separator();
                if ui.button("Game Info").clicked() {
                    self.show_info_window = true;
                }
            });
        });

        if self.show_info_window {
            egui::Window::new("Game Info").show(ctx, |ui| {
                ui.label("Black:");
                ui.text_edit_singleline(&mut self.info_black);
                ui.label("White:");
                ui.text_edit_singleline(&mut self.info_white);
                ui.label("Date:");
                ui.text_edit_singleline(&mut self.info_date);
                ui.label("Komi:");
                ui.text_edit_singleline(&mut self.info_komi);
                ui.label("Result:");
                ui.text_edit_singleline(&mut self.info_result);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if let Some(r) = self.tree.get_root() {
                            if let Some(node) = self.tree.get_node_mut(r) {
                                if !self.info_black.is_empty() {
                                    node.set(Property::PB, vec![self.info_black.clone()]);
                                }
                                if !self.info_white.is_empty() {
                                    node.set(Property::PW, vec![self.info_white.clone()]);
                                }
                                if !self.info_date.is_empty() {
                                    node.set(Property::DT, vec![self.info_date.clone()]);
                                }
                                if !self.info_komi.is_empty() {
                                    node.set(Property::KM, vec![self.info_komi.clone()]);
                                }
                                if !self.info_result.is_empty() {
                                    node.set(Property::RE, vec![self.info_result.clone()]);
                                }
                            }
                        }
                        self.show_info_window = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_info_window = false;
                    }
                });
            });
        }

        if self.show_error_window {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&self.error_message);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.show_error_window = false;
                            self.error_message.clear();
                        }
                    });
                });
        }
    }
}

fn draw_board(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    board: &Board,
    show_coords: bool,
    _root: Option<usize>,
    _current: Option<usize>,
) {
    let painter = ui.painter_at(rect);
    let size = board.size as usize;

    // 背景色（木质效果）
    painter.rect_filled(rect, 0.0, Color32::from_rgb(230, 190, 120));

    // 动态计算内边距：最大20，最小2，约为宽度的3%
    let pad = (rect.width() * 0.03).clamp(2.0, 20.0);
    let inner_rect = rect.shrink(2.0);
    let drawing_rect = egui::Rect::from_min_max(
        inner_rect.min + egui::vec2(pad, pad),
        inner_rect.max - egui::vec2(pad, pad),
    );
    let cell = drawing_rect.width() / ((size as f32 - 1.0).max(1.0));

    // 画网格线
    for i in 0..size {
        let x = drawing_rect.left() + i as f32 * cell;
        let y = drawing_rect.top() + i as f32 * cell;
        painter.line_segment(
            [
                egui::pos2(x, drawing_rect.top()),
                egui::pos2(x, drawing_rect.bottom()),
            ],
            Stroke::new(1.2, Color32::BLACK),
        );
        painter.line_segment(
            [
                egui::pos2(drawing_rect.left(), y),
                egui::pos2(drawing_rect.right(), y),
            ],
            Stroke::new(1.2, Color32::BLACK),
        );
    }

    // 星位（仅对19路或更小尺寸有效，若小于9则自动调整）
    let star_points = if size == 19 {
        vec![3, 9, 15]
    } else if size == 13 {
        vec![3, 9]
    } else if size == 9 {
        vec![2, 6]
    } else {
        vec![]
    };
    for &sx in &star_points {
        for &sy in &star_points {
            if sx < size && sy < size {
                let cx = drawing_rect.left() + sx as f32 * cell;
                let cy = drawing_rect.top() + sy as f32 * cell;
                let star_radius = cell * 0.08; // 动态星标半径
                painter.circle_filled(egui::pos2(cx, cy), star_radius, Color32::BLACK);
            }
        }
    }

    // 画棋子
    for y in 0..size {
        for x in 0..size {
            let pt = Point {
                x: x as u8,
                y: y as u8,
            };
            if let Some(col) = board.get(pt) {
                let cx = drawing_rect.left() + x as f32 * cell;
                let cy = drawing_rect.top() + y as f32 * cell;
                let radius = cell * 0.42;
                match col {
                    Color::Black => {
                        painter.circle_filled(egui::pos2(cx, cy), radius, Color32::from_gray(10));
                    }
                    Color::White => {
                        painter.circle_filled(egui::pos2(cx, cy), radius, Color32::from_gray(240));
                        painter.circle_stroke(
                            egui::pos2(cx, cy),
                            radius,
                            Stroke::new(1.0, Color32::BLACK),
                        );
                    }
                }
            }
        }
    }

    // 坐标（动态字体大小）
    if show_coords {
        let font_id = egui::FontId::proportional((cell * 0.35).max(1.0));
        // 底部字母坐标
        for x in 0..size {
            let cx = drawing_rect.left() + x as f32 * cell;
            let label = if x >= 8 {
                ((b'A' + x as u8 + 1) as char).to_string()
            } else {
                ((b'A' + x as u8) as char).to_string()
            };
            painter.text(
                egui::pos2(cx - 6.0, drawing_rect.bottom() + 6.0),
                egui::Align2::LEFT_TOP,
                label,
                font_id.clone(),
                Color32::BLACK,
            );
        }
        // 右侧数字坐标
        for y in 0..size {
            let cy = drawing_rect.top() + y as f32 * cell;
            let label = (size - y).to_string();
            painter.text(
                egui::pos2(drawing_rect.right() + 6.0, cy - 6.0),
                egui::Align2::LEFT_TOP,
                label,
                font_id.clone(),
                Color32::BLACK,
            );
        }
    }
}

fn draw_tree_panel(
    ui: &mut egui::Ui, 
    rect: egui::Rect,
    tree: &GameTree, 
    show_tree: bool,
    current: Option<usize>
) {
    


}

fn screen_pos_to_point(rect: egui::Rect, pos: egui::Pos2, size: u8) -> Option<Point> {
    // 必须与 draw_board 中的几何计算完全一致
    let pad = (rect.width() * 0.03).clamp(2.0, 20.0);
    let inner_rect = rect.shrink(2.0);
    let drawing_rect = egui::Rect::from_min_max(
        inner_rect.min + egui::vec2(pad, pad),
        inner_rect.max - egui::vec2(pad, pad),
    );
    if !drawing_rect.contains(pos) {
        return None;
    }
    let cell = drawing_rect.width() / ((size as f32 - 1.0).max(1.0));
    let fx = (pos.x - drawing_rect.left()) / cell;
    let fy = (pos.y - drawing_rect.top()) / cell;
    let xi = fx.round() as i32;
    let yi = fy.round() as i32;
    if xi >= 0 && yi >= 0 && (xi as u8) < size && (yi as u8) < size {
        Some(Point {
            x: xi as u8,
            y: yi as u8,
        })
    } else {
        None
    }
}

fn next_to_move(tree: &GameTree, cur: Option<usize>) -> Color {
    // count moves from root to cur
    let mut count = 0usize;
    if let Some(mut n) = cur {
        while let Some(p) = tree.get_parent(n) {
            if let Some(node) = tree.get_node(n) {
                if node.contains(Property::B) || node.contains(Property::W) {
                    count += 1;
                }
            }
            n = p;
        }
        // include root
        if let Some(root) = tree.get_root() {
            if let Some(node) = tree.get_node(root) {
                if node.contains(Property::B) || node.contains(Property::W) {
                    count += 1;
                }
            }
        }
    }
    if count % 2 == 0 {
        Color::Black
    } else {
        Color::White
    }
}

fn node_depth(tree: &GameTree, idx: usize) -> usize {
    let mut d = 0;
    let mut cur = Some(idx);
    while let Some(i) = cur {
        cur = tree.get_parent(i);
        if cur.is_some() {
            d += 1;
        }
    }
    d
}
