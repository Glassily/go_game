use chardetng::{self, Iso2022JpDetection};
use eframe::egui;
use egui::{Color32, Layout, Sense, Stroke, Vec2};
use std::collections::HashMap;

use go_game::Board;
use go_game::model::{Color, Move, Point};
use go_game::record::{GoRecord, NodeInfo};
use go_game::sgf::{export, parse};

fn default_komi(rules: &str) -> &'static str {
    match rules {
        "Japanese" => "6.5",
        "Chinese" => "7.5",
        "AGA" => "7.0",
        "New Zealand" => "6.5",
        _ => "6.5",
    }
}

pub struct GoGui {
    record: GoRecord,
    edit_mode: bool,
    show_tree: bool,
    show_coords: bool,
    dark_theme: bool,
    /// 是否显示下一步提示（半透明棋子）
    show_next_moves: bool,
    comment_edit: String,
    show_comment_panel: bool,
    context_node: Option<usize>,
    context_pos: egui::Pos2,
    show_context_window: bool,
    info_game_name: String,
    info_black: String,
    info_black_rank: String,
    info_white: String,
    info_white_rank: String,
    info_event: String,
    info_round: String,
    info_place: String,
    info_date: String,
    info_komi: String,
    info_result: String,
    info_rules: String,
    info_handicap: String,
    info_black_team: String,
    info_white_team: String,
    info_user: String,
    show_info_window: bool,
    show_error_window: bool,
    error_message: String,
    show_illegal_move_popup: bool,
    illegal_move_error: Option<String>,
    show_new_game_dialog: bool,
    new_game_board_size: u8,
    new_game_rules: String,
    new_game_komi: String,
    new_game_komi_edited: bool,
}

impl GoGui {
    pub fn new() -> Self {
        Self {
            record: GoRecord::default(),
            edit_mode: true,
            show_tree: true,
            show_coords: true,
            dark_theme: false,
            show_next_moves: true,
            comment_edit: String::new(),
            show_comment_panel: true,
            context_node: None,
            context_pos: egui::pos2(0.0, 0.0),
            show_context_window: false,
            info_game_name: String::new(),
            info_black: String::new(),
            info_black_rank: String::new(),
            info_white: String::new(),
            info_white_rank: String::new(),
            info_event: String::new(),
            info_round: String::new(),
            info_place: String::new(),
            info_date: String::new(),
            info_komi: String::from("6.5"),
            info_result: String::new(),
            info_rules: String::new(),
            info_handicap: String::new(),
            info_black_team: String::new(),
            info_white_team: String::new(),
            info_user: String::new(),
            show_info_window: false,
            show_error_window: false,
            error_message: String::new(),
            show_illegal_move_popup: false,
            illegal_move_error: None,
            show_new_game_dialog: false,
            new_game_board_size: 19,
            new_game_rules: String::from("Japanese"),
            new_game_komi: String::from("6.5"),
            new_game_komi_edited: false,
        }
    }
}

impl eframe::App for GoGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowLeft) {
                self.record.go_prev();
            }
            if input.key_pressed(egui::Key::ArrowRight) {
                self.record.go_next();
            }
            if input.key_pressed(egui::Key::Home) {
                self.record.go_first();
            }
            if input.key_pressed(egui::Key::End) {
                self.record.go_last();
            }
            if input.modifiers.ctrl && input.key_pressed(egui::Key::Z) {
                if input.modifiers.shift {
                    self.record.redo();
                } else {
                    self.record.undo();
                }
            }
        });
        self.top_panel(ui);
        self.status_bar(ui);
        self.central_panel(ui);
        self.info_window(ui);
        self.error_window(ui);
        self.illegal_move_popup(ui);
        self.context_menu(ui);
        self.new_game_dialog(ui);
    }
}

impl GoGui {
    /// 菜单栏
    fn top_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                self.file_menu(ui);
                self.edit_menu(ui);
                self.view_menu(ui);
                self.help_menu(ui);
            });

            ui.horizontal(|ui| {
                if ui.button("|<").clicked() {
                    self.record.go_first();
                }
                if ui.button("<").clicked() {
                    self.record.go_prev();
                }
                if ui.button(">").clicked() {
                    self.record.go_next();
                }
                if ui.button("|>").clicked() {
                    self.record.go_last();
                }
                ui.label(format!(
                    "Move: {}/{}",
                    self.record.current_move_number(),
                    self.record.total_moves()
                ));
            });
        });
    }

    fn file_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() {
                self.new_game_board_size = self.record.board_size();
                if let Some(root) = self.record.tree.get_root() {
                    if let Some(node) = self.record.tree.get_node(root) {
                        if let Some(r) = node.get_first(go_game::Property::RU) {
                            self.new_game_rules = r.clone();
                        }
                        if let Some(k) = node.get_first(go_game::Property::KM) {
                            self.new_game_komi = k.clone();
                        }
                    }
                }
                if self.new_game_rules.is_empty() {
                    self.new_game_rules = String::from("Japanese");
                }
                let dk = default_komi(&self.new_game_rules);
                self.new_game_komi_edited = self.new_game_komi != dk;
                self.show_new_game_dialog = true;
                ui.close();
            }
            if ui.button("Open SGF").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            let content = if bytes.is_empty() {
                                String::new()
                            } else {
                                decode_sgf_content(&bytes)
                            };

                            match parse(&content) {
                                Ok(tree) => {
                                    self.record.load_sgf(tree);
                                    let info = self.record.get_game_info();
                                    self.info_game_name = info.game_name.unwrap_or_default();
                                    self.info_black = info.black.unwrap_or_default();
                                    self.info_black_rank = info.black_rank.unwrap_or_default();
                                    self.info_white = info.white.unwrap_or_default();
                                    self.info_white_rank = info.white_rank.unwrap_or_default();
                                    self.info_event = info.event.unwrap_or_default();
                                    self.info_round = info.round.unwrap_or_default();
                                    self.info_place = info.place.unwrap_or_default();
                                    self.info_date = info.date.unwrap_or_default();
                                    self.info_komi = info.komi.unwrap_or_default();
                                    self.info_result = info.result.unwrap_or_default();
                                    self.info_rules = info.rules.unwrap_or_default();
                                    self.info_handicap = info.handicap.unwrap_or_default();
                                    self.info_black_team = info.black_team.unwrap_or_default();
                                    self.info_white_team = info.white_team.unwrap_or_default();
                                    self.info_user = info.user.unwrap_or_default();
                                }
                                Err(e) => {
                                    self.show_error_window = true;
                                    self.error_message = format!("SGF parse error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            self.show_error_window = true;
                            self.error_message = format!("Failed to read file: {}", e);
                        }
                    }
                }
                ui.close();
            }
            if ui.button("Save As").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("sgf file", &["sgf"])
                    .save_file()
                {
                    let s = export(&self.record.tree);
                    let _ = std::fs::write(path, s);
                }
                ui.close();
            }
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Edit", |ui| {
            if ui.button("Undo (Ctrl+Z)").clicked() {
                self.record.undo();
                ui.close();
            }
            if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                self.record.redo();
                ui.close();
            }
            if ui
                .button(format!(
                    "Toggle Edit Mode ({})",
                    if self.edit_mode { "ON" } else { "OFF" }
                ))
                .clicked()
            {
                self.edit_mode = !self.edit_mode;
                ui.close();
            }
        });
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("View", |ui| {
            if ui.checkbox(&mut self.dark_theme, "Dark theme").clicked() {
                if self.dark_theme {
                    ui.ctx().set_visuals(egui::Visuals::dark());
                } else {
                    ui.ctx().set_visuals(egui::Visuals::light());
                }
            }
            ui.checkbox(&mut self.show_tree, "Show game tree");
            ui.checkbox(&mut self.show_coords, "Show coordinates");
            ui.checkbox(&mut self.show_comment_panel, "Show comment panel");
            ui.checkbox(&mut self.show_next_moves, "Show next moves");
        });
    }

    fn help_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Help", |ui| {
            ui.label("Go SGF Editor");
        });
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let avail = ui.available_rect_before_wrap();

            if self.show_tree {
                // 计算树面板宽度：基于窗口宽度的基础宽度乘以系数，使其随窗口大小变化
                let win_w = avail.width();
                let base_panel_w = (win_w * 0.28).clamp(200.0, 420.0);
                let coef = (win_w / 1200.0).clamp(0.6, 1.2);
                let tree_w = (base_panel_w * coef).min(win_w * 0.6);
                let gap = 8.0;

                let board_area_w = (avail.width() - tree_w - gap).max(120.0);

                // 绝对矩形分配：将棋盘放左，树面板放在中央面板的最右侧（靠右布局）
                let left_rect =
                    egui::Rect::from_min_size(avail.min, Vec2::new(board_area_w, avail.height()));
                let tree_rect = egui::Rect::from_min_max(
                    egui::pos2(avail.right() - tree_w, avail.top()),
                    egui::pos2(avail.right(), avail.bottom()),
                );

                ui.scope_builder(
                    egui::UiBuilder::new()
                        .sense(Sense::click())
                        .max_rect(left_rect),
                    |ui| {
                        // 棋盘绘制
                        let avail_child = ui.available_rect_before_wrap();
                        let board_size = avail_child.width().min(avail_child.height());
                        let center = avail_child.center();
                        let min_pos = center - Vec2::splat(board_size * 0.5);
                        let board_rect =
                            egui::Rect::from_min_size(min_pos, Vec2::splat(board_size));

                        let response = ui.allocate_rect(board_rect, egui::Sense::click());
                        let board_rect = response.rect;

                        let next_moves = if self.show_next_moves {
                            self.record.get_variation_moves()
                        } else {
                            vec![]
                        };
                        draw_board(
                            ui,
                            board_rect,
                            &self.record.board,
                            self.show_coords,
                            &next_moves,
                        );

                        if response.clicked() && self.edit_mode {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if let Some(pt) =
                                    screen_pos_to_point(board_rect, pos, self.record.board_size())
                                {
                                    let next_color = self.record.next_to_move();
                                    let mv = Move::new(next_color, pt);
                                    if let Err(e) = self.record.add_move(mv) {
                                        self.show_illegal_move_popup = true;
                                        self.illegal_move_error = Some(format!("{:?}", e));
                                    }
                                }
                            }
                        }

                        if response.secondary_clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if let Some(pt) =
                                    screen_pos_to_point(board_rect, pos, self.record.board_size())
                                {
                                    self.context_node = self.record.find_move_at_point(pt);
                                    self.context_pos = pos;
                                    self.show_context_window = true;
                                }
                            }
                        }
                    },
                );

                ui.scope_builder(
                    egui::UiBuilder::new()
                        .layout(Layout::top_down(egui::Align::LEFT))
                        .sense(Sense::click())
                        .max_rect(tree_rect),
                    |ui| {
                        ui.label("Game Tree");

                        // tree_panel绘制
                        let node_views: Vec<(usize, NodeInfo)> = self.record.all_nodes();
                        let max_depth = node_views.iter().map(|t| t.1.depth).max().unwrap_or(0);

                        // 列分配
                        let mut col_map: HashMap<usize, usize> = HashMap::new();
                        let mut next_col: usize = 1;
                        fn assign_cols(
                            tree: &go_game::sgf::GameTree,
                            idx: usize,
                            col_map: &mut HashMap<usize, usize>,
                            next_col: &mut usize,
                            parent_col: usize,
                        ) {
                            if col_map.contains_key(&idx) {
                                return;
                            }
                            col_map.insert(idx, parent_col);
                            let children = tree.get_children(idx).to_vec();
                            if children.is_empty() {
                                return;
                            }
                            let mut iter = children.into_iter();
                            if let Some(first) = iter.next() {
                                assign_cols(tree, first, col_map, next_col, parent_col);
                            }
                            for c in iter {
                                let this_col = *next_col;
                                *next_col += 1;
                                assign_cols(tree, c, col_map, next_col, this_col);
                            }
                        }
                        if let Some(root) = self.record.tree.get_root() {
                            assign_cols(&self.record.tree, root, &mut col_map, &mut next_col, 0);
                        }

                        let max_col = col_map.values().copied().max().unwrap_or(0);

                        // 布局参数（更紧凑以适应子面板）
                        let row_h = 36.0;
                        let col_w = 40.0;
                        let canvas_w = (max_col as f32 + 1.0) * col_w + 20.0;
                        let canvas_h = (max_depth as f32 + 2.0) * row_h + 20.0;

                        egui::ScrollArea::both().show_viewport(ui, |ui, _viewport| {
                            // 限制最小宽度为 canvas_w，但不会超出子面板宽度
                            ui.set_min_width(canvas_w.min(tree_w - 8.0));

                            ui.allocate_space(Vec2::new(canvas_w, canvas_h));
                            let origin = ui.min_rect().min;
                            let painter = ui.painter();

                            let mut nodes_sorted = node_views.clone();
                            nodes_sorted.sort_by(|a, b| {
                                a.1.depth.cmp(&b.1.depth).then_with(|| {
                                    let ca = col_map.get(&a.0).copied().unwrap_or(0);
                                    let cb = col_map.get(&b.0).copied().unwrap_or(0);
                                    ca.cmp(&cb)
                                })
                            });

                            // 计算位置并缓存
                            let dot_size = 12.0;
                            let mut pos_map: HashMap<usize, egui::Pos2> = HashMap::new();
                            let mut rect_map: HashMap<usize, egui::Rect> = HashMap::new();
                            for (idx, info) in &nodes_sorted {
                                let col = col_map.get(idx).copied().unwrap_or(0) as f32;
                                let x = origin.x + 12.0 + col * col_w;
                                let y = origin.y + 8.0 + info.depth as f32 * row_h;
                                let node_rect = egui::Rect::from_min_size(
                                    egui::pos2(x, y),
                                    Vec2::new(dot_size + 8.0, dot_size + 8.0),
                                );
                                pos_map.insert(*idx, node_rect.center());
                                rect_map.insert(*idx, node_rect);
                            }

                            // 绘制连线
                            for (idx, _info) in &nodes_sorted {
                                if let Some(parent) = self.record.tree.get_parent(*idx) {
                                    if let (Some(&a), Some(&b)) =
                                        (pos_map.get(&parent), pos_map.get(idx))
                                    {
                                        let mid_y = (a.y + b.y) * 0.5;
                                        let p1 = egui::Pos2::new(a.x, mid_y);
                                        let p2 = egui::Pos2::new(b.x, mid_y);
                                        painter.line_segment(
                                            [a, p1],
                                            Stroke::new(1.0, Color32::from_gray(160)),
                                        );
                                        painter.line_segment(
                                            [p1, p2],
                                            Stroke::new(1.0, Color32::from_gray(160)),
                                        );
                                        painter.line_segment(
                                            [p2, b],
                                            Stroke::new(1.0, Color32::from_gray(160)),
                                        );
                                    }
                                }
                            }

                            // 绘制节点
                            for (idx, info) in &nodes_sorted {
                                let node_rect = rect_map.get(idx).copied().unwrap();
                                let resp = ui.interact(
                                    node_rect,
                                    egui::Id::new(format!("node_{}", idx)),
                                    egui::Sense::click(),
                                );
                                let center = node_rect.center();
                                if self.record.current == Some(*idx) {
                                    painter.rect_filled(
                                        node_rect.expand(4.0),
                                        4.0,
                                        Color32::from_rgb(200, 230, 255),
                                    );
                                }
                                match info.kind {
                                    1 => {
                                        painter.circle_filled(
                                            center,
                                            dot_size * 0.45,
                                            Color32::BLACK,
                                        );
                                    }
                                    2 => {
                                        painter.circle_filled(
                                            center,
                                            dot_size * 0.45,
                                            Color32::WHITE,
                                        );
                                        painter.circle_stroke(
                                            center,
                                            dot_size * 0.45,
                                            Stroke::new(1.0, Color32::BLACK),
                                        );
                                    }
                                    _ => {
                                        painter.circle_filled(
                                            center,
                                            dot_size * 0.25,
                                            Color32::from_gray(120),
                                        );
                                    }
                                }

                                if resp.clicked() {
                                    self.record.go_to(*idx);
                                    self.comment_edit = info.comment.clone().unwrap_or_default();
                                }

                                if let Some(c) = &info.comment {
                                    painter.text(
                                        egui::pos2(
                                            node_rect.right() + 6.0,
                                            node_rect.center().y - 6.0,
                                        ),
                                        egui::Align2::LEFT_TOP,
                                        c.clone(),
                                        egui::FontId::proportional(12.0),
                                        Color32::BLACK,
                                    );
                                }
                            }

                            // 评论面板
                            if self.show_comment_panel {
                                ui.separator();
                                ui.label("Comment");
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.comment_edit)
                                        .desired_rows(4),
                                );
                                ui.horizontal(|ui| {
                                    if ui.button("Save").clicked() {
                                        if let Some(i) = self.record.current {
                                            self.record.set_comment(i, self.comment_edit.clone());
                                        }
                                    }
                                    if ui.button("Clear").clicked() {
                                        self.comment_edit.clear();
                                    }
                                });
                            }
                        });
                    },
                );
            } else {
                // 无树时保持原始棋盘居中显示
                let board_size = avail.width().min(avail.height());
                let center = avail.center();
                let min_pos = center - Vec2::splat(board_size * 0.5);
                let board_rect = egui::Rect::from_min_size(min_pos, Vec2::splat(board_size));

                let response = ui.allocate_rect(board_rect, egui::Sense::click());
                let board_rect = response.rect;

                let next_moves = if self.show_next_moves {
                    self.record.get_variation_moves()
                } else {
                    vec![]
                };
                draw_board(
                    ui,
                    board_rect,
                    &self.record.board,
                    self.show_coords,
                    &next_moves,
                );

                if response.clicked() && self.edit_mode {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if let Some(pt) =
                            screen_pos_to_point(board_rect, pos, self.record.board_size())
                        {
                            let next_color = self.record.next_to_move();
                            let mv = Move::new(next_color, pt);
                            if let Err(e) = self.record.add_move(mv) {
                                self.show_illegal_move_popup = true;
                                self.illegal_move_error = Some(format!("{:?}", e));
                            }
                        }
                    }
                }

                if response.secondary_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if let Some(pt) =
                            screen_pos_to_point(board_rect, pos, self.record.board_size())
                        {
                            self.context_node = self.record.find_move_at_point(pt);
                            self.context_pos = pos;
                            self.show_context_window = true;
                        }
                    }
                }
            }
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Move: {}/{}",
                    self.record.current_move_number(),
                    self.record.total_moves()
                ));
                ui.separator();
                ui.label(format!(
                    "Next: {}",
                    if self.record.next_to_move() == Color::Black {
                        "Black"
                    } else {
                        "White"
                    }
                ));
                ui.separator();
                ui.label(format!(
                    "Captures - Black: {}  White: {}",
                    self.record.black_captures, self.record.white_captures
                ));
                ui.separator();
                ui.label(format!("Nodes: {}", self.record.node_count()));
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
                    let info = self.record.get_game_info();
                    self.info_game_name = info.game_name.unwrap_or_default();
                    self.info_black = info.black.unwrap_or_default();
                    self.info_black_rank = info.black_rank.unwrap_or_default();
                    self.info_white = info.white.unwrap_or_default();
                    self.info_white_rank = info.white_rank.unwrap_or_default();
                    self.info_event = info.event.unwrap_or_default();
                    self.info_round = info.round.unwrap_or_default();
                    self.info_place = info.place.unwrap_or_default();
                    self.info_date = info.date.unwrap_or_default();
                    self.info_komi = info.komi.unwrap_or_default();
                    self.info_result = info.result.unwrap_or_default();
                    self.info_rules = info.rules.unwrap_or_default();
                    self.info_handicap = info.handicap.unwrap_or_default();
                    self.info_black_team = info.black_team.unwrap_or_default();
                    self.info_white_team = info.white_team.unwrap_or_default();
                    self.info_user = info.user.unwrap_or_default();
                    self.show_info_window = true;
                }
            });
        });
    }

    fn info_window(&mut self, ctx: &egui::Context) {
        if !self.show_info_window {
            return;
        }

        egui::Window::new("Game Info")
            .resizable(true)
            .default_width(480.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(450.0);

                    ui.heading("Game Info");
                    ui.separator();

                    ui.group(|ui| {
                        ui.label("Title:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.info_game_name)
                                .desired_width(400.0),
                        );
                        ui.horizontal(|ui| {
                            ui.label("Event:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_event)
                                    .desired_width(200.0),
                            );
                            ui.label("Round:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_round)
                                    .desired_width(100.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Place:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_place)
                                    .desired_width(200.0),
                            );
                            ui.label("Date:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_date)
                                    .desired_width(140.0),
                            );
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Black:");
                                ui.text_edit_singleline(&mut self.info_black);
                                ui.label("Rank:");
                                ui.text_edit_singleline(&mut self.info_black_rank);
                                ui.label("Team:");
                                ui.text_edit_singleline(&mut self.info_black_team);
                            });
                            ui.add_space(20.0);
                            ui.vertical(|ui| {
                                ui.label("White:");
                                ui.text_edit_singleline(&mut self.info_white);
                                ui.label("Rank:");
                                ui.text_edit_singleline(&mut self.info_white_rank);
                                ui.label("Team:");
                                ui.text_edit_singleline(&mut self.info_white_team);
                            });
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Rules:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_rules)
                                    .desired_width(120.0),
                            );
                            ui.label("Komi:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_komi).desired_width(80.0),
                            );
                            ui.label("Handicap:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_handicap)
                                    .desired_width(60.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Result:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_result)
                                    .desired_width(200.0),
                            );
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("User:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.info_user)
                                    .desired_width(350.0),
                            );
                        });
                    });

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let info = go_game::record::GameInfo {
                                game_name: if self.info_game_name.is_empty() {
                                    None
                                } else {
                                    Some(self.info_game_name.clone())
                                },
                                black: if self.info_black.is_empty() {
                                    None
                                } else {
                                    Some(self.info_black.clone())
                                },
                                black_rank: if self.info_black_rank.is_empty() {
                                    None
                                } else {
                                    Some(self.info_black_rank.clone())
                                },
                                white: if self.info_white.is_empty() {
                                    None
                                } else {
                                    Some(self.info_white.clone())
                                },
                                white_rank: if self.info_white_rank.is_empty() {
                                    None
                                } else {
                                    Some(self.info_white_rank.clone())
                                },
                                event: if self.info_event.is_empty() {
                                    None
                                } else {
                                    Some(self.info_event.clone())
                                },
                                round: if self.info_round.is_empty() {
                                    None
                                } else {
                                    Some(self.info_round.clone())
                                },
                                place: if self.info_place.is_empty() {
                                    None
                                } else {
                                    Some(self.info_place.clone())
                                },
                                date: if self.info_date.is_empty() {
                                    None
                                } else {
                                    Some(self.info_date.clone())
                                },
                                komi: if self.info_komi.is_empty() {
                                    None
                                } else {
                                    Some(self.info_komi.clone())
                                },
                                result: if self.info_result.is_empty() {
                                    None
                                } else {
                                    Some(self.info_result.clone())
                                },
                                rules: if self.info_rules.is_empty() {
                                    None
                                } else {
                                    Some(self.info_rules.clone())
                                },
                                handicap: if self.info_handicap.is_empty() {
                                    None
                                } else {
                                    Some(self.info_handicap.clone())
                                },
                                black_team: if self.info_black_team.is_empty() {
                                    None
                                } else {
                                    Some(self.info_black_team.clone())
                                },
                                white_team: if self.info_white_team.is_empty() {
                                    None
                                } else {
                                    Some(self.info_white_team.clone())
                                },
                                user: if self.info_user.is_empty() {
                                    None
                                } else {
                                    Some(self.info_user.clone())
                                },
                            };
                            self.record.set_game_info(&info);
                            self.show_info_window = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_info_window = false;
                        }
                    });
                });
            });
    }

    fn error_window(&mut self, ctx: &egui::Context) {
        if !self.show_error_window {
            return;
        }

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

    fn illegal_move_popup(&mut self, ctx: &egui::Context) {
        if !self.show_illegal_move_popup {
            return;
        }

        egui::Window::new("Illegal Move")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let msg = self
                    .illegal_move_error
                    .as_deref()
                    .unwrap_or("Unknown error");
                ui.label(msg);
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        self.show_illegal_move_popup = false;
                        self.illegal_move_error = None;
                    }
                });
            });
    }

    fn context_menu(&mut self, ctx: &egui::Context) {
        if !self.show_context_window {
            return;
        }

        let node_idx = self.context_node;

        egui::Window::new("Move Options")
            .fixed_pos(self.context_pos)
            .show(ctx, |ui| {
                if let Some(idx) = node_idx {
                    ui.label(format!("Node: {}", idx));
                    if ui.button("Go to").clicked() {
                        self.record.go_to(idx);
                        self.context_node = None;
                        self.show_context_window = false;
                        ui.close();
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.context_node = None;
                    self.show_context_window = false;
                    ui.close();
                }
            });
    }

    fn new_game_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.show_new_game_dialog {
            return;
        }

        let mut close_dialog = false;

        egui::Window::new("New Game")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.set_width(320.0);

                ui.label("Board Size:");
                ui.horizontal(|ui| {
                    for size in &[19u8, 13, 9] {
                        if ui
                            .radio(
                                self.new_game_board_size == *size,
                                format!("{}x{}", size, size),
                            )
                            .clicked()
                        {
                            self.new_game_board_size = *size;
                        }
                    }
                });

                ui.label("Rules:");
                ui.horizontal(|ui| {
                    let rules_list = ["Japanese", "Chinese", "AGA", "New Zealand"];
                    for rule in &rules_list {
                        if ui.radio(self.new_game_rules == *rule, *rule).clicked() {
                            self.new_game_rules = rule.to_string();
                            if !self.new_game_komi_edited {
                                self.new_game_komi = default_komi(rule).to_string();
                            }
                        }
                    }
                });

                ui.label("Komi:");
                let old_komi = self.new_game_komi.clone();
                ui.text_edit_singleline(&mut self.new_game_komi);
                if self.new_game_komi != old_komi {
                    self.new_game_komi_edited = true;
                }

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        self.record = GoRecord::new(self.new_game_board_size);
                        self.record
                            .set_root_property(go_game::Property::GM, vec!["1".to_string()]);
                        self.record
                            .set_root_property(go_game::Property::FF, vec!["4".to_string()]);
                        self.record.set_root_property(
                            go_game::Property::SZ,
                            vec![self.new_game_board_size.to_string()],
                        );
                        self.record.set_root_property(
                            go_game::Property::RU,
                            vec![self.new_game_rules.clone()],
                        );
                        self.record.set_root_property(
                            go_game::Property::KM,
                            vec![self.new_game_komi.clone()],
                        );

                        self.info_game_name.clear();
                        self.info_black.clear();
                        self.info_black_rank.clear();
                        self.info_white.clear();
                        self.info_white_rank.clear();
                        self.info_event.clear();
                        self.info_round.clear();
                        self.info_place.clear();
                        self.info_date.clear();
                        self.info_result.clear();
                        self.info_komi = self.new_game_komi.clone();
                        self.info_rules = self.new_game_rules.clone();
                        self.info_handicap.clear();
                        self.info_black_team.clear();
                        self.info_white_team.clear();
                        self.info_user.clear();
                        close_dialog = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }
                });
            });

        if close_dialog {
            self.show_new_game_dialog = false;
        }
    }
}

fn decode_sgf_content(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).to_string();
    }

    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        let (decoded, _, had_errors) = encoding_rs::UTF_16LE.decode(bytes);
        if !had_errors {
            return decoded.into_owned();
        }
    }

    let mut detector = chardetng::EncodingDetector::new(Iso2022JpDetection::Allow);
    detector.feed(bytes, true);
    let detected_encoding = detector.guess(None, chardetng::Utf8Detection::Allow);

    let (result, had_errors) = detected_encoding.decode_with_bom_removal(bytes);

    if !had_errors {
        let result_str = result.to_string();
        if !result_str
            .chars()
            .any(|c: char| c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            return result_str;
        }
    }

    let encodings_priority = [
        (encoding_rs::GBK, "GBK"),
        (encoding_rs::GB18030, "GB18030"),
        (encoding_rs::BIG5, "BIG5"),
        (encoding_rs::SHIFT_JIS, "SHIFT_JIS"),
        (encoding_rs::EUC_JP, "EUC-JP"),
        (encoding_rs::EUC_KR, "EUC-KR"),
        (encoding_rs::UTF_8, "UTF-8"),
    ];

    for (encoding, _name) in encodings_priority {
        let (decoded, _, had_errors) = encoding.decode(bytes);
        if !had_errors {
            let s = decoded.to_string();
            if !s
                .chars()
                .any(|c: char| c.is_control() && c != '\n' && c != '\r' && c != '\t')
            {
                return s;
            }
        }
    }

    String::from_utf8_lossy(bytes).to_string()
}

fn draw_board(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    board: &Board,
    show_coords: bool,
    next_moves: &[(go_game::model::Color, Point)],
) {
    let painter = ui.painter_at(rect);
    let size = board.size as usize;

    painter.rect_filled(rect, 0.0, Color32::from_rgb(230, 190, 120));

    let pad = (rect.width() * 0.03).clamp(2.0, 20.0);
    let inner_rect = rect.shrink(2.0);
    let drawing_rect = egui::Rect::from_min_max(
        inner_rect.min + egui::vec2(pad, pad),
        inner_rect.max - egui::vec2(pad, pad),
    );
    let cell = drawing_rect.width() / ((size as f32 - 1.0).max(1.0));

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
                let star_radius = cell * 0.08;
                painter.circle_filled(egui::pos2(cx, cy), star_radius, Color32::BLACK);
            }
        }
    }

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

    // 绘制下一步提示（半透明棋子）
    if !next_moves.is_empty() {
        for &(col, pt) in next_moves {
            let cx = drawing_rect.left() + pt.x as f32 * cell;
            let cy = drawing_rect.top() + pt.y as f32 * cell;
            let radius = cell * 0.42;
            match col {
                Color::Black => {
                    let base = Color32::from_gray(10);
                    painter.circle_filled(egui::pos2(cx, cy), radius, base.gamma_multiply(0.4));
                }
                Color::White => {
                    let base = Color32::from_gray(240);
                    painter.circle_filled(egui::pos2(cx, cy), radius, base.gamma_multiply(0.4));
                    painter.circle_stroke(
                        egui::pos2(cx, cy),
                        radius,
                        Stroke::new(1.0, Color32::GRAY),
                    );
                }
            }
        }
    }

    if show_coords {
        let font_id = egui::FontId::proportional((cell * 0.35).max(1.0));
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

fn screen_pos_to_point(rect: egui::Rect, pos: egui::Pos2, size: u8) -> Option<Point> {
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
