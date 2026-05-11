use chardetng::{self, Iso2022JpDetection};
use eframe::egui;
use egui::{Color32, Stroke, Vec2};
use std::collections::HashMap;

use go_game::Board;
use go_game::model::{Color, Move, Point};
use go_game::record::{GoRecord, NodeInfo};
use go_game::sgf::{export, parse};

pub struct GoGui {
    record: GoRecord,
    edit_mode: bool,
    show_tree: bool,
    show_coords: bool,
    dark_theme: bool,
    comment_edit: String,
    show_comment_panel: bool,
    context_node: Option<usize>,
    context_pos: egui::Pos2,
    show_context_window: bool,
    info_black: String,
    info_white: String,
    info_date: String,
    info_komi: String,
    info_result: String,
    show_info_window: bool,
    show_error_window: bool,
    error_message: String,
    show_illegal_move_popup: bool,
    illegal_move_error: Option<String>,
}

impl GoGui {
    pub fn new() -> Self {
        Self {
            record: GoRecord::new(19),
            edit_mode: true,
            show_tree: true,
            show_coords: true,
            dark_theme: false,
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
            show_illegal_move_popup: false,
            illegal_move_error: None,
        }
    }
}

impl eframe::App for GoGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|input| {
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

        self.top_panel(ctx);
        self.status_bar(ctx);
        self.central_panel(ctx);
        self.info_window(ctx);
        self.error_window(ctx);
        self.illegal_move_popup(ctx);
        self.context_menu(ctx);
    }
}

impl GoGui {
    /// 菜单栏
    fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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
                self.record.new_game();
                self.info_black.clear();
                self.info_white.clear();
                self.info_date.clear();
                self.info_komi.clear();
                self.info_result.clear();
                ui.close_menu();
            }
            if ui.button("Open SGF").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    // 先用字节读取，再自动检测编码并解码
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            // 如果文件为空，直接当作空字符串；否则检测编码
                            let content = if bytes.is_empty() {
                                String::new()
                            } else {
                                let mut detector =
                                    chardetng::EncodingDetector::new(Iso2022JpDetection::Allow);
                                detector.feed(&bytes, true); // true 表示已经是全部数据
                                let encoding =
                                    detector.guess(None, chardetng::Utf8Detection::Allow); // true 允许检测为 UTF-8
                                let (cow, _had_errors) = encoding.decode_with_bom_removal(&bytes);
                                cow.into_owned() // Cow<str> -> String
                            };

                            match parse(&content) {
                                Ok(tree) => {
                                    self.record.load_sgf(tree);
                                    let info = self.record.get_game_info();
                                    self.info_black = info.black.unwrap_or_default();
                                    self.info_white = info.white.unwrap_or_default();
                                    self.info_date = info.date.unwrap_or_default();
                                    self.info_komi = info.komi.unwrap_or_default();
                                    self.info_result = info.result.unwrap_or_default();
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
                ui.close_menu();
            }
            if ui.button("Save As").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("sgf file", &["sgf"])
                    .save_file()
                {
                    let s = export(&self.record.tree);
                    let _ = std::fs::write(path, s);
                }
                ui.close_menu();
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
                ui.close_menu();
            }
            if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                self.record.redo();
                ui.close_menu();
            }
            if ui
                .button(format!(
                    "Toggle Edit Mode ({})",
                    if self.edit_mode { "ON" } else { "OFF" }
                ))
                .clicked()
            {
                self.edit_mode = !self.edit_mode;
                ui.close_menu();
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
        });
    }

    fn help_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Help", |ui| {
            ui.label("Go SGF Editor");
        });
    }

    fn central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
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

                ui.allocate_ui_at_rect(left_rect, |ui| {
                    // 棋盘绘制（与之前逻辑相同）
                    let avail_child = ui.available_rect_before_wrap();
                    let board_size = avail_child.width().min(avail_child.height());
                    let center = avail_child.center();
                    let min_pos = center - Vec2::splat(board_size * 0.5);
                    let board_rect = egui::Rect::from_min_size(min_pos, Vec2::splat(board_size));

                    let response = ui.allocate_rect(board_rect, egui::Sense::click());
                    let board_rect = response.rect;

                    draw_board(ui, board_rect, &self.record.board, self.show_coords);

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
                });

                ui.allocate_ui_at_rect(tree_rect, |ui| {
                    ui.label("Game Tree");

                    // 以下内容为原 tree_panel 中的绘制逻辑，已适配为在子 UI 中使用
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
                                    painter.circle_filled(center, dot_size * 0.45, Color32::BLACK);
                                }
                                2 => {
                                    painter.circle_filled(center, dot_size * 0.45, Color32::WHITE);
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
                                    egui::pos2(node_rect.right() + 6.0, node_rect.center().y - 6.0),
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
                            ui.add(
                                egui::TextEdit::multiline(&mut self.comment_edit).desired_rows(4),
                            );
                        }
                    });
                });
            } else {
                // 无树时保持原始棋盘居中显示
                let board_size = avail.width().min(avail.height());
                let center = avail.center();
                let min_pos = center - Vec2::splat(board_size * 0.5);
                let board_rect = egui::Rect::from_min_size(min_pos, Vec2::splat(board_size));

                let response = ui.allocate_rect(board_rect, egui::Sense::click());
                let board_rect = response.rect;

                draw_board(ui, board_rect, &self.record.board, self.show_coords);

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

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
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
                    self.info_black = info.black.unwrap_or_default();
                    self.info_white = info.white.unwrap_or_default();
                    self.info_date = info.date.unwrap_or_default();
                    self.info_komi = info.komi.unwrap_or_default();
                    self.info_result = info.result.unwrap_or_default();
                    self.show_info_window = true;
                }
            });
        });
    }

    fn info_window(&mut self, ctx: &egui::Context) {
        if !self.show_info_window {
            return;
        }

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
                    let info = go_game::record::GameInfo {
                        black: if self.info_black.is_empty() {
                            None
                        } else {
                            Some(self.info_black.clone())
                        },
                        white: if self.info_white.is_empty() {
                            None
                        } else {
                            Some(self.info_white.clone())
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
                    };
                    self.record.set_game_info(&info);
                    self.show_info_window = false;
                }
                if ui.button("Cancel").clicked() {
                    self.show_info_window = false;
                }
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
                        ui.close_menu();
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.context_node = None;
                    self.show_context_window = false;
                    ui.close_menu();
                }
            });
    }
}

fn draw_board(ui: &mut egui::Ui, rect: egui::Rect, board: &Board, show_coords: bool) {
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

    if show_coords {
        let font_id = egui::FontId::proportional((cell * 0.35).max(1.0));
        //下方坐标
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
        // 右边坐标
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
